use std::time::Duration;

use crate::{components::core::LockedAxesBundle, plugins::assets::AssetLoadingState};
use bevy::{gltf, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::plugins::assets::player_assets::*;

use super::ui::TypingState;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_BASE_SPEED: f32 = 20.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetLoadingState::Next), setup_player)
            .add_systems(
                Update,
                kb_control
                    .run_if(in_state(AssetLoadingState::Next))
                    .run_if(in_state(TypingState::IsMoving)),
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
                transform: Transform::from_translation(Vec3::new(0.0, 12.0, 0.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Player::default(),
            LockedAxesBundle::player(),
            ExternalImpulse::default(),
            ColliderMassProperties::Density(99999.0),
            GravityScale(3.0),
            Velocity::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
                Transform::from_translation(Vec3::new(0.0, PLAYER_HEIGHT, 0.0)),
            ));

            p.spawn(SpotLightBundle {
                transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                spot_light: SpotLight {
                    intensity: 1_000_000.0,           // lumens
                    color: Color::rgb(1.0, 0.9, 0.7), // candle light
                    shadows_enabled: true,
                    inner_angle: std::f32::consts::FRAC_PI_6,
                    outer_angle: std::f32::consts::FRAC_PI_3,
                    ..default()
                },
                ..default()
            });
        });
}

fn kb_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut Player,
        &mut Transform,
        &mut ExternalImpulse,
        &mut Velocity,
    )>,
    mut animation_players: Query<&mut AnimationPlayer>,
    player_assets: Res<PlayerAssets>,
    gltf_assets: Res<Assets<gltf::Gltf>>,
) {
    for (mut player, mut transform, mut _ext, mut velocity) in query.iter_mut() {
        let mut state = if transform.translation.y < 10.5 {
            PlayerState::Idle
        } else {
            PlayerState::Jumping
        };

        velocity.angvel = Vec3::ZERO;

        if keyboard_input.any_just_released([KeyCode::KeyA, KeyCode::KeyD]) {
            state = PlayerState::Idle;
            velocity.linvel.x = 0.0;
        }

        if keyboard_input.any_pressed([KeyCode::Space, KeyCode::KeyW]) {
            state = PlayerState::Jumping;
            velocity.linvel.y = 10.0;
        }

        if keyboard_input.just_pressed(KeyCode::KeyS) {
            state = PlayerState::Idle;
            velocity.linvel.y = -10.0;
            transform.rotation = Quat::from_rotation_y(0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            state = PlayerState::Walk;
            transform.rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
            velocity.linvel.x = -PLAYER_BASE_SPEED;
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            state = PlayerState::Walk;
            // point the player to positive x-axis
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            velocity.linvel.x = PLAYER_BASE_SPEED;
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
