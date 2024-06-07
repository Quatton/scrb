use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::*;

const WORLD_WIDTH: f32 = 100.0;
const WALL_WIDTH: f32 = 10.0;
const WALL_HEIGHT: f32 = 50.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AtmospherePlugin, PanOrbitCameraPlugin))
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 200.0,
            })
            .add_systems(Startup, (setup_camera, setup_world));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 30.0, 100.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
    ));
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(WORLD_WIDTH, 20.0, WORLD_WIDTH))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Collider::compound(vec![
            // Floor
            (
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WORLD_WIDTH / 2.0, 10.0, WORLD_WIDTH / 2.0),
            ),
            // Walls
            // Wall X_NEG_HALF, Z_0
            (
                Vec3::new(-WORLD_WIDTH / 2.0, WALL_HEIGHT / 2.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WALL_WIDTH / 2.0, WALL_HEIGHT / 2.0, WORLD_WIDTH / 2.0),
            ),
            // Wall X_POS_HALF, Z_0
            (
                Vec3::new(WORLD_WIDTH / 2.0, WALL_HEIGHT / 2.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WALL_WIDTH / 2.0, WALL_HEIGHT / 2.0, WORLD_WIDTH / 2.0),
            ),
            // Wall X_0, Z_NEG_HALF
            (
                Vec3::new(0.0, WALL_HEIGHT / 2.0, -WORLD_WIDTH / 2.0),
                Quat::IDENTITY,
                Collider::cuboid(WORLD_WIDTH / 2.0, WALL_HEIGHT / 2.0, WALL_WIDTH / 2.0),
            ),
            // Wall X_0, Z_POS_HALF
            (
                Vec3::new(0.0, WALL_HEIGHT / 2.0, WORLD_WIDTH / 2.0),
                Quat::IDENTITY,
                Collider::cuboid(WORLD_WIDTH / 2.0, WALL_HEIGHT / 2.0, WALL_WIDTH / 2.0),
            ),
        ]),
        RigidBody::Fixed,
    ));
}
