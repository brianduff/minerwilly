use air::AirPlugin;
use bevy::prelude::*;
use cavern::CavernPlugin;
use gamedata::GameDataPlugin;
use anyhow::Result;
use score::ScorePlugin;
use text::TextPlugin;
use willy::WillyPlugin;

mod air;
mod bitmap;
mod cavern;
mod color;
mod gamedata;
mod position;
mod score;
mod text;
mod willy;

pub static SCALE: f32 = 2.0;
static CELLSIZE: f32 = 8.0 * SCALE;
static TIMER_TICK: f32 = 0.075;
static BORDER_WIDTH_CHARS: f32 = 4.;

static DISPLAY_SCREEN_WIDTH_CH: f32 = 32.;
static DISPLAY_SCREEN_HEIGHT_CH: f32 = 24.;
static BORDER_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * BORDER_WIDTH_CHARS;
pub static SCREEN_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_WIDTH_CH;
pub static WINDOW_WIDTH_PX : f32  = SCREEN_WIDTH_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static SCREEN_HEIGHT_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_HEIGHT_CH;
static WINDOW_HEIGHT_PX : f32 = SCREEN_HEIGHT_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static PIX_PER_CHAR: f32 = 8.;
static BORDER_MUL: f32 = 2.;


pub fn handle_errors(In(result): In<Result<()>>) {
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}

fn main() -> Result<()>  {
    let window = Window {
        title: "Miner Willy".into(),
        resolution: (WINDOW_WIDTH_PX, WINDOW_HEIGHT_PX).into(),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.
            set(ImagePlugin::default_nearest())
            .set(WindowPlugin { primary_window: Some(window), ..default() })) // prevents blurry sprites
        .add_plugins((GameDataPlugin, CavernPlugin, TextPlugin, ScorePlugin, AirPlugin, WillyPlugin))
        .add_systems(PostStartup, setup)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
)  {
    commands.spawn(Camera2dBundle::default());
}
