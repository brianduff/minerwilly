use bevy::prelude::*;

static SCALE: f32 = 2.0;
static CELLSIZE: f32 = 8.0 * SCALE;
static TIMER_TICK: f32 = 0.075;
// The number of frames in a sprite animation
static FRAME_COUNT: usize = 4;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .add_system(check_keyboard)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .run();
}

#[derive(Component)]
struct AnimationIndices {
    // The sprite number. Each sprite has FRAME_COUNT animation frames.
    // This number identifies which set of 4 animation frames we're currently
    // rendering.
    sprite: usize,
    frame: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down
}

#[derive(Component)]
struct WillyMotion {
    walking: bool,
    jumping: bool,
    direction: Direction
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/willysprites.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 16, 9, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { sprite: 2, frame: 0 };
    let willy_motion = WillyMotion { walking: false, direction: Direction::Right, jumping: false };
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(SCALE)),
            ..default()
        },
        animation_indices,
        willy_motion,
        AnimationTimer(Timer::from_seconds(TIMER_TICK, TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
        &WillyMotion,
    )>,
) {
    let (mut indices, mut timer, mut sprite, mut transform, motion) = query.single_mut();

    timer.tick(time.delta());
    if timer.just_finished() && (motion.walking || motion.jumping) {
        let cycle = indices.frame == FRAME_COUNT - 1;

        indices.frame = if cycle {
            0
        } else {
            indices.frame + 1
        };


        let frame = match motion.direction {
            Direction::Right => indices.frame,
            Direction::Left => 3 - indices.frame,
            _ => indices.frame
        };
        sprite.index = (indices.sprite * FRAME_COUNT) + frame;

        println!("Sprite index: {:?}", sprite.index);

        if motion.jumping {
            transform.translation.y += match motion.direction {
                Direction::Up => CELLSIZE,
                Direction::Down => -CELLSIZE,
                _ => 0.0
            }
        }


        if cycle {
            transform.translation.x += match motion.direction {
                Direction::Left => -CELLSIZE,
                Direction::Right => CELLSIZE,
                _ => 0.0
            };
        }
    }
}


fn check_keyboard(keys: Res<Input<KeyCode>>, mut query: Query<(&mut WillyMotion, &mut AnimationIndices)>) {
    let (mut motion, mut indices) = query.single_mut();

    motion.walking = false;

    let old_direction = motion.direction;

    if keys.pressed(KeyCode::P) || keys.pressed(KeyCode::Right) {
        motion.direction = Direction::Right;
        motion.walking = true;
        indices.sprite = 0;
    }
    if keys.pressed(KeyCode::O) || keys.pressed(KeyCode::Left) {
        motion.direction = Direction::Left;
        motion.walking = true;
        indices.sprite = 1;
    }
    if keys.pressed(KeyCode::Space) {
        motion.direction = Direction::Up;
        motion.jumping = true;
    }

    if old_direction != motion.direction {
        indices.frame = 3 - indices.frame;
    }
}