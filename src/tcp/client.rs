use crate::tcp::event::Event;
use crate::tcp::packet::C2s;
use crate::tcp::state::State;
use crate::tcp::{network, packet, AsyncWriteOwnExt, ReadExt, WriteExt};
use crate::{blocks, log, VarInt};
use core::time;
use std::any::Any;
use std::io;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use uuid::Uuid;

use super::packet::{Players, S2c, Version};
use super::utils;

macro_rules! wtchan {
    ($chan:ident, $obj:expr) => {
        $chan
            .send($obj)
            .await
            .map_err(|err| io::Error::other(err.to_string()))
    };
}

pub async fn handle_incoming(
    receiver: &mut OwnedReadHalf,
    chan_writer: &Sender<S2c>,
) -> io::Result<()> {
    let mut state = State::Shake;

    loop {
        let data = network::next_packet(receiver).await?;

        state = handle_packet(state, data, chan_writer).await?;
    }
}

pub async fn handle_outgoing(
    connection_writer: &mut OwnedWriteHalf,
    chan_reader: &mut Receiver<S2c>,
) -> io::Result<()> {
    while let Some(packet) = chan_reader.recv().await {
        let mut buffer = vec![];
        packet.write_to(&mut buffer).await?;

        log::debug!("Writing {} bytes to the client.", buffer.len());
        connection_writer.write_var_int(buffer.len() as u32).await?;
        connection_writer.write(&*buffer).await?;
    }

    Ok(())
}

pub async fn handle_packet(
    mut state: State,
    data: impl io::Read,
    chan_writer: &Sender<S2c>,
) -> io::Result<State> {
    log::debug!("Reading C2s packet, using state: {:?}", state);
    match C2s::read(state, data)? {
        C2s::Handshake { next_state, .. } => {
            state = State::from_int(next_state).map_err(io::Error::other)?
        }
        C2s::StatusRequest => wtchan!(
            chan_writer,
            S2c::StatusResponse {
                text: "Powered by mars.rs".to_owned(),
                version: Version {
                    name: "Mars".to_owned(),
                    protocol: 762
                },
                players: Players { online: 5, max: 10 }
            }
        )?,
        C2s::PingRequest { timestamp } => wtchan!(chan_writer, S2c::PongResponse { timestamp })?,
        C2s::LoginStart { name, uuid } => {
            let uuid = uuid.unwrap_or(utils::generate_offline_uuid(name.as_str()));

            wtchan!(chan_writer, S2c::LoginSuccess { name, uuid })?;
            state = State::Play;
        }
    };

    Ok(state)
}
