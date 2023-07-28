use crate::color::SpectrumColor;
use anyhow::Result;

#[derive(Debug)]
pub struct Sprite {
  pixel_width: usize,
  pixel_height: usize,
  bytes: Vec<u8>,
  pub color: SpectrumColor,
}

impl Sprite {
  pub fn try_from_bytes(pixel_width: usize, pixel_height: usize, bytes: &[u8]) -> Result<Self> {
    let bytes_per_row = pixel_width / 8;
    let rows = pixel_height;

    let expected_bytes = 1 + (rows * bytes_per_row); // +1 for attributes (color)
    anyhow::ensure!(
      bytes.len() == expected_bytes,
      "Expected {} bytes for a {}x{} sprite",
      expected_bytes,
      pixel_width,
      pixel_height
    );

    let color = SpectrumColor::try_from(bytes[0])?;
    let bytes = bytes[1..].to_owned();

    Ok(Sprite {
      pixel_width,
      pixel_height,
      bytes,
      color,
    })
  }

  /// Converts this sprite into 2d rgba data.
  pub fn to_rgba(&self) -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(self.pixel_height);

    let mut byte_iter = self.bytes.iter();
    for _ in 0..self.pixel_height {
      let mut cols: Vec<u8> = Vec::with_capacity(self.pixel_width);
      for _ in 0..self.pixel_width / 8 {
        let byte = byte_iter.next().unwrap();
        let mut mask: u8 = 0b10000000;
        for _ in 0..8 {
          cols.append(&mut if byte & mask != 0 {
            self.color.ink_rgba()
          } else {
            self.color.paper_rgba()
          });
          mask >>= 1;
        }
      }
      result.push(cols);
    }

    result
  }
}
