use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 2.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(PLAYER_RADIUS, PLAYER_HEIGHT)),
            material: materials.add(Color::rgb(0.5, 0.5, 0.8)),
            transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
            ..default()
        },
        Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
        RigidBody::Dynamic,
    ));
}
