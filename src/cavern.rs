use crate::color::ColorName;
use crate::position::{Layer, Position};
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
struct CavernTile;

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
      ),
    );
  }
}

fn setup(mut commands: Commands) {
  commands.insert_resource(Cavern { cavern_number: 0 });

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

    println!("Current cavern is {:?}", current_cavern);

    for y in 0..16 {
      for x in 0..32 {
        let sprite_index = cavern.get_bg_sprite_index((x, y));
        if let Some(sprite_index) = sprite_index {
          let texture = &image_handles[sprite_index];
          commands.spawn((
            CavernTile,
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
