use bevy::{gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::components::color::Dictionary;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetLoadingState {
    #[default]
    AssetLoading,
    Next,
}

pub mod player_assets {
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
            .add_systems(Startup, load_dictionary);
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "characters/mage/Mage.glb#Scene0")]
    pub model: Handle<Scene>,
    #[asset(path = "characters/mage/Mage.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "characters/mage/Mage.glb")]
    pub gltf: Handle<Gltf>,
}

fn load_dictionary(mut commands: Commands) {
    commands.insert_resource(Dictionary::new());
}
