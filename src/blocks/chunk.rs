use super::{
    block::{self, Block},
    perlin::{self, PerlinNoiseGenerator},
    section,
};

use lazy_static::lazy_static;

lazy_static! {
    static ref terrain_generator: PerlinNoiseGenerator = PerlinNoiseGenerator::new(312312890);
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl From<block::BlockPos> for ChunkPos {
    fn from(pos: block::BlockPos) -> Self {
        ChunkPos {
            x: pos.x >> 4,
            z: pos.z >> 4,
        }
    }
}

pub struct Chunk {
    pub position: ChunkPos,
    pub sections: Vec<section::ChunkSection>,
}

impl Chunk {
    pub fn get_block(&self, pos: block::BlockPos) -> Option<&'static Block> {
        let (x, y, z) = (
            (pos.x & 15) as usize,
            (pos.y & 15) as usize,
            (pos.z & 15) as usize,
        );

        self.get_section(pos)
            .and_then(|section| section.blocks.get(y))
            .and_then(|rest| rest.get(z))
            .and_then(|rest| rest.get(x))
            .and_then(|&rest| Some(rest))
    }

    pub fn get_section(&self, pos: block::BlockPos) -> Option<&section::ChunkSection> {
        self.sections.get((4 + pos.y >> 4) as usize)
    }
}

pub fn generate_chunk(chunk_pos: ChunkPos) -> Chunk {
    let scale = 100.0;
    let mut perlin_values = vec![];
    for z in 0..16 {
        for x in 0..16 {
            let xpos = ((chunk_pos.x * 16 + x) as f64) / scale;
            let zpos = ((chunk_pos.z * 16 + z) as f64) / scale;

            perlin_values.push(16.0 * 7.0 + terrain_generator.get_height_for(xpos, zpos) * 30.0);
        }
    }

    let mut sections = vec![];
    for section_y in 0..24 {
        let mut blocks = [[[block::AIR; 16]; 16]; 16];

        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let perlin_value = perlin_values[16 * z + x];

                    let abs_y = section_y * 16 + y;
                    if abs_y < perlin_value.floor() as usize {
                        if abs_y < (perlin_value.floor() as usize) - 5 {
                            blocks[y][z][x] = block::STONE;
                        } else if abs_y < (perlin_value.floor() as usize) - 1 {
                            blocks[y][z][x] = block::DIRT;
                        } else if abs_y < perlin_value.floor() as usize {
                            blocks[y][z][x] = block::GRASS_BLOCK;
                        }
                    }
                }
            }
        }

        sections.push(section::ChunkSection { blocks });
    }

    Chunk {
        position: chunk_pos,
        sections,
    }
}
