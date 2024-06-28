use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_mod_reqwest::ReqwestPlugin;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use bevy_web_asset::WebAssetPlugin;
use scrb::plugins::prelude::*;

fn main() {
    dotenvy::dotenv().ok();
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            WebAssetPlugin,
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            ReqwestPlugin::default(),
        ))
        .insert_resource(RapierBackendSettings {
            require_markers: true,
        })
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins(TextInputPlugin)
        .add_plugins((CustomAssetPlugin, WorldPlugin, PlayerPlugin, MainUiPlugin))
        .run();
}
