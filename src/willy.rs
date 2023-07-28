use bevy::{ecs::query::Has, prelude::*, sprite::Anchor};

use crate::{
  color::{SpectrumColor, SpectrumColorName},
  gamedata::GameDataResource,
  position::{at_char_pos, Layer},
  CELLSIZE, SCALE, TIMER_TICK,
};

pub struct WillyPlugin;

// The number of frames in willy's sprite animation
static FRAME_COUNT: usize = 4;

impl Plugin for WillyPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
    app.add_systems(Update, (move_willy, check_keyboard));
  }
}

#[derive(Component)]
struct WillySprites {
  images: Vec<Handle<Image>>,
  current_frame: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Direction {
  Left,
  Right,
}

#[derive(Component, Debug)]
struct WillyMotion {
  walking: bool,
  jumping: bool,
  direction: Direction,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
  mut commands: Commands,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
) {
  let willy_color = SpectrumColor::new_transparent_bg(SpectrumColorName::White, false);

  let images: Vec<_> = game_data
    .willy_sprites
    .iter()
    .map(|s| images.add(s.render_with_color(&willy_color)))
    .collect();
  let initial_texture = images.first().unwrap().clone();

  let sprite_images = WillySprites {
    images,
    current_frame: 0,
  };

  let motion = WillyMotion {
    walking: false,
    jumping: false,
    direction: Direction::Right,
  };

  // Spawn Willy
  commands.spawn((
    motion,
    sprite_images,
    AnimationTimer(Timer::from_seconds(TIMER_TICK, TimerMode::Repeating)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: initial_texture,
      transform: at_char_pos(Layer::Characters, (2, 13)),
      ..Default::default()
    },
  ));
}

#[allow(clippy::type_complexity)]
fn move_willy(
  time: Res<Time>,
  mut query: Query<
    (
      &WillyMotion,
      &mut AnimationTimer,
      &mut WillySprites,
      &mut Handle<Image>,
      &mut Transform,
    ),
    Has<WillyMotion>,
  >,
) {
  let (motion, mut timer, mut sprites, mut image, mut transform) = query.single_mut();

  timer.tick(time.delta());
  if timer.just_finished() && (motion.walking || motion.jumping) {
    let cycle = sprites.current_frame == FRAME_COUNT - 1;

    sprites.current_frame = if cycle { 0 } else { sprites.current_frame + 1 };

    // Compute the texture index 0-7 of the current animation frame.
    let texture_index = match motion.direction {
      Direction::Right => sprites.current_frame,
      Direction::Left => 4 + (3 - sprites.current_frame),
    };

    // Update the image we're using for the sprite
    *image = sprites.images[texture_index].clone();

    // If we've reached the bound of the current frame, then move to the next char pos
    if cycle {
      transform.translation.x += match motion.direction {
        Direction::Left => -CELLSIZE,
        Direction::Right => CELLSIZE,
      };
    }
  }
}

fn check_keyboard(
  keys: Res<Input<KeyCode>>,
  mut query: Query<(&mut WillyMotion, &mut WillySprites), Has<WillyMotion>>,
) {
  let (mut motion, mut sprites) = query.single_mut();

  let old_direction = motion.direction;

  motion.walking = false;
  if !motion.walking && pressed(&keys, &RIGHT_KEYS) {
    motion.walking = true;
    motion.direction = Direction::Right;
  } else if !motion.walking && pressed(&keys, &LEFT_KEYS) {
    motion.walking = true;
    motion.direction = Direction::Left;
  }

  if old_direction != motion.direction {
    sprites.current_frame = 3 - sprites.current_frame;
  }
}

fn pressed(keys: &Res<'_, Input<KeyCode>>, expected: &[KeyCode]) -> bool {
  for code in expected {
    if keys.pressed(*code) {
      return true;
    }
  }
  false
}

const LEFT_KEYS: [KeyCode; 2] = [KeyCode::Left, KeyCode::O];
const RIGHT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::P];
