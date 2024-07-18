#![allow(dead_code)]
use tcp::server::start_server;

mod blocks;
mod log;
mod tcp;

type VarInt = u32;

#[derive(Debug, Clone)]
struct Position {
    X: i32,
    Y: i32,
    Z: i32,
}

// TODO: implement ReadFrom and Bytes() from mango

#[tokio::main]
async fn main() {
    start_server("127.0.0.1", "25565").await;
}
