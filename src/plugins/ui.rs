use std::io::Write;

use backends::rapier::RapierPickable;
use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputSubmitEvent};

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

#[derive(Component, Deref, DerefMut)]
struct MeshLoadingPoller(Timer);

impl Default for MeshLoadingPoller {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Component, Deref, DerefMut)]

struct MeshLoadingTimeout(Timer);
impl Default for MeshLoadingTimeout {
    fn default() -> Self {
        Self(Timer::from_seconds(120.0, TimerMode::Once))
    }
}

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TypingState>()
            .add_systems(Startup, run_python_backend)
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
                poll_mesh_until_loaded_or_timeout.run_if(any_with_component::<MeshLoading>),
            )
            .add_systems(
                Update,
                on_drag_end_despawn.run_if(any_with_component::<PickingAnchor>),
            )
            .add_systems(Update, on_drag_start);
        // .add_systems(Update, click_listener);
        // .add_systems(
        //     Update,
        //     attach_collider_to_scene.run_if(any_with_component::<PendingCollider>),
        // );
    }
}

fn click_listener(
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

        println!("Speed: {}, Angle: {}", speed, angle);

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

        // let max_toi = 100.0;
        // let solid = true;
        // let filter = QueryFilter::default();

        // let Some((_, toi)) = rapier_context.cast_ray(
        //     cursor_ray.origin,
        //     cursor_ray.direction.into(),
        //     max_toi,
        //     solid,
        //     filter,
        // ) else {
        //     return;
        // };

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

fn poll_mesh_until_loaded_or_timeout(
    time: Res<Time>,
    mut commands: Commands,
    mut timeout: Local<HashMap<Entity, MeshLoadingTimeout>>,
    mut timer: Local<MeshLoadingPoller>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Transform, &MeshLoading)>,
) {
    if timer.tick(time.delta()).just_finished() {
        for (entity, transform, mesh_loading) in query.iter() {
            match timeout.get_mut(&entity) {
                Some(timer) => {
                    if timer.tick(time.delta()).just_finished() {
                        {
                            commands.entity(entity).despawn_recursive();
                            timeout.remove(&entity);
                        }
                    }

                    let noun = &mesh_loading.noun;
                    if std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
                        println!("Successfully loaded {}", noun);

                        commands
                            .entity(entity)
                            .remove::<MeshLoading>()
                            .insert(SceneBundle {
                                scene: asset_server.load(format!("models/{noun}/mesh.glb#Scene0")),
                                transform: *transform,
                                ..default()
                            });

                        timeout.remove(&entity);
                    }
                }
                None => {
                    timeout.insert(entity, MeshLoadingTimeout::default());
                }
            }
        }
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

#[allow(clippy::too_many_arguments)]
fn spawn_listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dictionary: ResMut<Dictionary>,
    mut python_stdin: ResMut<PythonStdin>,
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

            let noun = parts.next().unwrap_or("ball");

            let collider = match noun {
                "cube" => Collider::cuboid(0.5, 0.5, 0.5),
                "ball" => Collider::ball(0.5),
                _ => Collider::cuboid(0.5, 0.5, 0.5),
            };
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

            let shape: MeshOrScene = match noun {
                "cube" => MeshOrScene::Mesh(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
                "ball" => MeshOrScene::Mesh(Mesh::from(Sphere::new(0.5))),
                _ => {
                    if !std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
                        let stdin = &mut python_stdin.stdin;

                        if writeln!(stdin, "{}", noun).is_ok() {
                            let noun = noun.to_string();
                            MeshOrScene::Loading(noun)
                        } else {
                            MeshOrScene::Mesh(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))
                        }
                    } else {
                        MeshOrScene::Scene(
                            asset_server.load(format!("models/{noun}/mesh.glb#Scene0")),
                        )
                    }
                }
            };

            match shape {
                MeshOrScene::Mesh(mesh) => {
                    ent.insert((
                        PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(material),
                            transform,
                            ..default()
                        },
                        collider,
                    ));
                }
                MeshOrScene::Scene(model) => {
                    ent.insert((
                        SceneBundle {
                            scene: model,
                            transform: transform.with_rotation(
                                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                                    * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                            ),
                            ..default()
                        },
                        collider,
                    ));
                }
                MeshOrScene::Loading(noun) => {
                    ent.insert((
                        SceneBundle {
                            scene: asset_server.load("models/mystery_block/mesh.glb#Scene0"),
                            transform: transform.with_rotation(
                                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                                    * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                            ),
                            ..default()
                        },
                        collider,
                        MeshLoading { noun },
                    ));
                }
                MeshOrScene::MeshHandle(handle) => {
                    ent.insert((
                        PbrBundle {
                            mesh: handle,
                            material: materials.add(material),
                            transform,
                            ..default()
                        },
                        collider,
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
pub struct MeshLoading {
    noun: String,
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

fn run_python_backend(mut commands: Commands) {
    // run subprocess
    let mut child = std::process::Command::new("/Users/quatton/.pyenv/versions/tsr/bin/python")
        .current_dir("/Users/quatton/Documents/GitHub/TripoSR")
        .arg("fal.py")
        .arg("--output-dir")
        .arg("/Users/quatton/Documents/GitHub/scrb/assets/models")
        .arg("--no-remove-bg")
        .arg("--pipe-to-3d")
        .arg("--mc-resolution")
        .arg("32")
        // pipe stdin
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        // background this
        .spawn()
        .expect("failed to execute process");

    commands.insert_resource(PythonStdin {
        stdin: child.stdin.take().unwrap(),
    });
}

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
