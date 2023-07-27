use bevy::{prelude::*, sprite::Anchor};

use crate::{gamedata::GameDataResource, color::{SpectrumColor, SpectrumColorName}, SCALE, position::{at_char_pos, Layer}};

pub struct WillyPlugin;

#[derive(Component)]
struct WillySprites {
  images: Vec<Handle<Image>>,
  current_sprite: usize,
  current_frame: usize,
}

impl Plugin for WillyPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
  }
}

fn setup(mut commands: Commands, game_data: Res<GameDataResource>, mut images: ResMut<Assets<Image>>) {
  println!("Setup for willy");
  let willy_color = SpectrumColor::new_transparent_bg(SpectrumColorName::White, false);

  let images: Vec<_> = game_data.willy_sprites.iter().map(|s| images.add(s.render_with_color(&willy_color))).collect();
  let initial_texture = images.first().unwrap().clone();

  let sprite_images = WillySprites {
    images,
    current_sprite: 0,
    current_frame: 0
  };

  // Spawn Willy
  commands.spawn((
    sprite_images,
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: initial_texture,
      transform: at_char_pos(Layer::Characters, (2, 13)),
      ..Default::default()
    }
  ));
}
