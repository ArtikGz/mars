use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    blocks::{
        block, chunk,
        section::{self, ChunkSection},
    },
    VarInt,
};

use super::packet::{NetworkChunkPos, NetworkChunkSection, PalettedContainer, S2c};

pub fn map_chunk_to_packet(chunk: Arc<chunk::Chunk>) -> S2c {
    let position = NetworkChunkPos {
        x: chunk.position.x,
        z: chunk.position.z,
    };

    let mut sections = vec![];
    for section in chunk.sections.iter() {
        sections.push(map_chunk_section(section));
    }

    S2c::ChunkDataAndLight { position, sections }
}

fn map_chunk_section(chunk_section: &section::ChunkSection) -> NetworkChunkSection {
    let mut block_palette = HashSet::new();
    let mut non_air_blocks = 0;

    for i in 0..16 {
        for j in 0..16 {
            for z in 0..16 {
                let block = chunk_section.blocks[i][j][z];
                block_palette.insert(block.id as VarInt);

                if block != block::AIR {
                    non_air_blocks += 1;
                }
            }
        }
    }

    let bits_per_entry = match block_palette.len() {
        1 => 0,
        v if v <= 16 => 4,
        v if v <= 32 => 5,
        v if v <= 64 => 6,
        v if v <= 128 => 7,
        v if v <= 256 => 8,
        _ => 15,
    };

    let biomes = PalettedContainer {
        bits_per_entry: 0,
        palette: {
            let mut set = HashSet::new();
            set.insert(55);

            set
        },
        data: vec![],
    };

    let data = generate_data_vec(chunk_section, bits_per_entry, &block_palette);
    let block_states = PalettedContainer {
        bits_per_entry,
        palette: block_palette,
        data,
    };

    NetworkChunkSection {
        non_air_blocks,
        block_states,
        biomes,
    }
}

fn generate_data_vec(
    section: &section::ChunkSection,
    bits_per_entry: u8,
    palette: &HashSet<VarInt>,
) -> Vec<u64> {
    if bits_per_entry == 0 {
        return vec![];
    }

    let mut map = HashMap::new();

    for (i, entry_id) in palette.iter().enumerate() {
        if bits_per_entry == 15 {
            map.insert(*entry_id, *entry_id);
        } else {
            map.insert(*entry_id, i as VarInt);
        }
    }

    let entries_per_long = (64 / bits_per_entry) as u32;
    let output_size = (16 * 16 * 16 + entries_per_long - 1) / entries_per_long;
    let mut output = vec![0u64; output_size as usize];

    let mut i = 0;
    let mut offset = 0;

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let id = section.blocks[y][z][x].id;
                let value = *map.get(&(id as VarInt)).unwrap();

                let bits = extract_bits(value, bits_per_entry);

                output[i] |= bits << offset;

                offset += bits_per_entry;
                if offset + bits_per_entry > 64 {
                    offset = 0;
                    i += 1;
                }
            }
        }
    }

    output
}

fn extract_bits(value: u32, bits: u8) -> u64 {
    (value & ((1 << bits) - 1)) as u64
}
