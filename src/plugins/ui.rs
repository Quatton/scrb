use backends::rapier::RapierPickable;
use bevy::{input::mouse::MouseButtonInput, prelude::*};
use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::prelude::*;
use bevy_mod_reqwest::{reqwest::header, BevyReqwest, ReqResponse};
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputSubmitEvent};
use serde::{Deserialize, Serialize};

use crate::components::{
    core::LockedAxesBundle,
    modifier::{Dictionary, Modifier},
};

use super::player::Player;

const BORDER_COLOR_ACTIVE: Color = Color::rgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Component)]
pub struct TypingUi;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum TypingState {
    #[default]
    IsMoving,
    IsTyping,
}

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TypingState>()
            .add_systems(OnEnter(TypingState::IsTyping), setup_ui_on_typing)
            .add_systems(OnExit(TypingState::IsTyping), kill_ui_on_typing)
            .add_systems(
                Update,
                (
                    typing_toggler,
                    command_listener,
                    spawn_listener,
                    update_handle_drag,
                ),
            )
            .add_systems(
                Update,
                on_drag_end_despawn.run_if(any_with_component::<PickingAnchor>),
            )
            .add_systems(Update, on_drag_start)
            .add_event::<FalResponse>()
            .add_systems(Update, handle_asset_generation);
        // .add_systems(Update, click_listener);
        // .add_systems(
        //     Update,
        //     attach_collider_to_scene.run_if(any_with_component::<PendingCollider>),
        // );
    }
}

fn _click_listener(
    mut commands: Commands,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_query: Query<(&Transform, &Player), Without<Camera3d>>,
    window_query: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for mousebtn_ev in mousebtn_evr.read() {
        if !mousebtn_ev.state.is_pressed() && mousebtn_ev.button != MouseButton::Left {
            continue;
        }

        let (cm, ct) = camera_query.single();

        let cursor_pos = window_query
            .get(mousebtn_ev.window)
            .unwrap()
            .cursor_position()
            .unwrap();

        let cursor_ray = cm.viewport_to_world(ct, cursor_pos).unwrap();

        let distance = cursor_ray
            .intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Z))
            .unwrap();

        let hit_point = cursor_ray.get_point(distance);

        let player_transform = player_query.single().0;

        let start = player_transform.translation + Vec3::Y * 3.0;

        let d = hit_point - start;
        let dx = d.x;
        let dy = d.y;

        let angle = dy.atan2(dx) / 2.0 + std::f32::consts::FRAC_PI_4;
        let tan_angle = angle.tan();

        let speed = (9.81 * (dx).powi(2) * (tan_angle.powi(2) + 1.0)
            / (2.0 * (dx * tan_angle - dy)))
            .sqrt();

        commands.spawn((
            RapierPickable,
            RigidBody::Dynamic,
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.5, 0.5, 0.5),
                    ..default()
                }),
                transform: Transform::from_translation(start),
                ..default()
            },
            Collider::ball(0.5),
            Velocity {
                linvel: Vec3::new(speed * angle.cos(), speed * angle.sin(), 0.0),
                angvel: Vec3::ZERO,
            },
            ColliderMassProperties::Density(1.0),
            LockedAxesBundle::default(),
        ));
    }
}

fn on_drag_start(
    mut commands: Commands,
    mut events: EventReader<Pointer<DragStart>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    entity_query: Query<(&Transform, &GlobalTransform), Without<Camera3d>>,
) {
    for event in events.read() {
        let (cm, ct) = camera_query.single();

        let cursor_ray = cm
            .viewport_to_world(ct, event.pointer_location.position)
            .unwrap();

        let distance = cursor_ray
            .intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Z))
            .unwrap();

        let hit_point = cursor_ray.get_point(distance);

        let transform = Transform::from_translation(hit_point);

        let (target_transform, _) = entity_query.get(event.target).unwrap();

        let anchor = target_transform
            .with_scale(Vec3::splat(1.0))
            .compute_matrix()
            .inverse()
            .transform_point3(hit_point);
        let joint = RopeJointBuilder::new(0.5)
            .local_anchor1(anchor)
            .local_anchor2(Vec3::ZERO);

        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(0.0, 0.0, 0.0),
            ColliderDisabled,
            ColliderMassProperties::Density(0.0),
            TransformBundle::from_transform(transform),
            PickingAnchor,
            ImpulseJoint::new(event.target, joint),
            LockedAxesBundle::default(),
        ));
    }
}

fn kill_ui_on_typing(mut commands: Commands, ui_query: Query<Entity, With<TypingUi>>) {
    if let Ok(entity) = ui_query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_ui_on_typing(mut commands: Commands) {
    commands
        .spawn((
            TypingUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    border_color: BORDER_COLOR_ACTIVE.into(),
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default().with_text_style(TextStyle {
                    font_size: 40.,
                    color: TEXT_COLOR,
                    ..default()
                }),
            ));
        });
}

fn command_listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    everything_query: Query<(Entity, &SpawnedObject)>,
) {
    for event in events.read() {
        let TextInputSubmitEvent { value, .. } = event;

        if value.starts_with('/') {
            handle_command(value, &mut commands, &everything_query);
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]

struct FalOutput {
    image: String,
}

#[derive(Deserialize, Debug, Event)]
struct FalResponse {
    target: Entity,
    image_url: String,
}

impl From<ListenerInput<ReqResponse>> for FalResponse {
    fn from(input: ListenerInput<ReqResponse>) -> Self {
        let output: FalOutput = input.deserialize_json().unwrap();
        FalResponse {
            target: input.target(),
            image_url: output.image,
        }
    }
}

#[allow(dead_code)]
#[derive(Component)]
struct AssetGenerator(String);

fn handle_asset_generation(
    mut commands: Commands,
    mut events: EventReader<FalResponse>,
    asset_server: Res<AssetServer>,
    mat_query: Query<&Handle<StandardMaterial>, With<AssetGenerator>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    for event in events.read() {
        let FalResponse { image_url, target } = event;

        let texture: Handle<Image> =
            asset_server.load(image_url);

        let mat = mat_query.get(*target).unwrap();
        let material = materials.get_mut(mat).unwrap();

        material.base_color_texture = Some(texture);

       commands.entity(*target).remove::<AssetGenerator>();
    }
}

fn spawn_listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dictionary: ResMut<Dictionary>,
    mut client: BevyReqwest,
) {
    for event in events.read() {
        let TextInputSubmitEvent { value, .. } = event;

        if !value.starts_with('/') {
            let mut parts = value.split_whitespace().rev();
            if parts.clone().count() < 1 {
                return;
            }

            let mut ent = commands.spawn((
                RapierPickable,
                RigidBody::Dynamic,
                LockedAxesBundle::default(),
                PickableBundle::default(),
                SpawnedObject,
                ColliderMassProperties::Density(1.0),
                On::<Pointer<DragStart>>::target_commands_mut(|_, cmd| {
                    cmd.insert(Pickable::IGNORE);
                }), // Disable picking
                On::<Pointer<DragEnd>>::target_commands_mut(|_, cmd| {
                    cmd.insert(Pickable::default());
                    cmd.insert(Velocity {
                        angvel: Vec3::ZERO,
                        linvel: Vec3::ZERO,
                    });
                }), // Enable picking
            ));

            let noun = parts.next().unwrap_or("ball").to_string();
            let mut material = StandardMaterial::default();
            let mut transform = Transform::from_xyz(0.0, 20.0, 0.0);
            for adj in parts {
                if let Some(entry) = dictionary.search(adj).first() {
                    for modifier in entry.modifier.clone() {
                        match modifier {
                            Modifier::ColorModifier(color) => material.base_color = color,
                            Modifier::ScaleModifier(scale) => {
                                transform.scale = Vec3::splat(scale);
                                transform.translation.y += scale * 0.5;
                            }
                            Modifier::RoughnessModifier(roughness) => {
                                material.perceptual_roughness = if roughness < 0.089 {
                                    0.089
                                } else if roughness > 1.0 {
                                    1.0
                                } else {
                                    roughness
                                };
                            }
                            Modifier::MetallicModifier(metallic) => {
                                material.metallic = if metallic < 0.0 {
                                    0.0
                                } else if metallic > 1.0 {
                                    1.0
                                } else {
                                    metallic
                                };
                            }
                            Modifier::ReflectanceModifier(reflectance) => {
                                material.reflectance = if reflectance < 0.0 {
                                    0.0
                                } else if reflectance > 1.0 {
                                    1.0
                                } else {
                                    reflectance
                                };
                            }
                        }
                    }
                }
            }

            match noun.as_str() {
                "ball" => {
                    ent.insert((
                        PbrBundle {
                            mesh: meshes.add(Circle { radius: 0.5 }),
                            material: materials.add(material),
                            transform,
                            ..default()
                        },
                        Collider::ball(0.5),
                    ));
                }
                "cube" => {
                    ent.insert((
                        PbrBundle {
                            mesh: meshes.add(Cuboid {
                                half_size: Vec3::splat(0.5),
                            }),
                            material: materials.add(material),
                            transform,
                            ..default()
                        },
                        Collider::cuboid(0.5, 0.5, 0.5),
                    ));
                }
                _ => {
                    let req = client
                            
                            .post("https://fal.run/workflows/quatton/scrb-2d")
                            .header(header::AUTHORIZATION, format!("Key {}", std::env::var("FAL_KEY").unwrap()))
                            .json(&serde_json::json!({
                                "prompt": format!("{}, scribblenauts style, side profile, side view, 2d, stationary pose, simple background, subtle shading, thick outline, simple colors", noun),
                                "negative_prompt": "anime, 3d, isometric, front-facing, portrait, front profile, moving, detailed background, colorful, realistic, detailed shading, thin outline, sketch",
                            })).build().unwrap();

                    client.send_using_entity(ent.id(), req, On::send_event::<FalResponse>());

                    ent.insert((
                        PbrBundle {
                            mesh: meshes.add(Rectangle::from_size(Vec2::new(1.0, 1.0))),
                            material: materials.add(material),
                            transform,
                            ..default()
                        },
                        Collider::ball(0.5),
                        AssetGenerator(noun),
                    ));
                }
            }
        }
    }
}

fn handle_command(
    value: &str,
    commands: &mut Commands,
    everything_query: &Query<(Entity, &SpawnedObject)>,
) {
    let mut parts = value.trim_start_matches('/').split_whitespace();
    let noun = parts.next().unwrap_or("clear");

    #[allow(clippy::single_match)]
    match noun {
        "clear" => {
            for (entity, _) in everything_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        _ => {}
    }
}

#[derive(Component)]
pub struct SpawnedObject;

pub enum MeshOrScene {
    Mesh(Mesh),
    Scene(Handle<Scene>),
    Loading(String),
    MeshHandle(Handle<Mesh>),
}

#[derive(Component)]
pub enum PbrOrScene {
    Pbr(PbrBundle),
    Scene(SceneBundle),
}

#[derive(Component)]
pub struct PendingCollider;

#[derive(Resource)]
pub struct PythonStdin {
    pub stdin: std::process::ChildStdin,
}

// fn run_python_backend(mut commands: Commands) {
//     // run subprocess
//     let mut child = std::process::Command::new("/Users/quatton/.pyenv/versions/tsr/bin/python")
//         .current_dir("/Users/quatton/Documents/GitHub/TripoSR")
//         .arg("fal.py")
//         .arg("--output-dir")
//         .arg("/Users/quatton/Documents/GitHub/scrb/assets/models")
//         .arg("--no-remove-bg")
//         .arg("--pipe-to-3d")
//         .arg("--mc-resolution")
//         .arg("32")
//         // pipe stdin
//         .stdin(std::process::Stdio::piped())
//         .stdout(std::process::Stdio::null())
//         // background this
//         .spawn()
//         .expect("failed to execute process");

//     commands.insert_resource(PythonStdin {
//         stdin: child.stdin.take().unwrap(),
//     });
// }

#[derive(Component)]
pub struct PickingAnchor;

fn on_drag_end_despawn(
    mut commands: Commands,
    mut events: EventReader<Pointer<DragEnd>>,
    query: Query<Entity, With<PickingAnchor>>,
) {
    for _ in events.read() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_handle_drag(
    mut events: EventReader<Pointer<Drag>>,
    mut entity_query: Query<(&mut Transform, &PickingAnchor), Without<Camera3d>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for event in events.read() {
        let (cm, ct) = camera_query.single();
        if let Ok((mut transform, _)) = entity_query.get_single_mut() {
            let cursor_pos = cm
                .viewport_to_world(ct, event.pointer_location.position)
                .unwrap();
            let distance = cursor_pos
                .intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::new(0.0, 0.0, 1.0)))
                .unwrap();

            transform.translation = cursor_pos.get_point(distance);
        }
    }
}

fn typing_toggler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    typing_state: Res<State<TypingState>>,
    mut next_state: ResMut<NextState<TypingState>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Slash, KeyCode::Escape, KeyCode::Enter]) {
        match typing_state.get() {
            TypingState::IsMoving => next_state.set(TypingState::IsTyping),
            TypingState::IsTyping => next_state.set(TypingState::IsMoving),
        }
    }
}
