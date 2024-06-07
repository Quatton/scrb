mod plugins;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use plugins::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((CustomAssetPlugin, WorldPlugin, PlayerPlugin))
        .run();
}
