use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use bevy_mod_picking::backends::rapier::RapierPickable;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::components::core::FallPrevention;

use super::player::Player;

const WORLD_WIDTH: f32 = 100.0;
const WALL_WIDTH: f32 = 10.0;
const WALL_HEIGHT: f32 = 50.0;
const LOOKUP_OFFSET: f32 = 5.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmospherePlugin)
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 200.0,
            })
            .add_systems(Startup, (setup_camera, setup_world))
            .add_systems(Update, camera_tracking.run_if(any_with_component::<Player>))
            .add_systems(Update, teleport_offlimit_objects);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, LOOKUP_OFFSET, 30.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            // projection: Projection::Orthographic(OrthographicProjection {
            //     scale: 0.05,
            //     ..default()
            // }),
            ..default()
        },
        AtmosphereCamera::default(),
        RapierPickable,
    ));
}

fn camera_tracking(
    mut query: Query<(&mut Transform, &Camera3d), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single();
    let (mut camera, _) = query.single_mut();
    camera.translation = player_transform.translation + Vec3::new(0.0, LOOKUP_OFFSET, 30.0);
    *camera = camera.looking_at(
        player_transform.translation + Vec3::new(0.0, LOOKUP_OFFSET, 0.0),
        Vec3::Y,
    );
}

fn teleport_offlimit_objects(mut query: Query<&mut Transform, With<FallPrevention>>) {
    let mut spawn_pos = Vec3::new(0.0, 20.0, 0.0);
    let mut gen = rand::thread_rng();
    spawn_pos.x += gen.gen_range(-WORLD_WIDTH / 2.0..WORLD_WIDTH / 2.0);
    spawn_pos.y += gen.gen_range(-10.0..10.0);
    for mut transform in query.iter_mut() {
        if transform.translation.y < -10.0 || transform.translation.y > 100.0 {
            transform.translation = spawn_pos;
        }
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(WORLD_WIDTH, 20.0, WALL_WIDTH))),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Restitution::coefficient(0.0),
        Collider::compound(vec![
            // Floor
            (
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WORLD_WIDTH / 2.0, 10.0, WALL_WIDTH / 2.0),
            ),
            // Walls
            // Wall X_NEG_HALF
            (
                Vec3::new(-(WORLD_WIDTH + WALL_WIDTH) / 2.0, WALL_HEIGHT / 2.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WALL_WIDTH / 2.0, WALL_HEIGHT / 2.0, WORLD_WIDTH / 2.0),
            ),
            // Wall X_POS_HALF
            (
                Vec3::new((WORLD_WIDTH + WALL_WIDTH) / 2.0, WALL_HEIGHT / 2.0, 0.0),
                Quat::IDENTITY,
                Collider::cuboid(WALL_WIDTH / 2.0, WALL_HEIGHT / 2.0, WORLD_WIDTH / 2.0),
            ),
        ]),
        RigidBody::Fixed,
        Friction::coefficient(0.5),
    ));
}
