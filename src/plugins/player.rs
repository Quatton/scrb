use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, scene::SceneInstance,
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;

const PLAYER_RADIUS: f32 = 1.0;
const PLAYER_HEIGHT: f32 = 2.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetLoadingState {
    #[default]
    AssetLoading,
    Next,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetLoadingState>()
            .add_loading_state(
                LoadingState::new(AssetLoadingState::AssetLoading)
                    .continue_to_state(AssetLoadingState::Next)
                    .load_collection::<PlayerAssets>(),
            )
            .add_systems(OnEnter(AssetLoadingState::Next), setup_player)
            .add_systems(
                Update,
                customize_scene_materials.run_if(any_with_component::<CustomizePlayerMaterial>),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "characters/Model/characterMedium.glb#Scene0")]
    model: Handle<Scene>,
    #[asset(path = "characters/Skins/skaterMaleA.png")]
    skater: Handle<Image>,
    #[asset(path = "characters/Animations/idle.glb#Animation0")]
    idle: Handle<AnimationClip>,
    #[asset(path = "characters/Animations/jump.glb#Animation0")]
    jump: Handle<AnimationClip>,
    #[asset(path = "characters/Animations/run.glb#Animation0")]
    run: Handle<AnimationClip>,
}

#[derive(Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct CustomizePlayerMaterial;

#[derive(Resource)]
pub struct PlayerAnimations(Vec<Handle<AnimationClip>>);

fn setup_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands.spawn((
        SceneBundle {
            scene: player_assets.model.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
            ..default()
        },
        CustomizePlayerMaterial,
        Collider::capsule_y(PLAYER_HEIGHT / 2.0, PLAYER_RADIUS),
        RigidBody::Dynamic,
    ));
}

pub fn customize_scene_materials(
    unloaded_instances: Query<(Entity, &SceneInstance), With<CustomizePlayerMaterial>>,
    handles: Query<(Entity, &Handle<StandardMaterial>)>,
    mut pbr_materials: ResMut<Assets<StandardMaterial>>,
    player_assets: Res<PlayerAssets>,
    scene_manager: Res<SceneSpawner>,
    mut cmds: Commands,
) {
    for (entity, instance) in unloaded_instances.iter() {
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).remove::<CustomizePlayerMaterial>();
        }
        // Iterate over all entities in scene (once it's loaded)
        let handles = handles.iter_many(scene_manager.iter_instance_entities(**instance));
        for (_entity, material_handle) in handles {
            let Some(material) = pbr_materials.get_mut(material_handle) else {
                continue;
            };
            material.base_color_texture = Some(player_assets.skater.clone());
        }
    }
}
