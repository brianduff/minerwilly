use bevy::{ecs::query::Has, prelude::*, sprite::Anchor};

use crate::{
  cavern::Cavern,
  color::{Attributes, ColorName},
  debug::{DebugText, DebugStateToggled},
  gamedata::{cavern::CavernTileType, GameDataResource},
  position::{Direction, Layer, Position, vec2},
  TIMER_TICK, SCALE,
};

static JUMP_DELTAS: [f32; 16] = [
  4.0, 4.0, 3.0, 3.0, 2.0, 2.0, 1.0, 1.0, -1.0, -1.0, -2.0, -2.0, -3.0, -3.0, -4.0, -4.0,
];

pub struct WillyPlugin;

// The number of frames in willy's sprite animation
static FRAME_COUNT: usize = 4;

impl Plugin for WillyPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
    app.add_systems(
      Update,
      (
        check_wall_collision,
        check_keyboard,
        move_willy,
        check_landing,
        listen_for_debug,
        draw_debug_overlay,
      )
        .chain(),
    );
    app.add_systems(Update, update_debug_info);
  }
}

#[derive(Component)]
struct WillySprites {
  images: Vec<Handle<Image>>,
  current_frame: usize,
}

/// Willy's airborne status.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]

enum AirborneStatus {
  NotJumpingOrFalling,
  Jumping,
  FallingSafeToLand,
  FallingUnsafeToLand,
  Collided, // not sure if we need this
}

impl AirborneStatus {
  fn is_airborne(&self) -> bool {
    !matches!(
      self,
      AirborneStatus::NotJumpingOrFalling | AirborneStatus::Collided
    )
  }

  fn is_falling(&self) -> bool {
    matches!(self, AirborneStatus::FallingSafeToLand | AirborneStatus::FallingUnsafeToLand)
  }
}

#[derive(Debug, Default, Resource)]
struct KeyboardState {
  left_pressed: bool,
  right_pressed: bool,
  jump_just_pressed: bool
}

// TODO: this could just be a resource rather than a component, since willy's
// state is effectively global.
#[derive(Component, Debug)]
struct WillyMotion {
  walking: bool,
  airborne_status: AirborneStatus,
  direction: Direction,
  // This keeps track of where in a jump we are. It's
  // initialized to 0 when jumping starts, and incremented
  // on each timer tick as long as we're jumping.
  jump_counter: u8,

  // Collision detection will update these to indicate whether moving to the
  // next cell is possible in the given direction.
  can_move_left: bool,
  can_move_right: bool,
}

impl WillyMotion {
  fn can_move(&self) -> bool {
    match self.direction {
      Direction::Left => self.can_move_left,
      Direction::Right => self.can_move_right
    }
  }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
  mut commands: Commands,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
) {
  let willy_color = Attributes::new_transparent_bg(ColorName::White, false);

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
    airborne_status: AirborneStatus::NotJumpingOrFalling,
    direction: Direction::Right,
    jump_counter: 0,
    can_move_left: true,
    can_move_right: true,
  };

  // TODO: use cavern data to set spawn position
  let willy_pos = Position::at_char_pos(Layer::Characters, (2, 13));
  let transform: Transform = (&willy_pos).into();

  // Spawn Willy
  commands.spawn((
    motion,
    willy_pos,
    sprite_images,
    AnimationTimer(Timer::from_seconds(TIMER_TICK, TimerMode::Repeating)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: initial_texture,
      // TODO: use the cavern data to spawn in the right place
      transform,
      ..Default::default()
    },
  ));

  commands.insert_resource(KeyboardState::default());
  commands.insert_resource(DebugState { show_debug_info: true });

}

#[allow(clippy::type_complexity)]
fn move_willy(
  time: Res<Time>,
  // keys: Res<Input<KeyCode>>,
  mut keys: ResMut<KeyboardState>,
  mut query: Query<
    (
      &mut Position,
      &mut WillyMotion,
      &mut AnimationTimer,
      &mut WillySprites,
      &mut Handle<Image>,
      &mut Transform,
    ),
    With<WillyMotion>,
  >,
) {
  let (mut position, mut motion, mut timer, mut sprites, mut image, mut transform) =
    query.single_mut();

  timer.tick(time.delta());

  if timer.just_finished() {

    let old_direction = motion.direction;

    // let left_pressed = pressed(&keys, &LEFT_KEYS);
    // let right_pressed = pressed(&keys, &RIGHT_KEYS);
    // let jump_pressed = keys.just_pressed(KeyCode::Space);

    if keys.jump_just_pressed {
      keys.jump_just_pressed = false;
      if !&motion.airborne_status.is_airborne() {
        motion.airborne_status = AirborneStatus::Jumping;
        motion.jump_counter = 0;
      }
    }

    if !&motion.airborne_status.is_airborne() {
      // If no key is pressed, we're not walking.
      if !keys.left_pressed && !keys.right_pressed {
        motion.walking = false;
      } else {
        // Else we're walking
        motion.walking = true;
        // If only left is pressed, we're walking left.
        if keys.left_pressed && !keys.right_pressed {
          motion.direction = Direction::Left;
        } else if !keys.left_pressed {
          motion.direction = Direction::Right;
        }
      }

      if keys.left_pressed || keys.right_pressed {
        motion.walking = true;

      }

      if motion.walking && (!keys.right_pressed && !keys.left_pressed) {
        motion.walking = false;
      }

      if !motion.walking && keys.right_pressed {
        motion.walking = true;
        motion.direction = Direction::Right;
      }

      if !motion.walking && !keys.right_pressed && keys.left_pressed {
        motion.walking = true;
        motion.direction = Direction::Left;
      }
    }

    // // Whatever we're doing, we must stop moving horizontally if we hit a wall.
    // if motion.walking
    //   && (motion.direction == Direction::Right && !motion.can_move_right)
    //     | (motion.direction == Direction::Left && !motion.can_move_left)
    // {
    //   motion.walking = false;
    // }

    let changed_direction = old_direction != motion.direction;
    if changed_direction {
      sprites.current_frame = 3 - sprites.current_frame;
    }

    // Stop moving if we've hit a wall.
    if motion.walking && position.will_change_cell(motion.direction) && !motion.can_move()  {
      motion.walking = false;
    }


    // First, check if we're airborne. In this case, we move the y-coordinate of
    // willy, and increment the jump animation counter.
    if motion.airborne_status.is_airborne() {
      if motion.jump_counter <= 15 {
        let delta = JUMP_DELTAS[motion.jump_counter as usize];
        position.jump(delta);
      }

      if motion.jump_counter > 7 {
        motion.airborne_status = AirborneStatus::FallingSafeToLand;
      }

      // In free fall!
      if motion.jump_counter > 15 {
        position.jump(-4.0);
        // Stop walking
        motion.walking = false;
      }

      if motion.jump_counter > 20 {
        motion.airborne_status = AirborneStatus::FallingUnsafeToLand;
      }

      motion.jump_counter += 1;
    }

    if motion.walking {
      let cycle = sprites.current_frame == FRAME_COUNT - 1;

      if !changed_direction {
        sprites.current_frame = if cycle { 0 } else { sprites.current_frame + 1 };
      }

      if !changed_direction {
        position.step(motion.direction);
      }
    }

    // Compute the texture index 0-7 of the current animation frame.
    let texture_index = match motion.direction {
      Direction::Right => sprites.current_frame,
      Direction::Left => 4 + (3 - sprites.current_frame),
    };
    // Update the image we're using for the sprite
    *image = sprites.images[texture_index].clone();


    // Actually move the sprite
    *transform = (&*position).into();
  }
}

fn check_keyboard(
  keys: Res<Input<KeyCode>>,
  mut keyboard_state: ResMut<KeyboardState>) {

  keyboard_state.left_pressed = pressed(&keys, &LEFT_KEYS);
  keyboard_state.right_pressed = pressed(&keys, &RIGHT_KEYS);

  if keys.just_pressed(KeyCode::Space) {
    keyboard_state.jump_just_pressed = true;
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

// Check to see if moving left or right would collide with a wall, and should
// therefore be disallowed. This will update the can_move_left and can_move_right
// fields of WillyMotion.
#[allow(clippy::type_complexity)]
fn check_wall_collision(
  data: Res<GameDataResource>,
  cavern: Res<Cavern>,
  mut query: Query<
    (&Position, &mut WillyMotion),
    (
      With<WillyMotion>,
      Or<(Changed<WillyMotion>, Changed<Position>)>,
    ),
  >,
) {
  if query.get_single().is_ok() {
    let (position, mut motion) = query.get_single_mut().unwrap();

    let cavern_data = &data.caverns[cavern.cavern_number];

    let (curx, cury) = position.char_pos();

    motion.can_move_left = !matches!(
      cavern_data.get_tile_type((curx - 1, cury)),
      CavernTileType::Wall
    ) && !matches!(
      cavern_data.get_tile_type((curx - 1, cury + 1)),
      CavernTileType::Wall
    );
    motion.can_move_right = !matches!(
      cavern_data.get_tile_type((curx + 2, cury)),
      CavernTileType::Wall
    ) && !matches!(
      cavern_data.get_tile_type((curx + 2, cury + 1)),
      CavernTileType::Wall
    );
  }
}

// Check if willy has landed on something. Ideally a floor ;)
fn check_landing(
  data: Res<GameDataResource>,
  cavern: Res<Cavern>,
  mut query: Query<(&mut WillyMotion, &Position), Has<WillyMotion>>,
) {
  let (mut motion, position) = query.get_single_mut().unwrap();
  if motion.is_changed() && motion.airborne_status.is_falling() {
    // Willy must be on a precise cell boundary to land.
    // let (cx, cy, pxo, pyo) = to_cell((transform.translation.x, transform.translation.y));
    if position.is_vertically_cell_aligned() {
      let (cx, cy) = position.char_pos();
      // Is the tile under willy's feet something he can stand on?
      let cavern_data = &data.caverns[cavern.cavern_number];
      if cavern_data.get_tile_type((cx, cy + 2)).can_land() || cavern_data.get_tile_type((cx + 1, cy + 2)).can_land() {
        motion.walking = false;
        motion.airborne_status = AirborneStatus::NotJumpingOrFalling;
      }
    }
  }
}

#[derive(Resource)]
struct DebugState {
  show_debug_info: bool
}

fn listen_for_debug(mut event_reader: EventReader<DebugStateToggled>, mut state: ResMut<DebugState>) {
  if !event_reader.is_empty() {
    let event = event_reader.iter().last().unwrap();
    state.show_debug_info = **event;

  }
}

#[allow(clippy::type_complexity)]
fn update_debug_info(
  mut debug_text: ResMut<DebugText>,
  query: Query<
    (&WillyMotion, &Position, &WillySprites),
    (
      With<WillyMotion>,
      Or<(Changed<WillyMotion>, Changed<Position>)>,
    ),
  >,
) {
  if query.get_single().is_ok() {
    let (motion, position, sprites) = query.get_single().unwrap();


    debug_text.line1 = format!("Pos: {:?} {:?}", position.pixel_pos(), position.char_pos());
    debug_text.line2 = format!("{:?} S:{}", motion.airborne_status, sprites.current_frame);
    debug_text.line3 = format!("can move: L: {:?} R: {:?}", motion.can_move_left, motion.can_move_right);
  }
}

#[allow(clippy::type_complexity)]
fn draw_debug_overlay(
  mut gizmos: Gizmos,
  debug_state: Res<DebugState>,
  query: Query<
    &Position,
    (
      With<WillyMotion>,
    ),
  >,
) {
  if !debug_state.show_debug_info {
    return;
  }
  // Draw a bounding box around Willy's sprite charbox
  if query.get_single().is_ok() {

    let position = query.get_single().unwrap();

    let (mut x, mut y) = position.get_cell_box();

    x += 8. * SCALE;
    y -= 8. * SCALE;

    // Draw a box around Willy's 16x16 sprite grid
    gizmos.rect_2d(vec2((x, y)), 0., vec2((16. * SCALE, 16. * SCALE)), Color::WHITE);

    // Draw a box around willy's 8*16 bounding pixel box

    let (mut x, mut y) = position.pixel_pos();

    x += 4. * SCALE;
    y -= 8. * SCALE;

    gizmos.rect_2d(vec2((x, y)), 0., vec2((8. * SCALE, 16. * SCALE)), Color::GOLD);
  }
}

