//! Plugin providing game data. At the moment this is hardcoded to be based
//! on a manic binary z80 binary, and extracts the data directly from that.

mod data;
mod cavern;
mod sprite;

use anyhow::Result;
use bevy::{
  prelude::*,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::handle_errors;

use self::data::GameData;

pub struct GameDataPlugin;

#[derive(Resource, Deref)]
pub struct CavernTexture(Handle<TextureAtlas>);

#[derive(Resource, Deref)]
pub struct GameDataResource(GameData);

impl Plugin for GameDataPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.add_systems(PreStartup, load_game_data.pipe(handle_errors));

//    app.register_type::<GameDataResource>();
  }
}

fn load_game_data(
  mut commands: Commands,
  mut images: ResMut<Assets<Image>>,
  mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) -> Result<()> {
  println!("Loading game data");
  let game_data = GameData::load("assets/ManicMiner.bin")?;
  let tiles_image = images.add(Image::new(
    Extent3d {
      width: 128,
      height: 88,
      depth_or_array_layers: 1,
    },
    TextureDimension::D2,
    game_data.cavern_tiles_rgba()?,
    TextureFormat::Rgba8Unorm,
  ));
  let tile_textures = texture_atlases.add(TextureAtlas::from_grid(
    tiles_image,
    Vec2::new(8.0, 8.0),
    16,
    10,
    None,
    None,
  ));


  commands.insert_resource(CavernTexture(tile_textures));
  commands.insert_resource(GameDataResource(game_data));

  Ok(())
}
