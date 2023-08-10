use bevy::prelude::*;

use crate::{
  color::ColorName,
  position::{Layer, Position, vec2},
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
  pub line3: String,
}

impl DebugText {
  fn get(&self, line: usize) -> &str {
    match line {
      0 => &self.line1,
      1 => &self.line2,
      2 => &self.line3,
      _ => "",
    }
  }
}
#[derive(Event, Deref)]
pub struct DebugStateToggled(bool);

#[derive(Resource, Debug, Default)]
struct DebugState {
  show_text: bool,
  show_grid: bool,
}

impl DebugState {
  fn new() -> Self {
    let mut state = DebugState::default();
    state.set_all_on(true);
    state
  }

  fn set_all_on(&mut self, on: bool) -> &mut Self {
    self.show_text = on;
    self.show_grid = on;
    self
  }

  fn toggle(&mut self) -> bool {
    self.set_all_on(!self.show_text);
    self.show_text
  }
}


#[derive(Component, Deref)]
struct DebugDisplayText(usize);

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, init);
    app.add_systems(Update, (check_keyboard, update, draw_grid));
    app.add_event::<DebugStateToggled>();
  }
}

fn init(mut commands: Commands, mut gizmo_config: ResMut<GizmoConfig>) {

  // Make debug lines draw above everything else in the scene.
  gizmo_config.depth_bias = -1.;

  let color = TextAttributes::new(ColorName::Green, ColorName::Black);

  commands.insert_resource(DebugText {
    line1: " ".to_owned(),
    line2: " ".to_owned(),
    line3: " ".to_owned(),
  });
  commands.insert_resource(DebugState::new());

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

fn update(
  debug_text: Res<DebugText>,
  debug_state: Res<DebugState>,
  mut query: Query<(&mut Text, &mut DebugDisplayText), With<DebugDisplayText>>,
) {
  if debug_text.is_changed() || debug_state.is_changed() {
    if debug_state.show_text {
      for (mut text, display) in query.iter_mut() {
        text.value = debug_text.get(**display).to_owned()
      }
    } else {
      for (mut text, _) in query.iter_mut() {
        text.value = " ".to_owned();
      }
    }
  }
}

fn draw_grid(mut gizmos: Gizmos, debug_state: Res<DebugState>) {
  if !debug_state.show_grid {
    return;
  }
  let mut pos = Position::at_char_pos(Layer::Debug,(0, 0));

  let color = Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.05 };

  for x in 1..32 {
    let start = vec2(pos.set_char_pos((x, 0)).pixel_pos());
    let end = vec2(pos.set_char_pos((x, 24)).pixel_pos());

    gizmos.line_2d(start, end, color);
  }

  for y in 1..24 {
    let start = vec2(pos.set_char_pos((0, y)).pixel_pos());
    let end = vec2(pos.set_char_pos((32, y)).pixel_pos());

    gizmos.line_2d(start, end, color);

  }
}

fn check_keyboard(mut debug_state: ResMut<DebugState>, keys: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<DebugStateToggled>) {
  if keys.just_pressed(KeyCode::D) {
    let on = debug_state.toggle();
    event_writer.send(DebugStateToggled(on));
  }
}