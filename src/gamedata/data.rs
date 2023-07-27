use std::{fs::File, io::SeekFrom, path::Path};

use anyhow::Result;
use std::io::{Read, Seek};

use crate::bitmap::Bitmap;

use super::cavern::Cavern;

const WILLY_SPRITE_OFFSET: u64 = 0x8200;
const WILLY_SPRITE_SIZE_BYTES: usize = 8 * 4;

const CAVERNS_OFFSET: u64 = 0xb000;
const CAVERN_COUNT: usize = 20;
const CAVERN_DATA_SIZE_BYTES: usize = 1024;

#[derive(Debug)]
pub struct GameData {
    pub caverns: Vec<Cavern>,
    pub willy_sprites: Vec<Bitmap>,
}

impl GameData {
    /// Load game data from a manic miner binary file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;

        let willy_sprites = extract_willy_sprites(&file)?;
        let caverns = extract_caverns(&file)?;


        Ok(Self { caverns, willy_sprites })
    }

    pub fn cavern_tiles_rgba(&self) -> Result<Vec<u8>> {
        let sprites: Vec<_> = self.caverns.iter().flat_map(|c| &c.tile_sprites).collect();
        let columns = 16;

        // Fixme - hardcoded assumption that eash sprite is 8x8
        let width = 8 * columns;
        let rows = (sprites.len() / columns) + 1;
        let height = 8 * rows;

        let mut merged = Vec::with_capacity(width * height * 4);

        for pixel_row in 0..height {
            for pixel_col in 0..width {
                let block_col = pixel_col / 8;
                let block_row = pixel_row / 8;
                let sprite_num = (block_row * columns) + block_col;

                let sprite_row = pixel_row % 8;
                let sprite_col = pixel_col % 8;

                if sprite_num < sprites.len() {
                    merged.push(sprites[sprite_num].to_rgba()[sprite_row][sprite_col * 4]);
                    merged.push(sprites[sprite_num].to_rgba()[sprite_row][sprite_col * 4 + 1]);
                    merged.push(sprites[sprite_num].to_rgba()[sprite_row][sprite_col * 4 + 2]);
                    merged.push(sprites[sprite_num].to_rgba()[sprite_row][sprite_col * 4 + 3]);
                }
            }
        }

        let expected_count = width * height * 4;

        merged.resize(expected_count, 0);

        Ok(merged)
    }
}

fn extract_willy_sprites(mut file: &File) -> Result<Vec<Bitmap>> {
    let mut result = Vec::with_capacity(8);
    let mut buffer = vec![0; WILLY_SPRITE_SIZE_BYTES];

    let mut pos = WILLY_SPRITE_OFFSET;
    for _ in 0..8 {
        file.seek(SeekFrom::Start(pos))?;
        file.read_exact(&mut buffer)?;

        result.push(Bitmap::create(16, 16, &buffer));
        pos += WILLY_SPRITE_SIZE_BYTES as u64;
    }


    Ok(result)
}

fn extract_caverns(mut file: &File) -> Result<Vec<Cavern>> {
    let mut buf = vec![0; CAVERN_DATA_SIZE_BYTES];

    file.seek(SeekFrom::Start(CAVERNS_OFFSET))?;
    let mut caverns = Vec::with_capacity(CAVERN_COUNT);
    for _ in 0..CAVERN_COUNT {
        file.read_exact(&mut buf)?;
        let cavern = Cavern::try_from(&buf[..])?;
        caverns.push(cavern);
    }
    Ok(caverns)
}
