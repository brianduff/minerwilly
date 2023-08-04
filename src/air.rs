use crate::color::ColorName;
use bevy::prelude::*;

use crate::text::{Text, TextAttributes};

/// Adds the air supply bar that shows how much time Willy has left until he runs out of air.
pub struct AirPlugin;

impl Plugin for AirPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
  }
}

fn setup(mut commands: Commands) {
  // Red handlebar
  commands.spawn(Text::new(
    "AIR       ",
    (0, 17),
    &TextAttributes::new_bright(ColorName::White, ColorName::Red),
  ));

  // Green handlebar
  commands.spawn(Text::new(
    "                       ",
    (9, 17),
    &TextAttributes::new_bright(ColorName::White, ColorName::Green),
  ));

  // Black separator bar
  commands.spawn(Text::new(
    "                                ",
    (0, 18),
    &TextAttributes::new_bright(ColorName::Black, ColorName::Black),
  ));
}
