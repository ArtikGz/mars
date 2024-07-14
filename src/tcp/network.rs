use crate::tcp::AsyncReadOwnExt;
use std::io;
use std::task::{Context, Poll};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;

pub async fn next_packet(stream: &mut OwnedReadHalf) -> io::Result<impl io::Read> {
    let packet_len = stream.read_var_int().await?;
    let mut buffer = vec![0; packet_len as usize];

    stream.read_exact(&mut buffer).await?;

    Ok(io::Cursor::new(buffer))
}
