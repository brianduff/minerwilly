use bevy::prelude::*;

use crate::{SCALE, SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX};

pub fn new_transform() -> Transform {
  Transform::from_scale(Vec3::splat(SCALE))
}

pub fn at_char_pos(pos: (u8, u8)) -> Transform {
  let (screen_x, screen_y) = char_pos_to_screen(pos);
  new_transform().with_translation(Vec3 { x: screen_x, y: screen_y, z: 0. })
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
