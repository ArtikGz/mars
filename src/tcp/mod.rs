use crate::VarInt;
use std::io;
use std::io::{Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub mod client;
mod event;
mod mapper;
mod packet;
pub mod server;
mod state;
mod utils;

pub trait AsyncReadOwnExt: AsyncReadExt + Unpin {
    async fn read_var_int(&mut self) -> io::Result<VarInt> {
        let mut value = 0u32;

        let mut current_byte = 0x80u8;
        let mut n = 0;
        while current_byte & 0x80 != 0 {
            if n > 5 {
                return Err(io::Error::other("VarInt reading error"));
            }

            current_byte = self.read_u8().await?;
            value |= ((current_byte & 0x7F) as u32) << (7 * n);

            n += 1;
        }

        Ok(value)
    }

    async fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_var_int().await?;
        let mut buffer = vec![0; len as usize];

        self.read_exact(&mut buffer).await?;

        Ok(String::from_utf8(buffer).unwrap())
    }

    async fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8().await? != 0)
    }

    async fn read_uuid(&mut self) -> io::Result<(bool, Vec<u8>)> {
        let has_uuid = self.read_bool().await?;
        let uuid = if has_uuid {
            let mut buffer = vec![0; 16];

            self.read_exact(&mut buffer).await?;
            buffer
        } else {
            vec![]
        };

        Ok((has_uuid, uuid))
    }
}

impl<T: AsyncRead + Unpin + ?Sized> AsyncReadOwnExt for T {}

pub trait AsyncWriteOwnExt: AsyncWriteExt + Unpin {
    async fn write_var_int(&mut self, value: VarInt) -> io::Result<()> {
        let mut value = value;

        for _ in 0..5 {
            let mut current = (value & 0x7F) as u8;
            value = value >> 7;

            if value > 0 {
                current = current | 0x80;
            }

            self.write_u8(current).await?;

            if value == 0 {
                break;
            }
        }

        Ok(())
    }

    async fn write_string(&mut self, value: &str) -> io::Result<()> {
        self.write_var_int(value.len() as VarInt).await?;
        self.write(value.as_bytes()).await?;

        Ok(())
    }
}

impl<T: AsyncWriteExt + Unpin + ?Sized> AsyncWriteOwnExt for T {}
