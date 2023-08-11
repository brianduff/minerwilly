use air::AirPlugin;
use anyhow::Result;
use bevy::prelude::*;
use cavern::CavernPlugin;
use debug::DebugPlugin;
use gamedata::GameDataPlugin;
use guardian::GuardianPlugin;
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
    ))
    .add_systems(PostStartup, setup)
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .run();

  Ok(())
}

fn setup(mut commands: Commands) {
  commands.spawn(Camera2dBundle::default());
}
