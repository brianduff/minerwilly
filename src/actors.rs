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
  pub current_frame: usize,
}

impl HorizontalMotion {
  pub fn direction(&self) -> Direction {
    if self.current_frame < 4 {
      Direction::Right
    } else {
      Direction::Left
    }
  }

  pub fn step(&mut self, pos: &mut Position) {
    let direction = self.direction();
    pos.step(direction);

    match direction {
      Direction::Left => {
        self.current_frame -= 1;
        clamp(&mut self.current_frame, 4, 7);
      },
      Direction::Right => {
        self.current_frame += 1;
        clamp(&mut self.current_frame, 0, 3);
      }
    };
  }

  pub fn change_direction(&mut self) {
    match self.direction() {
      Direction::Left => {
        self.current_frame -= 4;
      },
      Direction::Right => {
        self.current_frame += 4;
      }
    };
  }
}

// If value is < lb, then clamp it to ub.
// If value is > ub, then clamp it to lb.
//
// E.g. clamp(x, 0, 3) when x = -1, x becomes 3
//      clamp(x, 0, 3) when x = 4, x becomes 0
fn clamp(value: &mut usize, lb: usize, ub: usize) {
  if *value < lb {
    *value = ub;
  } else if *value > ub {
    *value = lb;
  }
}

#[derive(Component)]
pub struct Sprites {
  pub images: Vec<Handle<Image>>,
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
      texture: sprites.images[movement.current_frame].clone(),
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

// Generic system that updates an actor sprite's current image
// texture and transform.
#[allow(clippy::type_complexity)]
pub fn update_actor_sprite<T: Component>(mut query: Query<(
  &Position,
  &HorizontalMotion,
  &Sprites,
  &mut Handle<Image>,
  &mut Transform
), (
  With<T>,
  Or<(Changed<Position>, Changed<Sprites>)>
)>) {

  for (pos, motion, sprites, mut image, mut transform) in query.iter_mut() {
    *transform = pos.into();
    *image = sprites.images[motion.current_frame].clone();
  }
}