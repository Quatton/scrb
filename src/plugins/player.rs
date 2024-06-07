use crate::plugins::assets::AssetLoadingState;
use bevy::{gltf, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::plugins::assets::player_assets::*;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 1.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetLoadingState::Next), setup_player)
            .add_systems(
                Update,
                setup_scene_once_loaded.run_if(in_state(AssetLoadingState::Next)),
            );
    }
}

#[derive(Default, Component)]
pub struct Player;

fn setup_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands
        .spawn((
            SceneBundle {
                scene: player_assets.model.clone_weak(),
                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Player,
        ))
        .with_children(|p| {
            p.spawn((
                Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
                Transform::from_translation(Vec3::new(0.0, PLAYER_HEIGHT, 0.0)),
            ));
        });
}

fn setup_scene_once_loaded(
    player_assets: Res<PlayerAssets>,
    gltf_assets: Res<Assets<gltf::Gltf>>,
    mut animation_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    let gltf = gltf_assets.get(player_assets.gltf.clone_weak()).unwrap();
    for mut player in &mut animation_players {
        player
            .play(gltf.named_animations["Idle"].clone_weak())
            .repeat();
    }
}
