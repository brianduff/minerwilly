use bevy::prelude::*;

use crate::SCALE;

fn new_transform() -> Transform {
  Transform::from_scale(Vec3::splat(SCALE))
}


/// The layer that a sprite is rendered at. This is translated into its
/// z-coordinate.
#[derive(Debug, Clone, Copy)]
pub enum Layer {
  //Background = 0,
  Tiles = 1,
  Characters = 2,
  // For HUD etc.
  Debug = 3,
}

/// Represents a position on screen.
/// We provide access to both a character position and an absolute
/// pixel x, y position (unscaled, and in a simplified version of
/// the ZX Spectrum's coordinate system, where x ranges from 0-255 and
/// y ranges from 0-191, but pixels are laid out in the modern way).
/// In both cases, the position
/// refers to the top leftmost character cell or pixel of the actor.
///
/// The most common way to convert to bevy's coordinate system (and
/// apply scaling etc) is to convert the position into a Transform.
#[derive(Component, Debug)]
pub struct Position {
  layer: Layer,
  // The canonical position is the zx spectrum pixel pos, which can
  // always be snapped to the character cell that it lies within.
  zx_pixel_pos: (f32, f32),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
  Left,
  Right,
}

impl Position {
  /// Creates a new position at the top let of the given char pos.
  pub fn at_char_pos(layer: Layer, (x, y): (u8, u8)) -> Self {
    let zx_pixel_pos = (x as f32 * 8.0, y as f32 * 8.0);

    Position {
      layer,
      zx_pixel_pos,
    }
  }

  /// Returns true if Willy's head is aligned with the top left of a character
  /// cell boundary (basically, when his y coordinate is divisible by 8)
  pub fn is_vertically_cell_aligned(&self) -> bool {
    self.zx_pixel_pos.1 % 8. == 0.
  }

  // Take a single step in the given direction. This moves the
  // pixel coordinate by 2 in that direction (each animation frame,
  // willy or a guardian's sprite moves by 2 pixels)
  pub fn step(&mut self, direction: Direction) {
    let (x, y) = self.zx_pixel_pos;
    self.zx_pixel_pos = (
      x + match direction {
        Direction::Left => -2.,
        Direction::Right => 2.,
      },
      y,
    );
  }

  // Jump (or fall if distance is negative) the given distance in pixels.
  pub fn jump(&mut self, distance: f32) {
    let (x, y) = self.zx_pixel_pos;
    self.zx_pixel_pos = (x, y - distance);
  }

  /// Return this position as a scaled bevy coordinate system pixel
  /// position.
  /// -(SCALE * 128.0) <= x < (SCALE * 128.0)
  /// -(SCALE * 96.0) <= y < (SCALE * 96.0)
  pub fn pixel_pos(&self) -> (f32, f32) {
    let (zx_x, zx_y) = self.zx_pixel_pos;
    (SCALE * (zx_x - 128.), SCALE * (96. - zx_y))
  }

  /// Return the character position that contains the pixel.
  /// 0 <= x < 32
  /// 0 <= y < 24
  pub fn char_pos(&self) -> (u8, u8) {
    let (zx_x, zx_y) = self.zx_pixel_pos;

    ((zx_x / 8.) as u8, (zx_y / 8.) as u8)
  }
}

/// Return a transform for this actor position. Note that the x
/// coordinate of this transform *does not* match the value returned
/// by pixel_pos, because sprites stay in the same cell until they
/// reach the end of their 4 movement sprite animations.
impl From<&Position> for Transform {
  fn from(value: &Position) -> Self {
    let (_, y) = value.pixel_pos();
    let z = (value.layer as u32) as f32;

    // Snap x back to the start of the current char cell
    let (char_x, _) = value.char_pos();
    let x = SCALE * (char_x as f32 * 8. - 128.);

    new_transform().with_translation(Vec3 { x, y, z })
  }
}

impl From<Position> for Transform {
  fn from(value: Position) -> Self {
    (&value).into()
  }
}
