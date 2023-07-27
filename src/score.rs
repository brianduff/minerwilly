use crate::{
  handle_errors,
  text::{Text, TextAttributes},
};
use anyhow::Result;
use bevy::prelude::*;
use crate::color::SpectrumColorName;

pub struct ScorePlugin;

#[derive(Resource)]
pub struct Score {
  pub score: u16,
  pub high_score: u16,
}

impl Score {
  pub fn add(&mut self, amount: u16) {
    self.score += amount;
    println!("Set score to {}", self.score);
    // In the original game, this doesn't update until the game is over
//    self.high_score = Ord::max(self.score, self.high_score);
  }
}

#[derive(Component)]
struct ScoreType;

#[derive(Component)]
struct HighScoreType;

impl Plugin for ScorePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, init_score);
    app.add_systems(Update, (check_debug_keyboard, update_score.pipe(handle_errors), update_high_score.pipe(handle_errors)));
  }
}

fn init_score(mut commands: Commands) {
  commands.insert_resource(Score {
    score: 0,
    high_score: 0,
  });

  let attr = TextAttributes::new(SpectrumColorName::Yellow, SpectrumColorName::Black);
  commands.spawn(Text::new("High Score ", (0, 19), &attr));
  commands.spawn(Text::new("   Score ", (17, 19), &attr));

  commands.spawn((ScoreType, Text::new(&pad(0), (26, 19), &attr)));
  commands.spawn((HighScoreType, Text::new(&pad(0), (11, 19), &attr)));
}

fn update_score(
  score: Res<Score>,
  mut query: Query<&mut Text, With<ScoreType>>,
) -> Result<()> {
  if score.is_changed() {
    query.get_single_mut()?.value = pad(score.score);
  }

  Ok(())
}

fn update_high_score(
  score: Res<Score>,
  mut query: Query<&mut Text, With<HighScoreType>>,
) -> Result<()> {
  if score.is_changed() {
    query.get_single_mut()?.value = pad(score.high_score);
  }

  Ok(())
}

fn pad(score: u16) -> String {
  format!("{:0>6}", score)
}

fn check_debug_keyboard(keys: Res<Input<KeyCode>>, mut score: ResMut<Score>) {
  if keys.just_released(KeyCode::Return)  {
    score.add(100);
  }
}