use anyhow::Result;

use crate::color::SpectrumColor;

use super::sprite::Sprite;

// A cavern
#[derive(Debug)]
pub struct Cavern {
  pub layout: Layout,
  pub name: String,
  pub tile_sprites: Vec<Sprite>,
  pub border_color: SpectrumColor,
}

impl Cavern {
  pub fn get_bg_sprite_index(&self, char_x: usize, char_y: usize) -> Option<usize> {
    let color = self.layout.get_cell_color(char_x, char_y);

    for (i, s) in self.tile_sprites.iter().enumerate() {
      if *color == s.color {
        return Some(i);
      }
    }

    println!(
      "({}, {}): {} - SPRITE NOT FOUND",
      char_x,
      char_y,
      u8::from(color)
    );

    None
  }
}

impl TryFrom<&[u8]> for Cavern {
  type Error = anyhow::Error;

  fn try_from(bytes: &[u8]) -> Result<Cavern> {
    anyhow::ensure!(bytes.len() == 1024, "Expected 1024 bytes");

    let layout = Layout::try_from(&bytes[0..512])?;
    let name = core::str::from_utf8(&bytes[512..544])?.to_owned();

    let mut tile_sprites = Vec::with_capacity(8);
    let mut pos = 544;
    for _ in 0..8 {
      let end = pos + 9;
      tile_sprites.push(Sprite::try_from_bytes(8, 8, &bytes[pos..end])?);
      pos = end;
    }

    let border_color = SpectrumColor::try_from(&bytes[627])?;

    Ok(Cavern {
      layout,
      name,
      tile_sprites,
      border_color,
    })
  }
}

/// The layout of a cavern - a 32x16 grid of 8x8 pixel squares.
/// Each square is represented by a color attribute, and in turn
/// these color attributes index into background tile sprites for
/// the cavern.
#[derive(Debug)]
pub struct Layout {
  cells: Vec<SpectrumColor>,
}

impl Layout {
  fn get_cell_color(&self, char_x: usize, char_y: usize) -> &SpectrumColor {
    &self.cells[(char_y * 32) + char_x]
  }
}

impl TryFrom<&[u8]> for Layout {
  type Error = anyhow::Error;

  fn try_from(bytes: &[u8]) -> Result<Layout> {
    anyhow::ensure!(bytes.len() == 512, "Expected 512 bytes");

    let mut cells: Vec<SpectrumColor> = Vec::with_capacity(512);

    for byte in bytes {
      cells.push(SpectrumColor::try_from(byte)?)
    }

    Ok(Layout { cells })
  }
}
