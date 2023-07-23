use std::{path::Path, fs::File};
use anyhow::Result;
use bevy::{prelude::Image, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};
use minerdata::color::SpectrumColor;
use std::io::Read;

pub struct Charset {
  bytes: Vec<u8>
}

impl Charset {
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let mut f = File::open(path)?;
    let mut bytes = vec![0;768];
    f.read_exact(&mut bytes)?;

    Ok(Self { bytes })
  }

  /// Converts the result of to_rgba into a Bevy Image.
  pub fn to_image(&self, color: &SpectrumColor, text: &str) -> Image {
    let data = self.to_rgba(color, text);
    Image::new(Extent3d { width: text.len() as u32 * 8, height: 8, depth_or_array_layers: 1 },
        TextureDimension::D2, data, TextureFormat::Rgba8Unorm)
  }

  /// Given some text, return rgba data containing that text with the given
  /// paper and ink color. Any characters in `text` that are not in the ascii
  /// range 32-127 will be rendered as spaces.
  pub fn to_rgba(&self, color: &SpectrumColor, text: &str) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(8*4*text.len());

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
        let mut mask: u8 = 0b10000000;
        for _ in 0..=7 {
          let ink = (bitmap[r] & mask) != 0;
          buffer.extend_from_slice(if ink {
            ink_rgba
          } else {
            paper_rgba
          });
          mask >>= 1;
        }
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
    let rgba = charset.to_rgba(&SpectrumColor { ink: 0, paper: 6, bright: false }, text);

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
