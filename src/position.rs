use bevy::prelude::*;

use crate::{SCALE, SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX};

pub fn new_transform() -> Transform {
  Transform::from_scale(Vec3::splat(SCALE))
}

pub fn at_char_pos(layer: Layer, pos: (u8, u8)) -> Transform {
  let (screen_x, screen_y) = char_pos_to_screen(pos);
  let z = (layer as u32) as f32;
  new_transform().with_translation(Vec3 { x: screen_x, y: screen_y, z })
}

// Converts a character position on screen to the top left screen
// coordinate that contains that character.
pub fn char_pos_to_screen((x, y): (u8, u8)) -> (f32, f32) {
  let x : f32 = x.into();
  let y : f32 = y.into();
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