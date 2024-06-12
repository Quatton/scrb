use std::{io::Write, time::Duration};

use backends::raycast::RaycastPickable;
use bevy::{prelude::*, utils::HashMap};
use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputSubmitEvent};

use crate::components::{
    core::LockedAxesBundle,
    modifier::{Dictionary, Modifier},
};

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
        Self(Timer::from_seconds(60.0, TimerMode::Once))
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
            );
        // .add_systems(
        //     Update,
        //     attach_collider_to_scene.run_if(any_with_component::<PendingCollider>),
        // );
    }
}

fn poll_mesh_until_loaded_or_timeout(
    time: Res<Time>,
    mut commands: Commands,
    mut timeout: Local<HashMap<Entity, MeshLoadingPoller>>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Transform, &MeshLoading)>,
) {
    for (entity, transform, mesh_loading) in query.iter() {
        match timeout.get_mut(&entity) {
            Some(timer) => {
                if timer.0.tick(time.delta()).just_finished() {
                    commands.entity(entity).despawn_recursive();
                    timeout.remove(&entity);
                }
                let noun = &mesh_loading.noun;
                if std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
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
                timeout.insert(entity, MeshLoadingPoller::default());
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
            handle_spawning_object(
                value,
                &mut dictionary,
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut python_stdin,
            );
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
        .arg("/Users/quatton/Documents/GitHub/TripoSR/realtime.py")
        .arg("--output-dir")
        .arg("/Users/quatton/Documents/GitHub/scrb/assets/models")
        .arg("--pipe-to-3d")
        .arg("--mc-resolution")
        .arg("128")
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

fn handle_spawning_object(
    value: &str,
    dictionary: &mut ResMut<Dictionary>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    python_stdin: &mut ResMut<PythonStdin>,
) {
    // let thread_pool = AsyncComputeTaskPool::get();

    let mut parts = value.split_whitespace().rev();
    if parts.clone().count() < 1 {
        return;
    }

    let mut ent = commands.spawn((
        RaycastPickable,
        RigidBody::Dynamic,
        LockedAxesBundle::default(),
        PickableBundle::default(),
        SpawnedObject,
        ColliderMassProperties::Density(1.0),
        On::<Pointer<DragStart>>::target_commands_mut(|drag, cmd| {
            cmd.insert(Pickable::IGNORE);
            // .insert(RigidBody::KinematicPositionBased);

            let joint = RopeJointBuilder::new(1.0);

            cmd.commands().spawn((
                RigidBody::Fixed,
                Collider::cuboid(0.0, 0.0, 0.0),
                ColliderMassProperties::Density(0.0),
                TransformBundle::from_transform(Transform::from_xyz(0.0, 3.0, 0.0)),
                // ColliderDisabled,
                PickingAnchor,
                ImpulseJoint::new(drag.target, joint),
                LockedAxesBundle::default(),
            ));
        }), // Disable picking
        On::<Pointer<DragEnd>>::target_commands_mut(|_, cmd| {
            cmd.insert(Pickable::default());
        }), // Enable picking
    ));

    // let id = ent.id();

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
                writeln!(stdin, "{}", noun).unwrap();

                // poll "models" directory until the file is created

                let noun = noun.to_string();
                // let task = thread_pool.spawn(async move {
                //     let mut command_queue = CommandQueue::default();

                //     let max_wait = 60;
                //     for _ in 0..max_wait {
                //         if std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists()
                //         {
                //             break;
                //         }
                //         std::thread::sleep(std::time::Duration::from_secs(1));
                //     }

                //     command_queue
                // });

                MeshOrScene::Loading(noun)
            } else {
                MeshOrScene::Scene(asset_server.load(format!("models/{noun}/mesh.glb#Scene0")))
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
                    transform: transform
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                },
                collider,
                // AsyncSceneCollider {
                //     shape: Some(ComputedColliderShape::TriMesh),
                //     named_shapes: default(),
                // },
            ));
        }
        MeshOrScene::Loading(noun) => {
            ent.insert((
                SceneBundle {
                    scene: asset_server.load("models/cubed/mesh.glb#Scene0"),
                    transform: transform
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                },
                collider,
                MeshLoading { noun },
            ));
        }
    }
}

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
    // mut entity_query: Query<(&RaycastPickable, &Children, &mut Transform), Without<Camera3d>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    projection_query: Query<&Projection>,
    window_query: Query<&Window>,
) {
    for event in events.read() {
        let ct = camera_query.single();
        let proj = projection_query.single();

        if let Projection::Perspective(_proj) = proj {
            let window = window_query.single();
            let centerx = window.width() / 2.0;
            let centery = window.height() / 2.0;
            if let Ok((mut transform, _)) = entity_query.get_single_mut() {
                // transform.translation = (ct.translation
                // // flip y axis
                // + (Vec3::new(event.pointer_location.position.x - centerx, centery - event.pointer_location.position.y, 0.0) / 30.0))
                // .reject_from(Vec3::Z);

                let cursor_pos = (ct.translation
                    + (Vec3::new(
                        event.pointer_location.position.x - centerx,
                        centery - event.pointer_location.position.y,
                        0.0,
                    ) / 30.0))
                    .reject_from(Vec3::Z);

                transform.translation = cursor_pos;

                // for child in children.iter() {
                //     if let Ok(_joint) = joint_query.get_mut(*child) {
                //         // *joint = ImpulseJoint::new(
                //         //     joint.parent,
                //         //     SphericalJointBuilder::new()
                //         //         .local_anchor1(cursor_pos - transform.translation)
                //         //         .local_anchor2(Vec3::new(0.0, 0.0, 0.0)),
                //         // );

                //     }
                // }
            }
        };
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
