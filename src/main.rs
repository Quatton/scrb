mod plugins;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use plugins::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        // .add_plugins(UiGeneralPlugin)
        // .add_plugins(UiPlugin::<MainUi>::new())
        // .add_plugins(DefaultPickingPlugins)
        .add_plugins(TextInputPlugin)
        .add_plugins((CustomAssetPlugin, WorldPlugin, PlayerPlugin, MainUiPlugin))
        .run();
}
