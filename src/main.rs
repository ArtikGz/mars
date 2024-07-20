#![allow(dead_code)]

use crate::tcp::AsyncWriteOwnExt;
use std::io;
use tcp::server::start_server;

mod blocks;
mod log;
pub mod nbt;
mod tcp;

type VarInt = u32;

#[derive(Debug, Clone)]
pub struct Position {
    x: i64,
    y: i64,
    z: i64,
}

impl Position {
    pub async fn write_to(&self, writer: &mut impl AsyncWriteOwnExt) -> io::Result<()> {
        /*
        var value int64
        value |= int64((p.X & ) << 38) // x = 26 MSBs
        value |= int64((p.Z & 0x3FFFFFF) << 12) // z = 26 middle bits
        value |= int64(p.Y & 0xFFF)             // y = 12 LSBs

        buffer = make([]byte, 8)
        binary.LittleEndian.PutUint64(buffer, uint64(value))
        return
        */

        let mut value = 0u64;
        value = value | ((self.x & 0x3FFFFFF) << 38) as u64;
        value = value | ((self.z & 0x3FFFFFF) << 12) as u64;
        value = value | (self.y & 0xFFF) as u64;

        writer.write_u64(value).await?;

        Ok(())
    }
}

// TODO: implement ReadFrom and Bytes() from mango

#[tokio::main]
async fn main() {
    start_server("127.0.0.1", "25565").await;
}
