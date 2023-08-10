use bevy::{prelude::*, sprite::Anchor};

use crate::position::Position;

/// General stuff that applies to either willy
/// or guardians (i.e. actors)

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
  Left,
  Right,
}


#[derive(Component)]
pub struct HorizontalMotion {
  pub walking: bool,
  pub direction: Direction
}

#[derive(Component)]
pub struct Sprites {
  pub images: Vec<Handle<Image>>,
  pub current_frame: usize
}

#[derive(Bundle)]
pub struct Actor<T: Component> {
  data: T,
  position: Position,
  sprites: Sprites,
  movement: HorizontalMotion,
  sprite_bundle: SpriteBundle,
}

impl<T: Component> Actor<T> {
  pub fn new(data: T, position: Position, sprites: Sprites, movement: HorizontalMotion) -> Self {
    let sprite_bundle = SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      // Change to get the actual first image
      texture: sprites.images[sprites.current_frame].clone(),
      transform: (&position).into(),
      ..Default::default()
    };

    Self {
      data,
      position,
      sprites,
      movement,
      sprite_bundle
    }
  }
}