use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use bevy_xpbd_3d::prelude::*;
use scrb::plugins::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(RapierBackendSettings {
            require_markers: true,
        })
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins(TextInputPlugin)
        .add_plugins((CustomAssetPlugin, WorldPlugin, PlayerPlugin, MainUiPlugin))
        .run();
}
