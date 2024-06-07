use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::*;

const WORLD_WIDTH: f32 = 200.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AtmospherePlugin, PanOrbitCameraPlugin))
            .add_systems(Startup, (setup_camera, setup_world));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 100.0, 100.0))
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
        Collider::cuboid(WORLD_WIDTH / 2.0, 10.0, WORLD_WIDTH / 2.0),
        RigidBody::Fixed,
    ));
}
