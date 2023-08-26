

use air::AirPlugin;
use anyhow::Result;
use bevy::prelude::*;
use cavern::CavernPlugin;
use debug::DebugPlugin;
use gamedata::GameDataPlugin;
use guardian::GuardianPlugin;
use item::ItemPlugin;
use lives::LivesPlugin;
use portal::PortalPlugin;
use score::ScorePlugin;
use text::TextPlugin;
use timer::TimerPlugin;
use willy::WillyPlugin;

mod air;
mod actors;
mod bitmap;
mod cavern;
mod color;
mod debug;
mod item;
mod gamedata;
mod guardian;
mod lives;
mod portal;
mod position;
mod score;
mod text;
mod timer;
mod willy;

pub static SCALE: f32 = 2.0;
//static CELLSIZE: f32 = 8.0 * SCALE;
// static TIMER_TICK: f32 = 0.07;
// slow mo static TIMER_TICK: f32 = 0.25;
static BORDER_WIDTH_CHARS: f32 = 4.;

static DISPLAY_SCREEN_WIDTH_CH: f32 = 32.;
static DISPLAY_SCREEN_HEIGHT_CH: f32 = 24.;
static BORDER_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * BORDER_WIDTH_CHARS;
pub static SCREEN_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_WIDTH_CH;
pub static WINDOW_WIDTH_PX: f32 = SCREEN_WIDTH_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static SCREEN_HEIGHT_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_HEIGHT_CH;
static WINDOW_HEIGHT_PX: f32 = SCREEN_HEIGHT_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static PIX_PER_CHAR: f32 = 8.;
static BORDER_MUL: f32 = 2.;

pub fn handle_errors(In(result): In<Result<()>>) {
  if let Err(e) = result {
    eprintln!("Error: {}", e);
  }
}

fn main() -> Result<()> {
  let window = Window {
    title: "Miner Willy".into(),
    resolution: (WINDOW_WIDTH_PX, WINDOW_HEIGHT_PX).into(),
    ..default()
  };

  App::new()
    .add_plugins(
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(window),
          ..default()
        }),
    ) // prevents blurry sprites
    .add_plugins((
      TimerPlugin,
      DebugPlugin,
      GameDataPlugin,
      CavernPlugin,
      TextPlugin,
      ScorePlugin,
      AirPlugin,
      WillyPlugin,
      LivesPlugin,
      GuardianPlugin,
      PortalPlugin,
      ItemPlugin
    ))
    .add_systems(PostStartup, setup)
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .run();

  Ok(())
}

fn setup(mut commands: Commands) {
  commands.spawn(Camera2dBundle::default());
}


/// Despawns all entities with a given component type. Ideally, this would
/// be a system, but I can't seem to ever get it to be reliably ordered with other systems
/// such that it doesn't cause race conditions.
pub fn despawn_all<T: Component>(
  commands: &mut Commands,
  query: Query<Entity, With<T>>,
) {
  for entity in query.iter() {
    commands.entity(entity).despawn();
  }
}


// If value is < lb, then clamp it to ub.
// If value is > ub, then clamp it to lb.
//
// E.g. clamp(x, 0, 3) when x = -1, x becomes 3
//      clamp(x, 0, 3) when x = 4, x becomes 0
pub fn clamp(value: usize, lb: usize, ub: usize) -> usize {
  if value < lb {
    return ub;
  } else if value > ub {
    return lb;
  }

  value
}