//! Plugin providing game data. At the moment this is hardcoded to be based
//! on a manic binary z80 binary, and extracts the data directly from that.

mod data;
mod cavern;

use anyhow::Result;
use bevy::prelude::*;

use crate::handle_errors;

use self::data::GameData;

pub struct GameDataPlugin;

#[derive(Resource, Deref)]
pub struct GameDataResource(GameData);

impl Plugin for GameDataPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.add_systems(PreStartup, load_game_data.pipe(handle_errors));
  }
}

fn load_game_data(
  mut commands: Commands,
) -> Result<()> {
  commands.insert_resource(GameDataResource(GameData::load("assets/ManicMiner.bin")?));

  Ok(())
}
