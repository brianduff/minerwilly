use bevy::{prelude::*, sprite::Anchor};
use minerdata::color::SpectrumColor;

use crate::{gamedata::{GameDataResource, CavernTexture}, position::at_char_pos};

pub struct CavernPlugin;

#[derive(Resource, Debug)]
struct Cavern {
    cavern_number: usize
}

#[derive (Component, Debug)]
struct CavernTile {
    x: u8,
    y: u8,
}


impl Plugin for CavernPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
      app.add_systems(Startup, setup);
      app.add_systems(Update, (spawn_cavern, check_debug_keyboard));
    }
}

fn setup(mut commands: Commands) {
  commands.insert_resource(Cavern{ cavern_number: 0 });
}

/// Converts the ink of the given SpectrumColor into a bevy Color
fn ink_to_color(spectrum_color: &SpectrumColor) -> Color {
  let spectrum_rgba = spectrum_color.ink_rgba();
  Color::Rgba {
    red: spectrum_rgba[0] as f32 / 255.,
    green: spectrum_rgba[1] as f32 / 255.,
    blue: spectrum_rgba[2] as f32 / 255.,
    alpha: spectrum_rgba[3] as f32  / 255. }
}


fn spawn_cavern(
  mut commands: Commands,
  cavern: Res<Cavern>,
  game_data: Res<GameDataResource>,
  textures: Res<CavernTexture>,
  mut clear_color: ResMut<ClearColor>,
  query: Query<Entity, With<CavernTile>>,
) {
  if cavern.is_changed() {
      // Despawn any existing cavern tiles.
      query.for_each(|entity| {
          commands.entity(entity).despawn();
      });

      let current_cavern = cavern.cavern_number;
      let cavern = &game_data.caverns[current_cavern];
      println!("Current cavern is {:?}", current_cavern);
      let border_color = ink_to_color(&cavern.border_color);
      clear_color.0 = border_color;

      for y in 0..16 {
          for x in 0..32 {
              let sprite_index = cavern.get_bg_sprite_index(x.into(), y.into());
              let tile = CavernTile { x, y };
              if let Some(sprite_index) = sprite_index {
                  commands.spawn((
                      tile,
                      SpriteSheetBundle {
                          texture_atlas: textures.clone(),
                          sprite: TextureAtlasSprite {
                              index: (current_cavern * 8) + sprite_index,
                              anchor: Anchor::TopLeft,
                              ..default() },
                          transform: at_char_pos((x, y)),
                          ..default()
                      },
                  ));
              }
          }
      }
  }
}

fn check_debug_keyboard(keys: Res<Input<KeyCode>>, mut cavern: ResMut<Cavern>) {
  if keys.just_released(KeyCode::BracketRight) && cavern.cavern_number < 19 {
    cavern.cavern_number += 1;
  } else if keys.just_released(KeyCode::BracketLeft) && cavern.cavern_number > 0 {
    cavern.cavern_number -= 1;
  }
}