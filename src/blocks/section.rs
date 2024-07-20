use crate::log;

use super::block;

#[derive(Debug, Clone, Copy)]
pub struct ChunkSection {
    pub blocks: [[[&'static block::Block; 16]; 16]; 16], // [y][z][x]
}

impl Default for ChunkSection {
    fn default() -> Self {
        generate_section(block::AIR)
    }
}

pub fn generate_section(block: &'static block::Block) -> ChunkSection {
    let mut blocks = [[[block::AIR; 16]; 16]; 16];

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                blocks[y][z][x] = block;
            }
        }
    }

    ChunkSection { blocks }
}
