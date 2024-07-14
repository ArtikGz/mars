use crate::VarInt;
use std::io;
use std::io::{Read, Write};
use std::pin::Pin;
use std::thread::current;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, ReadHalf};
use tokio::net::TcpStream;

pub mod client;
mod event;
mod network;
mod packet;
pub mod server;
mod state;
mod utils;

trait AsyncReadOwnExt: AsyncRead + Unpin {
    async fn read_var_int(&mut self) -> io::Result<VarInt> {
        let mut value = 0u32;
        let mut position = 0;
        let mut current_byte;

        loop {
            current_byte = self.read_u8().await?;
            value |= ((current_byte & 0x7F) as u32) << position;

            if current_byte & 0x80 == 0 {
                break;
            }
            position += 7;

            if position >= 32 {
                return Err(io::Error::other("VarInt reading error"));
            }
        }

        Ok(value)
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

trait ReadExt: Read {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_var_int(&mut self) -> io::Result<VarInt> {
        let mut value = 0u32;
        let mut position = 0;
        let mut current_byte;

        loop {
            current_byte = self.read_u8()?;
            value |= ((current_byte & 0x7F) as u32) << position;

            if current_byte & 0x80 == 0 {
                break;
            }
            position += 7;

            if position >= 32 {
                return Err(io::Error::other("VarInt reading error"));
            }
        }

        Ok(value)
    }

    fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_var_int()?;
        let mut buffer = vec![0; len as usize];

        self.read_exact(&mut buffer)?;

        Ok(String::from_utf8(buffer).unwrap())
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buffer = [0; 2];

        self.read_exact(&mut buffer)?;

        Ok(u16::from_be_bytes(buffer))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let mut buffer = [0; 8];

        self.read_exact(&mut buffer)?;

        Ok(u64::from_be_bytes(buffer))
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let mut buffer = [0; 1];

        self.read_exact(&mut buffer)?;

        Ok(buffer[0] == 1)
    }

    fn read_uuid(&mut self) -> io::Result<(bool, Vec<u8>)> {
        let has_uuid = self.read_bool()?;
        let uuid = if has_uuid {
            let mut buffer = vec![0; 16];

            self.read_exact(&mut buffer)?;
            buffer
        } else {
            vec![]
        };

        Ok((has_uuid, uuid))
    }
}

impl<T: Read + ?Sized> ReadExt for T {}

trait WriteExt: io::Write {
    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        let mut buf = [value];
        self.write(&mut buf)?;

        Ok(())
    }

    fn write_var_int(&mut self, value: VarInt) -> io::Result<()> {
        let mut buffer = vec![];
        let mut value = value;

        for _ in 0..5 {
            let mut current = (value & 0x7F) as u8;
            value = value >> 7;

            if value > 0 {
                current = current | 0x80;
            }

            buffer.push(current);

            if value == 0 {
                break;
            }
        }

        self.write(&*buffer)?;

        Ok(())
    }

    fn write_string(&mut self, value: &str) -> io::Result<()> {
        self.write_var_int(value.len() as VarInt)?;
        self.write(value.as_bytes())?;

        Ok(())
    }

    fn write_u64(&mut self, value: u64) -> io::Result<()> {
        self.write(&value.to_be_bytes())?;

        Ok(())
    }
}

impl<T: Write + ?Sized> WriteExt for T {}
