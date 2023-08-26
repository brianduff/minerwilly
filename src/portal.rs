use bevy::{prelude::*, sprite::Anchor};

use crate::{position::{Position, Layer}, cavern::CurrentCavern, gamedata::{GameDataResource, self}, timer::GameTimer, despawn_all};

/// The number of timer ticks between flashes of the portal.
const TICKS_PER_FLASH: usize = 4;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.add_systems(Update, (spawn_portal, check_debug_keyboard, flash_if_unlocked));
  }
}

#[derive(Component)]
pub struct Portal {
  unlocked: bool,
  // The position of a portal never changes.
  pub position: Position,
  normal_image: Handle<Image>,
  inverse_image: Handle<Image>,
  inverted: bool,
  countdown: usize,
}

fn spawn_portal(
    mut commands: Commands,
    cavern: ResMut<CurrentCavern>,
    game_data: Res<GameDataResource>,
    images: ResMut<Assets<Image>>,
    query: Query<Entity, With<Portal>>) {
  if cavern.is_changed() {
    despawn_all(&mut commands, query);
    let portal_data = &game_data.caverns[cavern.number].portal;

    // todo: don't splat this sprite code all over the place
    commands.spawn(PortalBundle::new(images, portal_data));
  }
}

#[derive(Bundle)]
struct PortalBundle {
  portal: Portal,
  sprite: SpriteBundle
}

impl PortalBundle {
  fn new(mut images: ResMut<Assets<Image>>, portal_data: &gamedata::cavern::Portal) -> Self {
    let position = Position::at_char_pos(Layer::Portal, portal_data.position);

    let normal_image = images.add(
        portal_data.bitmap.render_with_color(&portal_data.attributes));
    let inverse_image = images.add(
        portal_data.bitmap.render_with_color(&portal_data.attributes.inverse()));

    let sprite = SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: normal_image.clone(),
      transform: (&position).into(),
      ..Default::default()
    };

    let portal = Portal {
      position,
      normal_image,
      inverse_image,
      unlocked: false,
      inverted: false,
      countdown: TICKS_PER_FLASH
    };


    Self {
      portal,
      sprite
    }
  }
}

fn flash_if_unlocked(timer: Res<GameTimer>, mut query: Query<(&mut Portal, &mut Handle<Image>)>) {
  if timer.just_finished() {
    for (mut portal, mut image) in query.iter_mut() {
      if portal.unlocked {
        portal.countdown -= 1;
        if portal.countdown == 0 {
          portal.inverted = !portal.inverted;
          *image = if portal.inverted { portal.inverse_image.clone() } else { portal.normal_image.clone() };
          portal.countdown = TICKS_PER_FLASH;
        }
      }
    }
  }
}

fn check_debug_keyboard(keys: Res<Input<KeyCode>>, mut query: Query<&mut Portal>) {
  if keys.just_released(KeyCode::X) {
    let mut portal = query.get_single_mut().unwrap();
    portal.unlocked = true;
  }
}
