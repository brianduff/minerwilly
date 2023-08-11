use bevy::prelude::*;

use crate::{
  actors::{Actor, Direction, HorizontalMotion, Sprites, update_actor_sprite},
  cavern::Cavern,
  gamedata::{cavern, GameDataResource},
  position::{Layer, Position}, timer::GameTimer,
};

pub struct GuardianPlugin;

impl Plugin for GuardianPlugin {
  fn build(&self, app: &mut App) {
    // app.add_systems(Startup, init);
    app.add_systems(Update, (spawn_guardians, move_guardians, change_direction, update_actor_sprite::<Guardian>).chain());
  }
}

#[derive(Component, Debug)]
struct Guardian {
  id: u8,
  data: cavern::Guardian,
}

// fn init(mut commands: Commands, game_data: Res<GameDataResource>) {}

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
        },
        HorizontalMotion {
          walking: true,
          current_frame: g.first_animation_frame as usize
        },
      ));
    }
  }
}


fn move_guardians(
  timer: Res<GameTimer>,
  mut query: Query<(
    &mut HorizontalMotion,
    &mut Position
  ),
    With<Guardian>
  >) {

  if timer.just_finished() {
    for (mut motion, mut pos) in query.iter_mut() {
      motion.step(&mut pos);
    }
  }
}

/// Changes the Guardian's direction if it has reached the end of its
/// path.
#[allow(clippy::type_complexity)]
fn change_direction(mut query: Query<(
  &mut HorizontalMotion,
  &mut Position,
  &Guardian
),
(
  With<Guardian>,
  Changed<Position>
)
>) {
  for (mut motion, mut position, guardian) in query.iter_mut() {
    let (x, _) = position.char_pos();
    if (matches!(motion.direction(), Direction::Right) &&  x > guardian.data.right_bound) ||
      (matches!(motion.direction(), Direction::Left) && x < guardian.data.left_bound) {
        motion.change_direction();
        motion.step(&mut position);
      }
  }
}