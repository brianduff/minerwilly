//! Implements the status area showing how many lives Willy has.

use bevy::{prelude::*, sprite::Anchor};

use crate::{
  color::{Attributes, ColorName},
  gamedata::GameDataResource,
  position::{Layer, Position},
  text::{Text, TextAttributes},
};

static LIVES_TIMER_TICK: f32 = 0.3;

pub struct LivesPlugin;

#[derive(Resource)]
pub struct Lives {
  pub lives_remaining: u8,
  current_animation_frame: u8,
  animation_timer: Timer,
}

#[derive(Resource, Deref)]
struct Textures(Vec<Handle<Image>>);

#[derive(Component)]
struct LifeSprite;

impl Plugin for LivesPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_life_sprites);
  }
}

fn setup(
  mut commands: Commands,
  game_data: Res<GameDataResource>,
  mut images: ResMut<Assets<Image>>,
) {
  commands.insert_resource(Lives {
    lives_remaining: 3,
    current_animation_frame: 0,
    animation_timer: Timer::from_seconds(LIVES_TIMER_TICK, TimerMode::Repeating),
  });

  // Get the animation images for the lives sprites
  let color = Attributes::new(ColorName::Cyan, ColorName::Black, true);

  let mut images: Vec<_> = game_data
    .willy_sprites
    .iter()
    .map(|s| images.add(s.render_with_color(&color)))
    .collect();
  // We only need the first 4 textures, which are the walking right animation.
  images.truncate(4);
  commands.insert_resource(Textures(images));

  // Render the background of the lives area, which is a fully filled black background (which we cheatingly generate using text)
  let bg_color = TextAttributes::new(ColorName::Black, ColorName::Black);

  for y in 20..24 {
    commands.spawn(Text::new(
      "                                ",
      (0, y),
      &bg_color,
    ));
  }
}

fn update_life_sprites(
  mut commands: Commands,
  time: Res<Time>,
  textures: Res<Textures>,
  mut lives: ResMut<Lives>,
  mut query: Query<(Entity, &mut Handle<Image>), With<LifeSprite>>,
) {
  lives.animation_timer.tick(time.delta());
  if lives.animation_timer.just_finished() {
    // Find the right texture for the current animation frame.
    let texture = &textures[lives.current_animation_frame as usize];

    // Update all images to use the current animation frame texture
    query.for_each_mut(|(_, mut image)| {
      *image = texture.clone();
    });

    // Despawn any extra lives that we don't need any more.
    let mut count: u8 = 0;
    query.for_each(|(entity, _)| {
      count += 1;
      if count >= lives.lives_remaining {
        commands.entity(entity).despawn();
      }
    });

    // Are we missing sprites?
    if count < lives.lives_remaining - 1 {
      for i in count..lives.lives_remaining - 1 {
        commands.spawn((
          LifeSprite,
          SpriteBundle {
            sprite: Sprite {
              anchor: Anchor::TopLeft,
              ..Default::default()
            },
            texture: texture.clone(),
            transform: Position::at_char_pos(Layer::Characters, (i * 2, 21)).into(),
            ..Default::default()
          },
        ));
      }
    }

    lives.current_animation_frame += 1;
    if lives.current_animation_frame == 4 {
      lives.current_animation_frame = 0;
    }
  }
}
