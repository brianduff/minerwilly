use anyhow::Result;

use crate::{bitmap::Bitmap, color::Attributes};

// #[derive(Debug)]
// pub enum SpecialBehavior {
//   SolarPower,
//   KongBeast,
//   VerticalGuardians,
//   SkylabVerticalGuardians,
//   Eugene
// }

// A cavern
#[derive(Debug)]
pub struct Cavern {
  pub layout: Layout,
  pub name: String,
  pub tile_bitmaps: Vec<Bitmap>,
  pub border_color: Attributes,
  pub portal: Portal,
  pub guardians: Vec<Guardian>,
  pub guardian_bitmaps: Vec<Bitmap>,
  // pub special_behaviors: HashSet<SpecialBehavior>,
  pub items: Vec<Item>,
  pub item_bitmap: Bitmap
}

/// There are eight types of cavern tiles.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CavernTileType {
  Background = 0,
  Floor = 1,
  CrumblingFloor = 2,
  Wall = 3,
  Conveyor = 4,
  Nasty1 = 5,
  Nasty2 = 6,
  Extra = 7,
}

impl From<usize> for CavernTileType {
  fn from(value: usize) -> Self {
    match value {
      0 => CavernTileType::Background,
      1 => CavernTileType::Floor,
      2 => CavernTileType::CrumblingFloor,
      3 => CavernTileType::Wall,
      4 => CavernTileType::Conveyor,
      5 => CavernTileType::Nasty1,
      6 => CavernTileType::Nasty2,
      7 => CavernTileType::Extra,
      _ => CavernTileType::Background,
    }
  }
}

impl CavernTileType {
  pub fn can_stand(&self) -> bool {
    matches!(
      *self,
      CavernTileType::Floor
        | CavernTileType::CrumblingFloor
        | CavernTileType::Conveyor
        | CavernTileType::Wall
        | CavernTileType::Extra
    )
  }

  pub fn is_nasty(&self) -> bool {
    matches!(
      *self,
      CavernTileType::Nasty1 | CavernTileType::Nasty2
    )
  }
}

impl Cavern {
  // pub fn get_tile_type(&self, pos: (u8, u8)) -> CavernTileType {
  //   self.get_bg_sprite_index(pos).unwrap_or(0).into()
  // }

  pub fn get_bg_sprite_index(&self, (char_x, char_y): (u8, u8)) -> Option<usize> {
    let color = self.layout.get_cell_color(char_x, char_y);

    for (i, s) in self.tile_bitmaps.iter().enumerate() {
      if s.color.as_ref().unwrap().eq(color) {
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

    let mut tile_bitmaps = Vec::with_capacity(8);
    let mut pos = 544;
    for _ in 0..8 {
      let end = pos + 9;
      tile_bitmaps.push(Bitmap::create_with_attributes(8, 8, &bytes[pos..end]));
      pos = end;
    }

    let border_color = Attributes::try_from(bytes[627])?;

    let portal: Portal = Portal::try_from(&bytes[655..692])?;

    // Read the guardians, starting at offset 702.
    let mut guardians = Vec::with_capacity(4);
    let mut offset = 702;

    while bytes[offset] != 255 {
      guardians.push(bytes[offset..offset + 7].try_into()?);
      offset += 7;
    }

    let mut guardian_bitmaps = Vec::with_capacity(8);
    offset = 768;
    for _ in 0..8 {
      guardian_bitmaps.push(Bitmap::create(16, 16, &bytes[offset..offset + 32]));
      offset += 32;
    }

    // Read items, starting at offset 629.
    // TODO: generalize this "reading a list of things" sequence.
    let mut items = Vec::with_capacity(5);
    let mut offset = 629;

    while bytes[offset] != 255 {
      items.push(bytes[offset..offset + 5].into());
      offset += 5;
    }


    let item_bitmap = Bitmap::create(8, 8, &bytes[692..=699]);

    Ok(Cavern {
      layout,
      name,
      tile_bitmaps,
      border_color,
      portal,
      guardians,
      guardian_bitmaps,
      // special_behaviors: HashSet::new(),
      items,
      item_bitmap
    })
  }
}

/// The layout of a cavern - a 32x16 grid of 8x8 pixel squares.
/// Each square is represented by a color attribute, and in turn
/// these color attributes index into background tile sprites for
/// the cavern.
#[derive(Debug)]
pub struct Layout {
  cells: Vec<Attributes>,
}

impl Layout {
  fn get_cell_color(&self, char_x: u8, char_y: u8) -> &Attributes {
    &self.cells[(char_y as usize * 32) + char_x as usize]
  }
}

impl TryFrom<&[u8]> for Layout {
  type Error = anyhow::Error;

  fn try_from(bytes: &[u8]) -> Result<Layout> {
    anyhow::ensure!(bytes.len() == 512, "Expected 512 bytes");

    let mut cells: Vec<Attributes> = Vec::with_capacity(512);

    for byte in bytes {
      cells.push(Attributes::try_from(*byte)?)
    }

    Ok(Layout { cells })
  }
}

#[derive(Debug, Clone)]
pub struct Guardian {
  pub attributes: Attributes,
  pub start_pos: (u8, u8),
  pub first_animation_frame: u8,
  pub left_bound: u8,
  pub right_bound: u8,
  pub speed: GuardianSpeed,
}

impl TryFrom<&[u8]> for Guardian {
  type Error = anyhow::Error;

  fn try_from(guardian_data: &[u8]) -> Result<Guardian> {
    anyhow::ensure!(guardian_data.len() == 7, "Expected 7 bytes");

    let speed = if guardian_data[0] & 0b10000000 == 0 {
      GuardianSpeed::Normal
    } else {
      GuardianSpeed::Fast
    };

    let mut attributes: Attributes = (guardian_data[0] & 0b01111111).into();
    attributes.transparent_background = true;

    let encoded_pos: u32 = guardian_data[1] as u32
      | ((guardian_data[2] as u32) << 8)
      | ((guardian_data[3] as u32) << 16);

    let x = (encoded_pos & 0b11111) as u8;
    let y = ((encoded_pos & 0b111100000) >> 5) as u8;
    let start_pos = (x, y);

    let first_animation_frame = guardian_data[4];
    let left_bound = guardian_data[5] & 0b11111;
    let right_bound = guardian_data[6] & 0b11111;

    Ok(Guardian {
      attributes,
      start_pos,
      first_animation_frame,
      left_bound,
      right_bound,
      speed,
    })
  }
}

#[derive(Debug, Copy, Clone)]
pub enum GuardianSpeed {
  Normal,
  Fast,
}

#[derive(Debug)]
pub struct Portal {
  pub attributes: Attributes,
  pub bitmap: Bitmap,
  pub position: (u8, u8),
}

impl TryFrom<&[u8]> for Portal {
  type Error = anyhow::Error;

  fn try_from(data: &[u8]) -> Result<Portal> {
    anyhow::ensure!(data.len() == 37, "Expected 37 bytes");

    let attributes: Attributes = data[0].into();
    let bitmap = Bitmap::create(16, 16, &data[1..33]);

    let position = decode_packed_position(&data[33..=34]);

    Ok(Portal {
      attributes,
      bitmap,
      position
    })
  }
}

fn decode_packed_position(bytes: &[u8]) -> (u8, u8) {
  let pos = (bytes[1] as u16) << 8 | bytes[0] as u16;
  let x = (pos & 0b11111) as u8;
  let y = ((pos & 0b111100000) >> 5) as u8;

  (x, y)
}

#[derive(Debug, Copy, Clone)]
pub struct Item {
  pub attributes: Attributes,
  pub position: (u8, u8),
}

impl From<&[u8]> for Item {
  fn from(data: &[u8]) -> Item {
    Item {
      attributes: data[0].into(),
      position: decode_packed_position(&data[1..=2])
    }
  }
}