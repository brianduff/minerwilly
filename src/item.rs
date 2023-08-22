use bevy::prelude::*;

use crate::{gamedata::{cavern, GameDataResource}, cavern::Cavern, despawn_on_cavern_change, actors::{Actor, HorizontalMotion, Sprites, update_actor_sprite}, position::Position, bitmap::Bitmap, color::{Attributes, ColorName}, timer::GameTimer, clamp};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (
      update_actor_sprite::<Item>,
      (despawn_on_cavern_change::<Item>, spawn_items).chain(),
      cycle_items,
      despawn_when_collected,
    ).chain());
  }
}

#[derive(Component, Debug)]
pub struct Item {
  data: cavern::Item,
  pub collected: bool
}

fn spawn_items(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cavern: ResMut<Cavern>,
    game_data: Res<GameDataResource>
) {
  if cavern.is_changed() {
    let cavern_data = &game_data.caverns[cavern.cavern_number];

    for item in cavern_data.items.iter() {
      let images = create_cycle_images(&cavern_data.item_bitmap, &item.attributes)
          .into_iter()
          .map(|a| images.add(a)).collect();

      commands.spawn(Actor::new(
        Item {
          data: *item,
          collected: false
        },
        Position::at_char_pos(crate::position::Layer::Items, item.position),
        Sprites {
          images
        },
        HorizontalMotion::frozen()
      ));
    }
  }
}

fn cycle_items(timer: Res<GameTimer>, mut query: Query<&mut HorizontalMotion, With<Item>>) {
  if timer.just_finished() {
    for mut motion in query.iter_mut() {
      motion.current_frame = clamp(motion.current_frame + 1, 0, 3);
    }
  }
}


fn create_cycle_images(bitmap: &Bitmap, initial_color: &Attributes) -> Vec<Image> {
  let mut images = Vec::with_capacity(4);

  // Find the starting index in COLOR_SEQUENCE based on the initial_color.
  let mut start_index = None;
  for (index, color) in COLOR_SEQUENCE.iter().enumerate() {
    let color_u8 : u8 = (*color).into();
    if color_u8 == initial_color.ink {
      start_index = Some(index);
      break;
    }
  }

  // Snap back to magenta in case the ink color isn't one of the colors in the
  // sequence.
  let mut index = start_index.unwrap_or(0);
  for _ in 0..4 {
    let mut attributes = *initial_color;
    attributes.ink = COLOR_SEQUENCE[index].into();
    images.push(bitmap.render_with_color(&attributes));

    index = clamp(index + 1, 0, 3);
  }

  images
}

fn despawn_when_collected(mut commands: Commands, query: Query<(&Item, Entity), Changed<Item>>) {
  for (item, entity) in query.iter() {
    if item.collected {
      commands.entity(entity).despawn();
    }
  }
}

/// The sequence of ink colors that items animate through.
const COLOR_SEQUENCE: [ColorName; 4] = [
  ColorName::Magenta,
  ColorName::Yellow,
  ColorName::Cyan,
  ColorName::Green
];