use anyhow::Result;
use bevy::{
  prelude::*,
  sprite::Anchor,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crate::{color::{SpectrumColor, SpectrumColorName}, position::Layer};
use std::io::Read;
use std::{fs::File, path::Path};

use crate::{handle_errors, position::at_char_pos};

#[derive(Component, Debug)]
pub struct Text {
  pub value: String,
  pub pos: (u8, u8),
  pub attributes: TextAttributes,
  sprite_entity: Option<Entity>
}

impl Text {
  pub fn new(value: &str, pos: (u8, u8), attributes: &TextAttributes) -> Self {
    Text {
      value: value.to_owned(),
      pos,
      attributes: *attributes,
      sprite_entity: None
    }
  }
}

#[derive(Resource, Deref)]
struct CharsetResource(Charset);

#[derive(Default, Debug, Copy, Clone)]
pub struct TextAttributes {
  ink: SpectrumColorName,
  paper: SpectrumColorName,
  bright: bool
}

impl TextAttributes {
  pub fn new(ink: SpectrumColorName, paper: SpectrumColorName) -> Self {
    Self { ink, paper, bright: false }
  }

  pub fn new_bright(ink: SpectrumColorName, paper: SpectrumColorName) -> Self {
    Self { ink, paper, bright: true }
  }
}

fn create_text(charset: &Charset, text: &str, attributes: &TextAttributes) -> Image {
  charset.to_image(&SpectrumColor::new(attributes.ink, attributes.paper, attributes.bright), text)
}

fn tile_sprite(texture: Handle<Image>, pos: (u8, u8)) -> SpriteBundle {
  SpriteBundle {
      sprite: new_top_left_sprite(),
      texture,
      transform: at_char_pos(Layer::Tiles, pos),
      ..default()
  }
}

fn new_top_left_sprite() -> Sprite {
  Sprite { anchor: Anchor::TopLeft, ..default() }
}

pub struct TextPlugin;

impl Plugin for TextPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, load_charset.pipe(handle_errors));
    app.add_systems(Update, render_text);
  }
}

fn render_text(mut commands: Commands, charset: Res<CharsetResource>, mut images: ResMut<Assets<Image>>, mut query: Query<&mut Text, Changed<Text>>) {
  query.for_each_mut(|mut text| {
    // Despawn any previous instance of the text.
    if let Some(entity) = text.sprite_entity {
      commands.entity(entity).despawn();
      text.sprite_entity = None;
    }

    let image_handle = images.add(create_text(&charset, &text.value, &text.attributes));
    let id = commands.spawn(tile_sprite(image_handle, text.pos)).id();
    text.sprite_entity = Some(id);
  });
}

fn load_charset(mut commands: Commands) -> Result<()> {
  commands.insert_resource(CharsetResource(Charset::load("assets/charset.bin")?));

  Ok(())
}

struct Charset {
  bytes: Vec<u8>,
}

impl Charset {
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let mut f = File::open(path)?;
    let mut bytes = vec![0; 768];
    f.read_exact(&mut bytes)?;

    Ok(Self { bytes })
  }

  /// Converts the result of to_rgba into a Bevy Image.
  fn to_image(&self, color: &SpectrumColor, text: &str) -> Image {
    let data = self.to_rgba(color, text);
    Image::new(
      Extent3d {
        width: text.len() as u32 * 8,
        height: 8,
        depth_or_array_layers: 1,
      },
      TextureDimension::D2,
      data,
      TextureFormat::Rgba8Unorm,
    )
  }

  /// Given some text, return rgba data containing that text with the given
  /// paper and ink color. Any characters in `text` that are not in the ascii
  /// range 32-127 will be rendered as spaces.
  fn to_rgba(&self, color: &SpectrumColor, text: &str) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(8 * 4 * text.len());

    let ink_rgba = &color.ink_rgba()[0..4];
    let paper_rgba = &color.paper_rgba()[0..4];

    let text_bytes = text.as_bytes();

    for r in 0..=7 {
      (0..text.len()).for_each(|c| {
        let charcode = if text_bytes[c] < 32 || text_bytes[c] > 127 {
          32
        } else {
          text_bytes[c]
        };

        let char_index = charcode - 32;
        let offset: usize = char_index as usize * 8;
        let bitmap = &self.bytes[offset..offset + 8];
        crate::bitmap::to_rgba(&mut buffer, &bitmap[r], ink_rgba, paper_rgba);
      });
    }

    buffer
  }
}

#[cfg(test)]
mod tests {
  use std::io::BufWriter;

  use super::*;

  #[test]
  fn to_rgba_works() -> Result<()> {
    let charset = Charset::load("assets/textures/charset.bin")?;
    let text = "         Central Cavern         ";
    let rgba = charset.to_rgba(
      &SpectrumColor {
        ink: 0,
        paper: 6,
        bright: false,
        ..Default::default()
      },
      text,
    );

    let file = File::create("/tmp/chars.png")?;
    let w = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, text.len() as u32 * 8, 8);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&rgba)?;

    Ok(())
  }
}
