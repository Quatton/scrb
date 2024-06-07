use std::time::Duration;

use crate::plugins::assets::AssetLoadingState;
use bevy::{gltf, prelude::*, transform};
use bevy_rapier3d::prelude::*;

use crate::plugins::assets::player_assets::*;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_BASE_SPEED: f32 = 2.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetLoadingState::Next), setup_player)
            .add_systems(
                Update,
                (
                    // setup_scene_once_loaded.run_if(in_state(AssetLoadingState::Next)),
                    kb_control.run_if(in_state(AssetLoadingState::Next)),
                ),
            );
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlayerState {
    Idle,
    Walk,
    Jumping,
}

#[derive(Default, Component)]
pub struct Player {
    pub state: PlayerState,
}

#[allow(clippy::derivable_impls)]
impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Idle
    }
}

fn setup_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands
        .spawn((
            SceneBundle {
                scene: player_assets.model.clone_weak(),
                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Player::default(),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            ExternalImpulse::default(),
            ColliderMassProperties::Density(1.0),
            GravityScale(3.0),
        ))
        .with_children(|p| {
            p.spawn((
                Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
                Transform::from_translation(Vec3::new(0.0, PLAYER_HEIGHT, 0.0)),
            ));
        });
}

fn kb_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform, &mut ExternalImpulse)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    player_assets: Res<PlayerAssets>,
    gltf_assets: Res<Assets<gltf::Gltf>>,
) {
    for (mut player, mut transform, mut ext) in query.iter_mut() {
        let mut state = if transform.translation.y < 10.5 {
            PlayerState::Idle
        } else {
            PlayerState::Jumping
        };

        let direction = (transform.rotation * Vec3::Z).reject_from_normalized(Vec3::Y);
        // let perpen_direction = (transform.rotation * Vec3::X).reject_from_normalized(Vec3::Y);

        if keyboard_input.just_pressed(KeyCode::Space) {
            state = PlayerState::Jumping;
            ext.impulse = Vec3::Y * 100.0;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            state = PlayerState::Walk;
            transform.translation += direction * PLAYER_BASE_SPEED * 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            state = PlayerState::Walk;
            transform.translation -= direction * PLAYER_BASE_SPEED * 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            state = PlayerState::Walk;
            transform.rotation *= Quat::from_rotation_y(0.05);
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            state = PlayerState::Walk;
            transform.rotation *= Quat::from_rotation_y(-0.05);
        }

        if player.state != state {
            let gltf = gltf_assets.get(player_assets.gltf.clone_weak()).unwrap();

            let animation = match state {
                PlayerState::Idle => "Idle",
                PlayerState::Walk => "Running_A",
                PlayerState::Jumping => "Jump_Idle",
            };

            player.state = state;

            for mut player in &mut animation_players {
                player
                    .play_with_transition(
                        gltf.named_animations[animation].clone_weak(),
                        Duration::from_millis(200),
                    )
                    .repeat();
            }
        }
    }
}

// fn setup_scene_once_loaded(
//     player_assets: Res<PlayerAssets>,
//     gltf_assets: Res<Assets<gltf::Gltf>>,
//     mut animation_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
// ) {
//     let gltf = gltf_assets.get(player_assets.gltf.clone_weak()).unwrap();
//     for mut player in &mut animation_players {
//         player
//             .play(gltf.named_animations["Idle"].clone_weak())
//             .repeat();
//     }
// }
