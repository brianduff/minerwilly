use crate::color::SpectrumColor;
use bevy::{
  prelude::Image,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// A bitmap is a grid of pixel data of a given width and height
/// and an optional SpectrumColor (attribute data). It can be
/// converted into an image, and the attributes can be changed
/// at that time.
#[derive(Debug)]
pub struct Bitmap {
  data: Vec<u8>,
  width: usize,
  height: usize,
  color: Option<SpectrumColor>,
}

impl Bitmap {
  /// Creates a bitmap of the given width and height from the given slice of data.
  /// It is not expected to contain any attribute data.
  /// The width and height must be a multiple of 8.
  pub fn create(width: usize, height: usize, data: &[u8]) -> Bitmap {
    println!("Bitmap with data: {:?}", data);
    Bitmap {
      width,
      height,
      color: None,
      data: data.to_vec(),
    }
  }

  /// Renders this bitmap as an image using its prefered color. If no color is
  /// defined, this will panic.
  pub fn render(&self) -> Image {
    self.render_with_color(self.color.as_ref().unwrap())
  }

  fn render_to_rgba(&self, color: &SpectrumColor) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(self.width * self.height * 4);

    let ink_color = color.ink_rgba();
    let paper_color = color.paper_rgba();

    for b in self.data.iter() {
      to_rgba(&mut rgba, b, &ink_color, &paper_color);
    }

    rgba
  }

  /// Renders this sprite to a bevy image using the given color attributes.
  pub fn render_with_color(&self, color: &SpectrumColor) -> Image {
    let data = self.render_to_rgba(color);
    println!("Rendering image with color: {:?}: \n{:?}", color, data);
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
