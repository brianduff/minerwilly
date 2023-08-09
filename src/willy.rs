use bevy::{ecs::query::Has, prelude::*, sprite::Anchor};

use crate::{
  cavern::Cavern,
  color::{Attributes, ColorName},
  debug::DebugText,
  gamedata::{cavern::CavernTileType, GameDataResource},
  position::{Direction, Layer, Position},
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

fn is_airborne(status: &AirborneStatus) -> bool {
  !matches!(
    status,
    AirborneStatus::NotJumpingOrFalling | AirborneStatus::Collided
  )
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
}

#[allow(clippy::type_complexity)]
fn move_willy(
  time: Res<Time>,
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
    // First, check if we're airborne. In this case, we move the y-coordinate of
    // willy, and increment the jump animation counter.
    if is_airborne(&motion.airborne_status) {
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

      sprites.current_frame = if cycle { 0 } else { sprites.current_frame + 1 };

      // Compute the texture index 0-7 of the current animation frame.
      let texture_index = match motion.direction {
        Direction::Right => sprites.current_frame,
        Direction::Left => 4 + (3 - sprites.current_frame),
      };

      // Update the image we're using for the sprite
      *image = sprites.images[texture_index].clone();

      position.step(motion.direction);
    }

    // Actually move the sprite
    *transform = (&*position).into();
  }
}

fn check_keyboard(
  keys: Res<Input<KeyCode>>,
  mut query: Query<(&mut WillyMotion, &mut WillySprites), Has<WillyMotion>>,
) {
  let (mut motion, mut sprites) = query.single_mut();

  let old_direction = motion.direction;

  let left_pressed = pressed(&keys, &LEFT_KEYS);
  let right_pressed = pressed(&keys, &RIGHT_KEYS);
  let jump_pressed = keys.just_pressed(KeyCode::Space);

  if jump_pressed && !is_airborne(&motion.airborne_status) {
    motion.airborne_status = AirborneStatus::Jumping;
    motion.jump_counter = 0;
  }

  if !is_airborne(&motion.airborne_status) {
    motion.walking = false;
    if !motion.walking && right_pressed {
      motion.walking = true;
      motion.direction = Direction::Right;
    } else if !motion.walking && left_pressed {
      motion.walking = true;
      motion.direction = Direction::Left;
    }
  }

  // Whatever we're doing, we must stop moving horizontally if we hit a wall.
  if motion.walking
    && (motion.direction == Direction::Right && !motion.can_move_right)
      | (motion.direction == Direction::Left && !motion.can_move_left)
  {
    motion.walking = false;
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
      cavern_data.get_tile_type((curx, cury)),
      CavernTileType::Wall
    ) && !matches!(
      cavern_data.get_tile_type((curx, cury + 1)),
      CavernTileType::Wall
    );
    motion.can_move_right = !matches!(
      cavern_data.get_tile_type((curx + 1, cury)),
      CavernTileType::Wall
    ) && !matches!(
      cavern_data.get_tile_type((curx + 1, cury + 1)),
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
  if motion.is_changed() && motion.airborne_status == AirborneStatus::FallingSafeToLand {
    // Willy must be on a precise cell boundary to land.
    // let (cx, cy, pxo, pyo) = to_cell((transform.translation.x, transform.translation.y));
    if position.is_vertically_cell_aligned() {
      let (cx, cy) = position.char_pos();
      // Is the tile under willy's feet something he can stand on?
      let cavern_data = &data.caverns[cavern.cavern_number];
      if cavern_data.get_tile_type((cx, cy + 2)).can_land() {
        motion.walking = false;
        motion.airborne_status = AirborneStatus::NotJumpingOrFalling;
      }
    }
  }
}

#[allow(clippy::type_complexity)]
fn update_debug_info(
  mut debug_text: ResMut<DebugText>,
  data: Res<GameDataResource>,
  cavern: Res<Cavern>,
  query: Query<
    (&WillyMotion, &Position),
    (
      With<WillyMotion>,
      Or<(Changed<WillyMotion>, Changed<Position>)>,
    ),
  >,
) {
  if query.get_single().is_ok() {
    let (motion, position) = query.get_single().unwrap();

    let cavern = &data.caverns[cavern.cavern_number];

    debug_text.line1 = format!("Pos: {:?} {:?}", position.pixel_pos(), position.char_pos());
    debug_text.line2 = format!("{:?}", motion.airborne_status);
  }
}

#[allow(clippy::type_complexity)]
fn draw_debug_overlay(
  mut gizmos: Gizmos,
  query: Query<
    (&WillyMotion, &Position),
    (
      With<WillyMotion>,
      Or<(Changed<WillyMotion>, Changed<Position>)>,
    ),
  >,
) {
  // Draw a bounding box around Willy's sprite charbox
  if query.get_single().is_ok() {

    let (motion, position) = query.get_single().unwrap();

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

fn vec3(layer: Layer, (x, y): (f32, f32)) -> Vec3 {
  let z = (layer as u32) as f32;
  Vec3::new(x, y, z)
}

fn vec2((x, y): (f32, f32)) -> Vec2 {
  Vec2::new(x, y)
}
