use std::io::Write;

use backends::raycast::RaycastPickable;
use bevy::prelude::*;
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
            );
        // .add_systems(
        //     Update,
        //     attach_collider_to_scene.run_if(any_with_component::<PendingCollider>),
        // );
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

fn handle_spawning_object(
    value: &str,
    dictionary: &mut ResMut<Dictionary>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    python_stdin: &mut ResMut<PythonStdin>,
) {
    let mut parts = value.split_whitespace().rev();
    if parts.clone().count() < 1 {
        return;
    }
    let noun = parts.next().unwrap_or("ball");
    let shape: MeshOrScene = match noun {
        "cube" => MeshOrScene::Mesh(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
        "ball" => MeshOrScene::Mesh(Mesh::from(Sphere::new(0.5))),
        _ => {
            if !std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
                let stdin = &mut python_stdin.stdin;
                writeln!(stdin, "{}", noun).unwrap();

                // poll "models" directory until the file is created

                let max_wait = 60;
                for _ in 0..max_wait {
                    if std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }

            if std::path::Path::new(&format!("assets/models/{noun}/mesh.glb")).exists() {
                MeshOrScene::Scene(asset_server.load(format!("models/{noun}/mesh.glb#Scene0")))
            } else {
                MeshOrScene::Mesh(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))
            }
        }
    };
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
    let mut ent = commands.spawn((
        RaycastPickable,
        RigidBody::Dynamic,
        LockedAxesBundle::default(),
        PickableBundle::default(),
        SpawnedObject,
        ColliderMassProperties::Density(1.0),
        On::<Pointer<DragStart>>::target_commands_mut(|_, cmd| {
            cmd.insert((RigidBody::Fixed, Pickable::IGNORE, ColliderDisabled));
        }), // Disable picking
        On::<Pointer<DragEnd>>::target_commands_mut(|_, cmd| {
            cmd.insert((RigidBody::Dynamic, Pickable::default()));
            cmd.remove::<ColliderDisabled>();
            // Collider::from_bevy_mesh(mesh, &ComputedColliderShape::ConvexHull);
        }), // Enable picking
    ));

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

            // let mesh_handle =
            //     asset_server.load(format!("models/{noun}/0/mesh.glb#Mesh0/Primitive0"));
            // let mesh = meshes.get(&mesh_handle).unwrap();

            // let collider = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::ConvexHull);

            // if let Some(collider) = collider {
            //     ent.insert(collider);
            // } else {
            //     ent.insert(PendingCollider);
            // }
        }
    }
}

// fn attach_collider_to_scene(
//     mut commands: Commands,
//     scene_meshes: Query<(Entity, &Children), With<PendingCollider>>,
//     mesh_query: Query<&Handle<Mesh>>,
//     children_query: Query<&Children>,
//     meshes: Res<Assets<Mesh>>,
// ) {
//     // iterate over all meshes in the scene and match them by their name.
//     for (pending, _children) in &scene_meshes {
//         println!("pending collider: {:?}", pending);
//         for children in children_query.iter_descendants(pending) {
//             println!("children: {:?}", children);
//             if let Ok(mesh_handle) = mesh_query.get(children) {
//                 println!("mesh_handle: {:?}", mesh_handle);
//                 if let Some(mesh) = meshes.get(mesh_handle) {
//                     let collider =
//                     // Attach collider to the entity of this same object.

//                     if let Some(collider) = collider {
//                         commands.entity(pending).insert(collider);
//                     }

//                     break;
//                 }
//             }
//         }
//         // commands.entity(pending).remove::<Collider>();
//         commands.entity(pending).remove::<PendingCollider>();
//     }
// }

fn update_handle_drag(
    mut events: EventReader<Pointer<Drag>>,
    mut query: Query<(&mut Transform, &RaycastPickable), Without<Camera3d>>,
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
            if let Ok((mut transform, _)) = query.get_mut(event.target) {
                transform.translation = (ct.translation
                // flip y axis
                + (Vec3::new(event.pointer_location.position.x - centerx, centery - event.pointer_location.position.y, 0.0) / 30.0))
                .reject_from(Vec3::Z);
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
