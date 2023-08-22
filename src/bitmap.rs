use crate::color::Attributes;
use bevy::{
  prelude::Image,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// A bitmap is a grid of pixel data of a given width and height
/// and an optional SpectrumColor (attribute data). It can be
/// converted into an image, and the attributes can be changed
/// at that time.
#[derive(Debug, Clone)]
pub struct Bitmap {
  data: Vec<u8>,
  width: usize,
  height: usize,
  pub color: Option<Attributes>,
}

impl Bitmap {
  /// Creates a bitmap of the given width and height from the given slice of data.
  /// It is not expected to contain any attribute data.
  /// The width and height must be a multiple of 8.
  pub fn create(width: usize, height: usize, data: &[u8]) -> Bitmap {
    Bitmap {
      width,
      height,
      color: None,
      data: data.to_vec(),
    }
  }

  /// Creates a bitmap of the given width and height from the given slice of data.
  /// It is expected that the first byte of the data contains attribute data, which
  /// will be used to determine the color the sprite is rendered with.
  pub fn create_with_attributes(width: usize, height: usize, data: &[u8]) -> Bitmap {
    // Read the first byte of attribute data to get the color.
    let color: Attributes = data[0].into();
    Bitmap {
      width,
      height,
      color: Some(color),
      data: data[1..].to_vec(),
    }
  }

  /// Renders this bitmap as an image using its prefered color.
  /// WARNING: If no color is defined, this will panic.
  pub fn render(&self) -> Image {
    self.render_with_color(self.color.as_ref().unwrap())
  }

  fn render_to_rgba(&self, color: &Attributes) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(self.width * self.height * 4);

    let ink_color = color.ink_rgba();
    let paper_color = color.paper_rgba();

    for b in self.data.iter() {
      to_rgba(&mut rgba, b, &ink_color, &paper_color);
    }

    rgba
  }

  /// Renders this sprite to a bevy image using the given color attributes.
  pub fn render_with_color(&self, color: &Attributes) -> Image {
    let data = self.render_to_rgba(color);
    Image::new(
      Extent3d {
        width: self.width as u32,
        height: self.height as u32,
        depth_or_array_layers: 1,
      },
      TextureDimension::D2,
      data,
      TextureFormat::Rgba8Unorm,
    )
  }

  // Creates a new bitmap that's identical to this one, but with all the pixels
  // shifted down by one row. The first row is filled with empty pixels, and the
  // last row is discarded. This is used to create animations for crumbling
  // tiles.
  pub fn shift_down(&self) -> Self {
    // The number of bytes we need to push in at the start depends on the width
    // of the image.
    let pushed_bytes = self.width / 8;

    let mut new_data = Vec::with_capacity(self.data.len());
    new_data.resize(pushed_bytes, 0);
    new_data.extend_from_slice(&self.data[0..self.data.len() - pushed_bytes]);

    Self {
      data: new_data,
      width: self.width,
      height: self.height,
      color: self.color
    }
  }
}

/// Given a byte of bitmap information and an ink and paper color in rgba,
/// extend the given rgba vec to include the rgba pixel data for this byte.
pub fn to_rgba(rgba: &mut Vec<u8>, b: &u8, ink_color: &[u8], paper_color: &[u8]) {
  let mut mask: u8 = 0b10000000;
  for _ in 0..8 {
    rgba.extend_from_slice(if b & mask != 0 {
      ink_color
    } else {
      paper_color
    });
    mask >>= 1;
  }
}
