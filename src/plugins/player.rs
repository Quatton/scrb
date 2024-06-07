use crate::plugins::assets::AssetLoadingState;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::plugins::assets::*;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 2.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetLoadingState::Next), setup_player);
    }
}

#[derive(Default, Component)]
pub struct Player;

fn setup_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands
        .spawn((
            SceneBundle {
                scene: player_assets.model.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
                ..default()
            },
            CustomizePlayerMaterial,
            RigidBody::Dynamic,
        ))
        .with_children(|p| {
            p.spawn((
                Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
                Transform::from_translation(Vec3::new(0.0, PLAYER_HEIGHT, 0.0)),
            ));
        });
}
