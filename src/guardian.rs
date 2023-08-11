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
    app.add_systems(Update, (
      spawn_guardians,
      update_actor_sprite::<Guardian>,
      move_guardians,
      change_direction
    ).chain());
  }
}

#[derive(Component, Debug)]
struct Guardian {
  id: u8,
  data: cavern::Guardian,
}

fn spawn_guardians(
  mut commands: Commands,
  query: Query<Entity, With<Guardian>>,
  cavern: ResMut<Cavern>,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
) {
  if cavern.is_changed() {

    // Despawn old entities
    for entity in query.iter() {
      commands.entity(entity).despawn();
    }

    let cavern_data = &game_data.caverns[cavern.cavern_number];

    // Create images for guardian sprites.

    for (id, g) in cavern_data.guardians.iter().enumerate() {
      let images: Vec<_> = cavern_data
        .guardian_bitmaps
        .iter()
        .map(|s| images.add(s.render_with_color(&g.attributes)))
        .collect();

      let mut position = Position::at_char_pos(Layer::Characters, g.start_pos);
      let movement = HorizontalMotion {
        walking: true,
        current_frame: g.first_animation_frame as usize
      };

      // If we're initially moving left, we need to move the position to the rightmost
      // pixel position of the cell.
      if movement.direction() == Direction::Left {
        position.step(Direction::Right);
        position.step(Direction::Right);
        position.step(Direction::Right);
      }

      commands.spawn(Actor::new(
        Guardian {
          id: id as u8,
          data: g.clone(),
        },
        position,
        Sprites {
          images,
        },
        movement,
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