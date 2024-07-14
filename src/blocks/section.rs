use super::block;

#[derive(Debug, Clone, Copy)]
pub struct ChunkSection {
    pub blocks: [[[block::Block; 16]; 16]; 16] // [y][z][x]
}

impl Default for ChunkSection {
    fn default() -> Self {
        generate_section(block::AIR)
    }
}


pub fn generate_section(block: block::Block) -> ChunkSection {
    let mut section = ChunkSection::default();

    for y in 0..16  {
        for z in 0..16 {
            for x in 0..16 {
                section.blocks[y][z][x] = block.clone();
            }
        }
    }

    section
}