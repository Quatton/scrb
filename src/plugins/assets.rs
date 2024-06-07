use bevy::{prelude::*, scene::SceneInstance};
use bevy_asset_loader::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetLoadingState {
    #[default]
    AssetLoading,
    Next,
}

pub mod player_assets {
    pub use super::CustomizePlayerMaterial;
    pub use super::PlayerAssets;
}

pub struct CustomAssetPlugin;

impl Plugin for CustomAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetLoadingState>()
            .add_loading_state(
                LoadingState::new(AssetLoadingState::AssetLoading)
                    .continue_to_state(AssetLoadingState::Next)
                    .load_collection::<PlayerAssets>(),
            )
            .add_systems(
                Update,
                customize_scene_materials.run_if(any_with_component::<CustomizePlayerMaterial>),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "characters/Model/characterMedium.glb#Scene0")]
    pub model: Handle<Scene>,
    #[asset(path = "characters/Skins/skaterMaleA.png")]
    pub skater: Handle<Image>,
    #[asset(path = "characters/Animations/idle.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "characters/Animations/jump.glb#Animation0")]
    pub jump: Handle<AnimationClip>,
    #[asset(path = "characters/Animations/run.glb#Animation0")]
    pub run: Handle<AnimationClip>,
}
#[derive(Component)]
pub struct CustomizePlayerMaterial;

#[derive(Resource)]
pub struct PlayerAnimations(Vec<Handle<AnimationClip>>);

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
