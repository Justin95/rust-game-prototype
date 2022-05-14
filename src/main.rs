mod terrain_plugin;
mod player;
mod terrain;

use bevy::prelude::*;
use bevy_flycam::NoCameraPlayerPlugin;
use crate::player::PlayerPlugin;
use crate::terrain_plugin::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerrainPlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}
