use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

const WORLD_WIDTH: f32 = 2000.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_systems(Startup, (setup_camera, setup_world))
        .add_systems(Startup, setup_player)
        .add_systems(
            Update,
            (kb_input_system, respawn_player_system, text_input_system),
        )
        .add_plugins(RapierDebugRenderPlugin::default())
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}

fn setup_world(mut commands: Commands) {
    // Create the ground
    commands.spawn((
        Collider::cuboid(WORLD_WIDTH / 2.0, 50.0),
        Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
        RigidBody::Fixed,
    ));
}

#[derive(Component, Default)]
struct Player {
    typing: bool,
}

#[derive(Component)]
struct PlayerTextInput;

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: bevy::sprite::Mesh2dHandle(meshes.add(Capsule2d::new(10.0, 50.0))),
            material: materials.add(Color::rgb(0.0, 0.0, 1.0)),
            transform: Transform {
                translation: Vec3::new(0.0, 50.0, 0.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        ColliderMassProperties::Density(1.0),
        Player::default(),
        Collider::capsule(
            // start
            Vec2::new(0.0, 25.0),
            // end
            Vec2::new(0.0, -25.0),
            // radius
            10.0,
        ),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                }],
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(-300.0, 200.0, 0.0)),
            ..default()
        },
        PlayerTextInput,
    ));
}

#[derive(Component)]
struct Scribble;

fn text_input_system(
    mut e_chr: EventReader<ReceivedCharacter>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
    mut text_query: Query<&mut Text, With<PlayerTextInput>>,
) {
    let mut ply = player_query.single_mut();

    if !ply.typing {
        return;
    }

    let mut text = text_query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Enter) {
        ply.typing = false;

        let words = text.sections[0]
            .value
            .split_whitespace()
            .collect::<Vec<&str>>();

        // ["color", "cube"]
        if words.len() == 2 {
            let color = match words[0] {
                "red" => Color::RED,
                "green" => Color::GREEN,
                "blue" => Color::BLUE,
                _ => Color::WHITE,
            };

            let (collider, shape) = match words[1] {
                "cube" => (
                    Collider::cuboid(25.0, 25.0),
                    meshes.add(Cuboid::new(50.0, 50.0, 0.0)),
                ),
                "circle" => (Collider::ball(25.0), meshes.add(Circle::new(25.0))),
                _ => return,
            };

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(shape),
                    material: materials.add(color),
                    transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
                    ..default()
                },
                RigidBody::Dynamic,
                ColliderMassProperties::Density(1.0),
                Scribble,
                collider,
            ));
        }

        text.sections[0].value = "".to_string();
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Backspace) {
        text.sections[0].value.pop();
        return;
    }

    for chr in e_chr.read() {
        if chr.char.is_ascii() {
            text.sections[0].value += &chr.char.to_string();
        }
    }
}

fn kb_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player), Without<Camera2d>>,
    mut camera_query: Query<(&mut Transform, &Camera2d)>,
) {
    let (mut ext, mut ply) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::KeyI) && !ply.typing {
        ply.typing = !ply.typing;
        return;
    }

    if ply.typing {
        return;
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        ext.translation.y += 10.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        ext.translation.x -= 20.0;
        ext.translation.x = ext
            .translation
            .x
            .clamp(-WORLD_WIDTH / 2.0, WORLD_WIDTH / 2.0);
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        ext.translation.x += 20.0;
        ext.translation.x = ext
            .translation
            .x
            .clamp(-WORLD_WIDTH / 2.0, WORLD_WIDTH / 2.0);
    }

    // Camera follow player
    let (mut camera_ext, _) = camera_query.single_mut();

    // don't let the camera go off the world
    camera_ext.translation.x = ext
        .translation
        .x
        .clamp(-WORLD_WIDTH / 2.0, WORLD_WIDTH / 2.0);

    // move the camera up and down with the player
    camera_ext.translation.y = ext.translation.y;
}

fn respawn_player_system(mut player_query: Query<(Entity, &mut Transform), With<Player>>) {
    for (_entity, mut transform) in player_query.iter_mut() {
        if transform.translation.y < 0.0 {
            transform.translation = Vec3::new(0.0, 50.0, 0.0);
        }
    }
}
