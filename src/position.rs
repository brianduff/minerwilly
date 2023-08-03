use bevy::prelude::*;

use crate::{SCALE, SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

pub fn new_transform() -> Transform {
  Transform::from_scale(Vec3::splat(SCALE))
}

pub fn at_char_pos(layer: Layer, pos: (u8, u8)) -> Transform {
  let (screen_x, screen_y) = char_pos_to_screen(pos);
  let z = (layer as u32) as f32;
  new_transform().with_translation(Vec3 {
    x: screen_x,
    y: screen_y,
    z,
  })
}

// Converts a character position on screen to the top left screen
// coordinate that contains that character.
pub fn char_pos_to_screen((x, y): (u8, u8)) -> (f32, f32) {
  let x: f32 = x.into();
  let y: f32 = y.into();
  let pos_x = 0.0 - (SCREEN_WIDTH_PX / 2.) + (8. * x * SCALE);
  let pos_y = 0.0 + (SCREEN_HEIGHT_PX / 2.) - (8. * y * SCALE);

  (pos_x, pos_y)
}

/// Returns true if the given pixel coordinates are aligned to a cell
/// boundary.
pub fn is_cell_aligned(pos: (f32, f32)) -> bool {
  let (_, _, offx, offy) = to_cell(pos);
  offx == 0.0 && offy == 0.0
}

/// Given a pixel coordinate, return the equivalent character coordinate
/// and the internal offset from that character coordinate within the
/// cell. Note that the offset is scaled (multiplied by SCALE).
pub fn to_cell((px, py): (f32, f32)) -> (u8, u8, f32, f32) {
  // Make 0 <= adjy < 192 * SCALE
  let adjy = -(py - (SCREEN_HEIGHT_PX / 2.));

  // Make 0 <= cy < 24
  let cy = (adjy / (8. * SCALE)) as u8;
  // Get the offset 0.0 <= offy <= 8 * SCALE
  let offy = adjy % (8. * SCALE);

  // Make 0 <= adjx < 192 * SCALE
  println!("X pixel pos = {}", px);
  let adjx = px + (SCREEN_HEIGHT_PX / 2.);
  println!("adjx = {} + {} = {}", px, (SCREEN_HEIGHT_PX / 2.), adjx);

  // Make 0 <= cx < 24
  let cx: u8 = (adjx / (8. * SCALE)) as u8;
  println!("cx = {}", cx);
  let offx = adjx % (8. * SCALE);

  (cx, cy, offx, offy)
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

/// The position of an actor: either Willy or a Guardian.
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
pub struct ActorPosition {
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


impl ActorPosition {
  /// Creates a new position at the top let of the given char pos.
  pub fn at_char_pos((x, y): (u8, u8)) -> Self {
    let zx_pixel_pos = (x as f32 * 8.0, y as f32 * 8.0);

    ActorPosition {
      layer: Layer::Characters,
      zx_pixel_pos
    }
  }

  // Take a single step in the given direction. This moves the
  // pixel coordinate by 2 in that direction (each animation frame,
  // willy or a guardian's sprite moves by 2 pixels)
  pub fn step(&mut self, direction: Direction) {
    let (x, y) = self.zx_pixel_pos;
    self.zx_pixel_pos = (x + match direction {
      Direction::Left => -2.,
      Direction::Right => 2.
    }, y)
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
impl From<&ActorPosition> for Transform {
  fn from(value: &ActorPosition) -> Self {
    let (_, y) = value.pixel_pos();
    let z = (value.layer as u32) as f32;

    // Snap x back to the start of the current char cell
    let (char_x, _) = value.char_pos();
    let x = SCALE * (char_x as f32 * 8. - 128.);

    new_transform().with_translation(Vec3 { x, y, z })
  }
}
