pub mod assets;
pub mod player;
pub mod ui;
pub mod world;

pub mod prelude {
    pub use super::{
        assets::CustomAssetPlugin, player::PlayerPlugin, ui::MainUiPlugin, world::WorldPlugin,
    };
}
