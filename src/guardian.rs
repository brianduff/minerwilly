use bevy::prelude::*;

use crate::{
  actors::{Actor, Direction, HorizontalMotion, Sprites},
  cavern::Cavern,
  gamedata::{cavern, GameDataResource},
  position::{Layer, Position},
};

pub struct GuardianPlugin;

impl Plugin for GuardianPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, init);
    app.add_systems(Update, spawn_guardians);
  }
}

#[derive(Component, Debug)]
struct Guardian {
  id: u8,
  data: cavern::Guardian,
}

fn init(mut commands: Commands, game_data: Res<GameDataResource>) {}

fn spawn_guardians(
  mut commands: Commands,
  cavern: ResMut<Cavern>,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
) {
  if cavern.is_changed() {
    println!(
      "I would like to spawn guardians for cavern {}",
      cavern.cavern_number
    );
    println!(
      "Guardians: {:?}",
      game_data.caverns[cavern.cavern_number].guardians
    );

    // TODO: despawn old guardians.

    let cavern_data = &game_data.caverns[cavern.cavern_number];

    // Create images for guardian sprites.

    for (id, g) in cavern_data.guardians.iter().enumerate() {
      let images: Vec<_> = cavern_data
        .guardian_bitmaps
        .iter()
        .map(|s| images.add(s.render_with_color(&g.attributes)))
        .collect();

      commands.spawn(Actor::new(
        Guardian {
          id: id as u8,
          data: g.clone(),
        },
        Position::at_char_pos(Layer::Characters, g.start_pos),
        Sprites {
          images,
          current_frame: g.first_animation_frame as usize,
        },
        HorizontalMotion {
          walking: true,
          direction: if g.first_animation_frame <= 3 {
            Direction::Right
          } else {
            Direction::Left
          },
        },
      ));
    }
  }
}
