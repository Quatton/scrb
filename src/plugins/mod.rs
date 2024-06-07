pub mod assets;
pub mod player;
pub mod world;

pub mod prelude {
    pub use super::{assets::CustomAssetPlugin, player::PlayerPlugin, world::WorldPlugin};
}
