use bevy::prelude::*;
use crate::color::SpectrumColorName;

use crate::text::{Text, TextAttributes};

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
    &TextAttributes::new_bright(SpectrumColorName::White, SpectrumColorName::Red)
  ));

  // Green handlebar
  commands.spawn(Text::new(
    "                       ",
    (9, 17),
    &TextAttributes::new_bright(SpectrumColorName::White, SpectrumColorName::Green)
  ));

  // Black separator bar
  commands.spawn(Text::new(
    "                                ",
    (0, 18),
    &TextAttributes::new_bright(SpectrumColorName::Black, SpectrumColorName::Black)
  ));
}
