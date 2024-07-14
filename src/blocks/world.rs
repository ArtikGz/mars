use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use tokio::sync::Mutex;

use super::{
    block,
    chunk::{self, ChunkPos},
};

pub fn get_world() -> &'static Arc<Mutex<World>> {
    static WORLD_INSTANCE: OnceLock<Arc<Mutex<World>>> = OnceLock::new();

    WORLD_INSTANCE.get_or_init(|| Arc::new(Mutex::new(World::default())))
}

#[derive(Default)]
pub struct World {
    pub chunks: HashMap<chunk::ChunkPos, chunk::Chunk>,
}

impl World {
    pub fn get_chunk(&mut self, pos: &chunk::ChunkPos) -> Option<&chunk::Chunk> {
        if !self.chunks.get(pos).is_some() {
            let new_chunk = chunk::generate_chunk(*pos);
            self.set_chunk(new_chunk);
        }

        self.chunks.get(pos)
    }

    pub fn set_chunk(&mut self, chunk: chunk::Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn get_block(&self, pos: block::BlockPos) -> Option<&block::Block> {
        let chunk_pos: ChunkPos = pos.into();
        let chunk = self.chunks.get(&chunk_pos);

        chunk.and_then(|chunk| chunk.get_block(pos))
    }
}
