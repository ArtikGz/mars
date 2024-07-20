use super::{
    block::{self, Block},
    section,
};

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
    let block = if (chunk_pos.x + chunk_pos.z) % 2 == 0 {
        block::STONE
    } else {
        block::DIORITE
    };

    let mut sections = vec![section::ChunkSection::default(); 24];
    sections[0] = section::generate_section(block);

    Chunk {
        position: chunk_pos,
        sections: sections,
    }
}
