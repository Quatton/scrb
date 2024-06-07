mod plugins;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use plugins::world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldPlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .run();
}
