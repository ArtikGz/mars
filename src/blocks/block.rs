#[derive(Clone, Copy)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub id: i32,
    pub name: &'static str,
}

// TODO: implement all the blocks
pub static AIR: Block = Block {
    id: 0,
    name: "minecraft:air",
};
pub static STONE: Block = Block {
    id: 1,
    name: "minecraft:stone",
};
pub static DIORITE: Block = Block {
    id: 5,
    name: "minecraft:diorite",
};
