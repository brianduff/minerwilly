use anyhow::Result;
use bevy::prelude::Color;

/// ZX Spectrum color attributes. Consists of an ink value 0-7, a paper
/// value 0-7, and a boolean bright flag.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct Attributes {
  // The ink color index 0..7
  pub ink: u8,
  // The paper color index 0..7
  pub paper: u8,
  // The bright flag
  pub bright: bool,
  // The transparent paper flag. This indicates to the rendering
  // system that the paper should be rendered as a transparent
  // pixel.
  pub transparent_background: bool
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ColorName {
  #[default]
  Black,
  Blue,
  Red,
  Magenta,
  Green,
  Cyan,
  Yellow,
  White,
}

impl From<ColorName> for u8 {
  fn from(value: ColorName) -> Self {
    match value {
      ColorName::Black => 0,
      ColorName::Blue => 1,
      ColorName::Red => 2,
      ColorName::Magenta => 3,
      ColorName::Green => 4,
      ColorName::Cyan => 5,
      ColorName::Yellow => 6,
      ColorName::White => 7,
    }
  }
}

impl Attributes {
  pub fn new(ink: ColorName, paper: ColorName, bright: bool) -> Self {
    Attributes {
      ink: ink.into(),
      paper: paper.into(),
      bright,
      ..Default::default()
    }
  }

  pub fn new_transparent_bg(ink: ColorName, bright: bool) -> Self {
    Attributes {
      ink: ink.into(),
      paper: ColorName::Black.into(),
      bright,
      transparent_background: true
    }
  }

  /// Converts a color into an rgba color value
  fn to_rgba(&self, value: &u8) -> Vec<u8> {
    let code = if self.bright { 0xff } else { 0xee };

    let mut red = 0;
    let mut green = 0;
    let mut blue = 0;
    let alpha = 255;

    if (value & 0b010) != 0 {
      red = code;
    }
    if (value & 0b100) != 0 {
      green = code;
    }
    if (value & 1) != 0 {
      blue = code;
    }
    // TODO: there are a finite number of these - we shouldn't create them over and over
    vec![red, green, blue, alpha]
  }

  /// Returns the rgba representation of the ink of this color.
  pub fn ink_rgba(&self) -> Vec<u8> {
    self.to_rgba(&self.ink)
  }

  /// Returns the rgba representation of the paper of this color.
  pub fn paper_rgba(&self) -> Vec<u8> {
    let mut rgba = self.to_rgba(&self.paper);
    if self.transparent_background {
      rgba[3] = 0;
    }

    rgba
  }

  pub fn ink_color(&self) -> Color {
    convert_color(&self.ink_rgba())
  }

  pub fn paper_color(&self) -> Color {
    convert_color(&self.paper_rgba())
  }


}

fn convert_color(rgba: &[u8]) -> Color {
  Color::Rgba {
    red: rgba[0] as f32 / 255.,
    green: rgba[1] as f32 / 255.,
    blue: rgba[2] as f32 / 255.,
    alpha: rgba[3] as f32  / 255. }
}

impl From<u8> for Attributes {
  fn from(b: u8) -> Self {
    let ink: u8 = b & 0b111;
    let paper: u8 = (b >> 3) & 0b111;
    let bright = ((b >> 6) & 1) == 1;

    Attributes { ink, paper, bright, ..Default::default() }
  }
}

impl TryFrom<&str> for Attributes {
  type Error = anyhow::Error;

  fn try_from(s: &str) -> Result<Self> {
    let value: u8 = u8::from_str_radix(s, 16)?;
    Ok(Attributes::from(value))
  }
}

impl TryFrom<&String> for Attributes {
  type Error = anyhow::Error;

  fn try_from(s: &String) -> Result<Self> {
    Attributes::try_from(s.as_str())
  }
}

impl From<&Attributes> for u8 {
  fn from(color: &Attributes) -> u8 {
    color.ink | (color.paper << 3) | if color.bright { 0b1000000 } else { 0 }
  }
}

#[cfg(test)]
mod tests {
  use super::Attributes;

  #[test]
  fn can_convert_to_u8() {
    assert_eq!(
      u8::from(&Attributes {
        paper: 3,
        ink: 4,
        bright: true,
        ..Default::default()
      }),
      0b1011100
    );
    assert_eq!(
      u8::from(&Attributes {
        paper: 3,
        ink: 4,
        bright: false,
        ..Default::default()
      }),
      0b0011100
    );
    assert_eq!(
      u8::from(&Attributes {
        paper: 7,
        ink: 7,
        bright: true,
        ..Default::default()
      }),
      0b1111111
    );
  }

  #[test]
  fn can_convert_to_rgba() {
    let color = Attributes {
      paper: 5,
      ink: 2,
      bright: true,
      ..Default::default()
    };
    assert_eq!(vec![0xff, 0x00, 0x00, 0xff], color.ink_rgba());
    assert_eq!(vec![0x00, 0xff, 0xff, 0xff], color.paper_rgba());
  }
}
