use bevy::{ecs::query::Has, prelude::*};

use crate::{
  actors::{update_actor_sprite, Actor, Direction, HorizontalMotion, Sprites},
  cavern::CavernState,
  color::{Attributes, ColorName},
  debug::{DebugStateToggled, DebugText},
  gamedata::{cavern::CavernTileType, GameDataResource},
  item::Item,
  position::{vec2, Layer, Position, Relative},
  timer::GameTimer,
  SCALE,
};

static JUMP_DELTAS: [f32; 16] = [
  4.0, 4.0, 3.0, 3.0, 2.0, 2.0, 1.0, 1.0, -1.0, -1.0, -2.0, -2.0, -3.0, -3.0, -4.0, -4.0,
];

pub struct WillyPlugin;

impl Plugin for WillyPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
    app.add_systems(
      Update,
      (
        check_wall_collision,
        check_keyboard,
        move_willy,
        update_actor_sprite::<Willy>,
        check_collisions,
        check_drop,
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
pub struct Willy {
  pub airborne_status: AirborneStatus,
  jump_counter: u8,
  can_move_left: bool,
  can_move_right: bool,
}

impl Willy {
  fn can_move(&self, direction: Direction) -> bool {
    match direction {
      Direction::Left => self.can_move_left,
      Direction::Right => self.can_move_right,
    }
  }
}

/// Willy's airborne status.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]

pub enum AirborneStatus {
  NotJumpingOrFalling,
  Jumping,
  FallingSafeToLand,
  FallingUnsafeToLand,
  Collided, // not sure if we need this
}

impl AirborneStatus {
  pub fn is_airborne(&self) -> bool {
    !matches!(
      self,
      AirborneStatus::NotJumpingOrFalling | AirborneStatus::Collided
    )
  }

  fn is_falling(&self) -> bool {
    matches!(
      self,
      AirborneStatus::FallingSafeToLand | AirborneStatus::FallingUnsafeToLand
    )
  }
}

#[derive(Debug, Default, Resource)]
struct KeyboardState {
  left_pressed: bool,
  right_pressed: bool,
  jump_pressed: bool,
}

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

  // TODO: use cavern data to set spawn position
  let willy_pos = Position::at_char_pos(Layer::Characters, (2, 13));

  commands.spawn(Actor::new(
    Willy {
      airborne_status: AirborneStatus::NotJumpingOrFalling,
      jump_counter: 0,
      can_move_left: true,
      can_move_right: true,
    },
    willy_pos,
    Sprites { images },
    HorizontalMotion {
      walking: false,
      current_frame: 0,
    },
  ));

  commands.insert_resource(KeyboardState::default());
  commands.insert_resource(DebugState {
    show_debug_info: true,
  });
}

#[allow(clippy::type_complexity)]
fn move_willy(
  timer: Res<GameTimer>,
  keys: ResMut<KeyboardState>,
  mut query: Query<(&mut Position, &mut Willy, &mut HorizontalMotion), With<Willy>>,
) {
  let (mut position, mut willy, mut motion) = query.single_mut();

  if timer.just_finished() {
    if keys.jump_pressed && !&willy.airborne_status.is_airborne() {
      willy.airborne_status = AirborneStatus::Jumping;
      willy.jump_counter = 0;
    }

    if !&willy.airborne_status.is_airborne() {
      // TODO: clean this up - wtf?

      // If no key is pressed, we're not walking.
      if !keys.left_pressed && !keys.right_pressed {
        motion.walking = false;
      } else {
        // Else we're walking
        motion.walking = true;
        // If only left is pressed, we're walking left.
        if keys.left_pressed && !keys.right_pressed {
          motion.set_direction(Direction::Left);
        } else if !keys.left_pressed {
          motion.set_direction(Direction::Right);
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
        motion.set_direction(Direction::Right);
      }

      if !motion.walking && !keys.right_pressed && keys.left_pressed {
        motion.walking = true;
        motion.set_direction(Direction::Left);
      }
    }

    // Stop moving if we've hit a wall.
    if motion.walking
      && position.will_change_cell(motion.direction())
      && !willy.can_move(motion.direction())
    {
      motion.walking = false;
    }

    // First, check if we're airborne. In this case, we move the y-coordinate of
    // willy, and increment the jump animation counter.
    if willy.airborne_status.is_airborne() {
      if willy.jump_counter <= 15 {
        let delta = JUMP_DELTAS[willy.jump_counter as usize];
        position.jump(delta);
      }

      if willy.jump_counter > 7 {
        willy.airborne_status = AirborneStatus::FallingSafeToLand;
      }

      // In free fall!
      if willy.jump_counter > 15 {
        position.jump(-4.0);
        // Stop walking
        motion.walking = false;
      }

      if willy.jump_counter > 20 {
        willy.airborne_status = AirborneStatus::FallingUnsafeToLand;
      }

      willy.jump_counter += 1;
    }

    if motion.walking {
      motion.step(&mut position);
    }
  }
}

fn check_keyboard(keys: Res<Input<KeyCode>>, mut keyboard_state: ResMut<KeyboardState>) {
  keyboard_state.left_pressed = pressed(&keys, &LEFT_KEYS);
  keyboard_state.right_pressed = pressed(&keys, &RIGHT_KEYS);
  keyboard_state.jump_pressed = pressed(&keys, &[KeyCode::Space]);
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
  cavern_state: Res<CavernState>,
  mut query: Query<(&Position, &mut Willy), (With<Willy>, Or<(Changed<Willy>, Changed<Position>)>)>,
) {
  if query.get_single().is_ok() {
    let (position, mut motion) = query.get_single_mut().unwrap();

    let (curx, cury) = position.char_pos();

    motion.can_move_left = !position.relative(Relative::Left).iter().map(|p| cavern_state.get_tile_type(*p)).any(|tt| matches!(tt, CavernTileType::Wall));

    // motion.can_move_left = !matches!(
    //   cavern_state.get_tile_type((curx - 1, cury)),
    //   CavernTileType::Wall
    // ) && !matches!(
    //   cavern_state.get_tile_type((curx - 1, cury + 1)),
    //   CavernTileType::Wall
    // );
    motion.can_move_right = !matches!(
      cavern_state.get_tile_type((curx + 2, cury)),
      CavernTileType::Wall
    ) && !matches!(
      cavern_state.get_tile_type((curx + 2, cury + 1)),
      CavernTileType::Wall
    );
  }
}

// Check if Willy should drop
fn check_drop(
  cavern_state: Res<CavernState>,
  timer: Res<GameTimer>,
  mut query: Query<(&mut Willy, &mut HorizontalMotion, &Position), Has<Willy>>,
) {
  let (mut willy, mut motion, position) = query.get_single_mut().unwrap();
  if timer.just_finished() && !willy.airborne_status.is_airborne() && !can_stand(position, &cavern_state) {
    willy.airborne_status = AirborneStatus::FallingSafeToLand;
    willy.jump_counter = 8;
    motion.walking = false;
  }
}

// Check if willy has landed on something. Ideally a floor ;)
fn check_landing(
  cavern_state: Res<CavernState>,
  timer: Res<GameTimer>,
  mut query: Query<(&mut Willy, &Position), Has<Willy>>,
) {
  let (mut motion, position) = query.get_single_mut().unwrap();

  // TODO: there's a bug where we don't get some positions to check for a landing. Debug why?
  if timer.just_finished() && motion.airborne_status.is_falling() && position.is_vertically_cell_aligned() {
    println!("Can land at {:?}? {}", position.char_pos(), can_stand(position, &cavern_state));
  }

  if timer.just_finished() &&
      motion.airborne_status.is_falling() &&
      position.is_vertically_cell_aligned() &&
      can_stand(position, &cavern_state) {
    println!("Landed");
    motion.airborne_status = AirborneStatus::NotJumpingOrFalling;
  }

}

fn can_stand(position: &Position, cavern_state: &CavernState) -> bool {
  position
    .relative(Relative::Below)
    .iter()
    .map(|p| cavern_state.get_tile_type(*p).can_stand())
    .any(|v| v)
}

#[derive(Resource)]
struct DebugState {
  show_debug_info: bool,
}

fn listen_for_debug(
  mut event_reader: EventReader<DebugStateToggled>,
  mut state: ResMut<DebugState>,
) {
  if !event_reader.is_empty() {
    let event = event_reader.iter().last().unwrap();
    state.show_debug_info = **event;
  }
}

#[allow(clippy::type_complexity)]
fn update_debug_info(
  mut debug_text: ResMut<DebugText>,
  query: Query<(&Willy, &Position), (With<Willy>, Or<(Changed<Willy>, Changed<Position>)>)>,
) {
  if query.get_single().is_ok() {
    let (motion, position) = query.get_single().unwrap();

    debug_text.line1 = format!("Pos: {:?} {:?}", position.pixel_pos(), position.char_pos());
    debug_text.line2 = format!("{:?}", motion.airborne_status);
    debug_text.line3 = format!(
      "can move: L: {:?} R: {:?}",
      motion.can_move_left, motion.can_move_right
    );
  }
}

#[allow(clippy::type_complexity)]
fn draw_debug_overlay(
  mut gizmos: Gizmos,
  debug_state: Res<DebugState>,
  query: Query<&Position, (With<Willy>,)>,
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
    gizmos.rect_2d(
      vec2((x, y)),
      0.,
      vec2((16. * SCALE, 16. * SCALE)),
      Color::WHITE,
    );

    // Draw a box around willy's 8*16 bounding pixel box

    let (mut x, mut y) = position.pixel_pos();

    x += 4. * SCALE;
    y -= 8. * SCALE;

    gizmos.rect_2d(
      vec2((x, y)),
      0.,
      vec2((8. * SCALE, 16. * SCALE)),
      Color::GOLD,
    );
  }
}

/// Checks for collisions.
fn check_collisions(
  cavern_state: Res<CavernState>,
  position: Query<&Position, (With<Willy>, Changed<Position>)>,
  mut item_query: Query<(&mut Item, &Position)>,
) {
  if !position.is_empty() {
    let pos = position.get_single().unwrap();

    // If any of the four grid cells that contain Willy's sprite contain
    // a nasty, he has collided.
    for (x, y) in pos.relative(Relative::Inside) {
      if cavern_state.get_tile_type((x, y)).is_nasty() {
        println!("Collided with NASTY at {:?}", (x, y));
      }

      // Did we intersect an item?
      for (mut item, pos) in item_query.iter_mut() {
        if pos.char_pos() == (x, y) && !item.collected {
          item.collected = true;
          println!("Collided with ITEM at {:?}", (x, y))
        }
      }
    }
  }
}
