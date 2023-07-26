use bevy::{prelude::*, sprite::Anchor};
use gamedata::{GameDataPlugin, CavernTexture, GameDataResource};
use minerdata::{color::SpectrumColor, color::SpectrumColorName};
use anyhow::Result;
use text::Charset;

mod cavern;
mod gamedata;
mod text;

static SCALE: f32 = 2.0;
static CELLSIZE: f32 = 8.0 * SCALE;
static TIMER_TICK: f32 = 0.075;
// The number of frames in a sprite animation
static FRAME_COUNT: usize = 4;
static BORDER_WIDTH_CHARS: f32 = 4.;

static DISPLAY_SCREEN_WIDTH_CH: f32 = 32.;
static DISPLAY_SCREEN_HEIGHT_CH: f32 = 24.;
static BORDER_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * BORDER_WIDTH_CHARS;
static SCREEN_WIDTH_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_WIDTH_CH;
static WINDOW_WIDTH_PX : f32  = SCREEN_WIDTH_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static SCREEN_HEIGHT_PX: f32 = SCALE * PIX_PER_CHAR * DISPLAY_SCREEN_HEIGHT_CH;
static WINDOW_HEIGHT_PX : f32 = SCREEN_HEIGHT_PX + (BORDER_WIDTH_PX * BORDER_MUL);
static PIX_PER_CHAR: f32 = 8.;
static BORDER_MUL: f32 = 2.;

/// Converts the ink of the given SpectrumColor into a bevy Color
fn ink_to_color(spectrum_color: &SpectrumColor) -> Color {
    let spectrum_rgba = spectrum_color.ink_rgba();
    Color::Rgba { red: spectrum_rgba[0] as f32 / 255., green: spectrum_rgba[1] as f32 / 255., blue: spectrum_rgba[2] as f32 / 255., alpha: spectrum_rgba[3] as f32  / 255. }
}

pub fn handle_errors(In(result): In<Result<()>>) {
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}

fn main() -> Result<()>  {

    let window = Window {
        title: "Miner Willy".into(),
        resolution: (WINDOW_WIDTH_PX, WINDOW_HEIGHT_PX).into(),
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.
            set(ImagePlugin::default_nearest())
            .set(WindowPlugin { primary_window: Some(window), ..default() })) // prevents blurry sprites
        .add_plugins(GameDataPlugin)
        .add_systems(PostStartup, setup)
        .add_systems(Update, (animate_sprite, check_keyboard, create_level, check_debug_keyboard))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .run();

    Ok(())
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

#[derive(Resource, Debug)]
struct SpriteSheets {
    willy_sprites: Handle<Image>,
}

#[derive(Resource, Debug)]
struct Levels {
    current_cavern: usize
}

#[derive (Component, Debug)]
struct CavernTile {
    x: u8,
    y: u8,
}


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut clear_color: ResMut<ClearColor>,
)  {
    println!("Setup called");
    let texture_handle = asset_server.load("textures/willysprites.png");

    let sprite_sheets = SpriteSheets { willy_sprites: texture_handle };

    let texture_handle = asset_server.load("textures/willysprites.png");

    let current_cavern = 0;
    let levels = Levels{ current_cavern };

    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 16, 9, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { sprite: 2, frame: 0 };
    let willy_motion = WillyMotion { walking: false, direction: Direction::Right, jumping: false };
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(sprite_sheets);
    commands.insert_resource(levels);

    // Spawn Willy
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


    let charset = Charset::load("assets/charset.bin").unwrap();

    // let cavern_name_handle = images.add(create_text(
    //     &charset,
    //     &game_data.caverns[current_cavern].name,
    //     SpectrumColorName::Black, SpectrumColorName::Yellow, false));
    // commands.spawn(tile_sprite(cavern_name_handle, (0, 16)));

    let air_bar_red_handle = images.add(create_text(
        &charset,
        "AIR       ",
        SpectrumColorName::White, SpectrumColorName::Red, true
    ));
    commands.spawn(tile_sprite(air_bar_red_handle, (0, 17)));

    let air_bar_green_handle = images.add(create_text(
        &charset,
        "                       ",
        SpectrumColorName::White, SpectrumColorName::Green, true
    ));
    commands.spawn(tile_sprite(air_bar_green_handle, (9, 17)));

    let high_score_image_handle = images.add(create_text(
        &charset,
        "High Score 000000   Score 000000",
        SpectrumColorName::Yellow, SpectrumColorName::Black, false));
    commands.spawn(tile_sprite(high_score_image_handle, (0, 19)));
}



fn setup2(sprite_sheets: Res<SpriteSheets>,) {
    println!("Setup 2 called!");

    println!("Sprite sheets is {:?}", sprite_sheets);
}


fn create_text(charset: &Charset, text: &str, ink: SpectrumColorName, paper: SpectrumColorName, bright: bool) -> Image {
    charset.to_image(&SpectrumColor::new(ink, paper, bright), text)
}

fn tile_sprite(texture: Handle<Image>, pos: (u8, u8)) -> SpriteBundle {
    SpriteBundle {
        sprite: new_top_left_sprite(),
        texture,
        transform: at_char_pos(pos),
        ..default()
    }
}

fn new_top_left_sprite() -> Sprite {
    Sprite { anchor: Anchor::TopLeft, ..default() }
}

fn new_transform() -> Transform {
    Transform::from_scale(Vec3::splat(SCALE))
}

fn at_char_pos(pos: (u8, u8)) -> Transform {
    let (screen_x, screen_y) = char_pos_to_screen(pos);
    new_transform().with_translation(Vec3 { x: screen_x, y: screen_y, z: 0. })
}

// Converts a character position on screen to the top left screen
// coordinate that contains that character.
fn char_pos_to_screen((x, y): (u8, u8)) -> (f32, f32) {
    let x : f32 = x.into();
    let y : f32 = y.into();
    let pos_x = 0.0 - (SCREEN_WIDTH_PX / 2.) + (8. * x * SCALE);
    let pos_y = 0.0 + (SCREEN_HEIGHT_PX / 2.) - (8. * y * SCALE);

    (pos_x, pos_y)
}

fn create_level(
    mut commands: Commands,
    levels: Res<Levels>,
    game_data: Res<GameDataResource>,
    textures: Res<CavernTexture>,
    mut clear_color: ResMut<ClearColor>,
    query: Query<Entity, With<CavernTile>>,
) {
    if levels.is_changed() {
        // Despawn any existing cavern tiles.
        query.for_each(|entity| {
            commands.entity(entity).despawn();
        });

        let current_cavern = levels.current_cavern;
        let cavern = &game_data.caverns[current_cavern];
        println!("Current cavern is {:?}", current_cavern);
        let border_color = ink_to_color(&cavern.border_color);
        clear_color.0 = border_color;

        for y in 0..16 {
            for x in 0..32 {
                let sprite_index = cavern.get_bg_sprite_index(x.into(), y.into());
                let tile = CavernTile { x, y };
                if let Some(sprite_index) = sprite_index {
                    commands.spawn((
                        tile,
                        SpriteSheetBundle {
                            texture_atlas: textures.clone(),
                            sprite: TextureAtlasSprite {
                                index: (current_cavern * 8) + sprite_index,
                                anchor: Anchor::TopLeft,
                                ..default() },
                            transform: at_char_pos((x, y)),
                            ..default()
                        },
                    ));
                }
                // pos_x += 8.0 * SCALE;
            }
        // pos_x = start_x;
        // pos_y -= 8.0 * SCALE;
        }
    }
}


fn animate_sprite(
    time: Res<Time>,
    images: Res<Assets<Image>>,
    sprite_sheets: Res<SpriteSheets>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
        &WillyMotion,
    )>,
) {
    let (mut indices, mut timer, mut sprite, mut transform, motion) = query.single_mut();

    //let image = images.get(&sprite_sheets.willy_sprites);

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

fn check_debug_keyboard(keys: Res<Input<KeyCode>>, mut levels: ResMut<Levels>) {
    if keys.just_released(KeyCode::BracketRight) && levels.current_cavern < 19 {
        levels.current_cavern += 1;
    } else if keys.just_released(KeyCode::BracketLeft) && levels.current_cavern > 0 {
        levels.current_cavern -= 1;
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