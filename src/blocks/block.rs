use std::hash::Hash;

#[derive(Clone, Copy)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub id: u32,
    pub name: &'static str,
}

impl Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id);
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Block {}

// TODO: implement all the blocks
pub static AIR: &'static Block = &Block {
    id: 0,
    name: "minecraft:air",
};
pub static STONE: &'static Block = &Block {
    id: 1,
    name: "minecraft:stone",
};
pub static DIORITE: &'static Block = &Block {
    id: 5,
    name: "minecraft:diorite",
};
