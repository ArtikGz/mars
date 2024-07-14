use tcp::server::start_server;

mod blocks;
mod log;
mod tcp;

type VarInt = u32;

#[tokio::main]
async fn main() {
    start_server("127.0.0.1", "25565").await;
}
