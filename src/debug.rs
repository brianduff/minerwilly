use bevy::prelude::*;

use crate::{
  color::SpectrumColorName,
  position::Layer,
  text::{Text, TextAttributes},
};

/// Provides the debug plugin which inserts helpful stuff like a HUD to make it easier to
/// debug the state of the running app. The debug plugin is pluggable itself - plugins
/// contribute text to it by obtaining the DebugText resource and mutating it.s

pub struct DebugPlugin;

#[derive(Resource)]
pub struct DebugText {
  pub line1: String,
  pub line2: String,
  pub line3: String
}

impl DebugText {
  fn get(&self, line: usize) -> &str {
    match line {
      0 => &self.line1,
      1 => &self.line2,
      2 => &self.line3,
      _ => ""
    }
  }
}

#[derive(Component, Deref)]
struct DebugDisplayText(usize);

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, init);
    app.add_systems(Update, update);
  }
}

fn init(mut commands: Commands) {
  let color = TextAttributes::new(SpectrumColorName::Green, SpectrumColorName::Black);

  commands.insert_resource(DebugText {
    line1: " ".to_owned(),
    line2: " ".to_owned(),
    line3: " ".to_owned()
  });

  commands.spawn((
    DebugDisplayText(0),
    Text::new_with_layer(" ", (4, 20), &color, Layer::Debug),
  ));
  commands.spawn((
    DebugDisplayText(1),
    Text::new_with_layer(" ", (4, 21), &color, Layer::Debug),
  ));
  commands.spawn((
    DebugDisplayText(2),
    Text::new_with_layer(" ", (4, 22), &color, Layer::Debug),
  ));
}

fn update(debug_text: Res<DebugText>,
    mut query: Query<(&mut Text, &mut DebugDisplayText), With<DebugDisplayText>>) {

  if debug_text.is_changed() {
    for (mut text, display) in query.iter_mut() {
      text.value = debug_text.get(**display).to_owned()
    }
  }
}
