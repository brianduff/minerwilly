use crate::bitmap::Bitmap;
use crate::color::ColorName;
use crate::gamedata::cavern::CavernTileType;
use crate::position::{Layer, Position};
use crate::timer::GameTimer;
use crate::willy::Willy;
use crate::{
  gamedata::GameDataResource,
  handle_errors,
  text::{Text, TextAttributes},
};
use anyhow::Result;
use bevy::{prelude::*, sprite::Anchor};

/// Adds drawing the current cavern
pub struct CavernPlugin;

#[derive(Resource, Debug)]
pub struct Cavern {
  pub cavern_number: usize,
}

#[derive(Component, Debug)]
struct CavernTile {
  pos: (u8, u8)
}

impl CavernTile {
  fn new(pos: (u8, u8)) -> Self {
    Self {
      pos
    }
  }
}


/// The current state of the cavern. This can be used by other plugins to query information
/// about the tiles surrounding Willy.
#[derive(Resource, Debug)]
pub struct CavernState {
  tile_types: [[CavernTileType; 16]; 32],
  crumble_level: [[u8; 16]; 32]
}

impl CavernState {
  pub fn get_tile_type(&self, (x, y): (u8, u8)) -> CavernTileType {
    self.tile_types[x as usize][y as usize]
  }

  pub fn get_crumble_level(&self, (x, y): (u8, u8)) -> u8 {
    self.crumble_level[x as usize][y as usize]
  }
}

#[derive(Component)]
struct CavernName;

impl Plugin for CavernPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.add_systems(Startup, setup);
    app.add_systems(
      Update,
      (
        update_border,
        update_cavern_name,
        spawn_cavern.pipe(handle_errors),
        check_debug_keyboard,
        update_tile_state,
        update_crumble,
        update_tile_sprites
      ),
    );
  }
}

#[derive(Resource)]
struct CrumblingTileImages {
  images: Vec<Handle<Image>>
}

impl CrumblingTileImages {
  fn new() -> Self {
    Self {
      images: Vec::with_capacity(7)
    }
  }

  fn update(&mut self, mut image_assets: ResMut<Assets<Image>>, base_bitmap: &Bitmap) {
    self.images.clear();
    let mut new_bitmap = base_bitmap.shift_down();
    for _ in 0..8 {
      self.images.push(image_assets.add(new_bitmap.render()));
      new_bitmap = new_bitmap.shift_down();
    }
  }
}

fn setup(mut commands: Commands) {
  commands.insert_resource(Cavern { cavern_number: 0 });
  commands.insert_resource(CavernState {
    tile_types: [[CavernTileType::Background; 16]; 32],
    crumble_level: [[7; 16]; 32]
  });
  commands.insert_resource(CrumblingTileImages::new());

  // Spawn the cavern name
  commands.spawn((
    CavernName,
    Text::new(
      "                     ",
      (0, 16),
      &TextAttributes::new(ColorName::Black, ColorName::Yellow),
    ),
  ));
}

fn update_border(
  game_data: Res<GameDataResource>,
  cavern: Res<Cavern>,
  mut clear_color: ResMut<ClearColor>,
) {
  if cavern.is_changed() {
    let cavern = &game_data.caverns[cavern.cavern_number];
    let border_color = cavern.border_color.ink_color();
    clear_color.0 = border_color;
  }
}

fn update_cavern_name(
  game_data: Res<GameDataResource>,
  cavern: Res<Cavern>,
  mut query: Query<&mut Text, With<CavernName>>,
) {
  if cavern.is_changed() {
    let name = &game_data.caverns[cavern.cavern_number].name;
    query.get_single_mut().unwrap().value = name.to_owned();
  }
}

fn spawn_cavern(
  mut commands: Commands,
  cavern: Res<Cavern>,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
  mut crumbling_tiles: ResMut<CrumblingTileImages>,
  tile_query: Query<Entity, With<CavernTile>>,
) -> Result<()> {
  if cavern.is_changed() {
    // Despawn any existing cavern tiles.
    tile_query.for_each(|entity| {
      commands.entity(entity).despawn();
    });

    let current_cavern = cavern.cavern_number;
    let cavern = &game_data.caverns[current_cavern];

    // Create images for the tiles in this cavern so we can spawn sprites for them
    let mut image_handles = Vec::new();
    for tile in cavern.tile_bitmaps.iter() {
      image_handles.push(images.add(tile.render()));
    }

    let crumbling_bitmap = &cavern.tile_bitmaps[2]; // TODO: don't hardcode this
    crumbling_tiles.update(images, crumbling_bitmap);


    for y in 0..16 {
      for x in 0..32 {
        let sprite_index = cavern.get_bg_sprite_index((x, y));
        if let Some(sprite_index) = sprite_index {
          let texture = &image_handles[sprite_index];
          commands.spawn((
            CavernTile { pos: (x, y) },
            SpriteBundle {
              texture: texture.clone(),
              sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..default()
              },
              transform: Position::at_char_pos(Layer::Tiles, (x, y)).into(),
              ..default()
            },
          ));
        }
      }
    }
  }
  Ok(())
}

fn check_debug_keyboard(keys: Res<Input<KeyCode>>, mut cavern: ResMut<Cavern>) {
  if keys.just_released(KeyCode::BracketRight) && cavern.cavern_number < 19 {
    cavern.cavern_number += 1;
  } else if keys.just_released(KeyCode::BracketLeft) && cavern.cavern_number > 0 {
    cavern.cavern_number -= 1;
  }
}

fn update_tile_state(mut cavern_state: ResMut<CavernState>,
  cavern: Res<Cavern>, game_data: Res<GameDataResource>,
) {
  if cavern.is_changed() {
    let current_cavern = cavern.cavern_number;
    let cavern = &game_data.caverns[current_cavern];

    for y in 0..16 {
      for x in 0..32 {
        cavern_state.tile_types[x][y] = cavern
            .get_bg_sprite_index((x as u8, y as u8))
            .unwrap_or(0).into();
        cavern_state.crumble_level[x][y] = 7;
      }
    }
  }
}

// If willy is standing on a crumbling tile, decrement its crumble
// animation, update its sprite. If it exceeds the last animation
// frame, change the tile type to Background so that Willy will fall.
fn update_crumble(timer: Res<GameTimer>, mut cavern_state: ResMut<CavernState>,
  query: Query<&Position, With<Willy>>,
) {
  if timer.just_finished() {
    for position in query.iter() {
      let (cx, mut cy) = position.char_pos();
      cy += 2;

      if matches!(cavern_state.get_tile_type((cx, cy)), CavernTileType::CrumblingFloor) {
        let level = cavern_state.crumble_level[cx as usize][cy as usize];
        if level == 0 {
          cavern_state.tile_types[cx as usize][cy as usize] = CavernTileType::Background;
        } else {
          cavern_state.crumble_level[cx as usize][cy as usize] -= 1;
        }
      }
    }
  }
}

fn update_tile_sprites(crumbling_images: Res<CrumblingTileImages>,
    cavern_state: Res<CavernState>, mut query: Query<(&CavernTile, &mut Handle<Image>)>) {
  if cavern_state.is_changed() {
    for (tile, mut image) in query.iter_mut() {
      if matches!(cavern_state.get_tile_type(tile.pos), CavernTileType::CrumblingFloor) {
        let crumble_level = cavern_state.get_crumble_level(tile.pos);
        if crumble_level < 7 {
          *image = crumbling_images.images[(7 - crumble_level) as usize].clone();
        }
      }
    }
  }
}
