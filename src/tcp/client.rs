use crate::tcp::packet::C2s;
use crate::tcp::state::State;
use crate::tcp::AsyncWriteOwnExt;
use crate::{log, measure};
use std::io::{self, Read, Write};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Take};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{Receiver, Sender};

use super::packet::{Players, S2c, Version};
use super::{utils, AsyncReadOwnExt};

pub async fn handle_incoming(
    receiver: &mut OwnedReadHalf,
    chan_writer: &Sender<Arc<S2c>>,
) -> io::Result<()> {
    let mut state = State::Shake;
    let reader = &mut BufReader::new(receiver);

    loop {
        let packet_len = reader.read_var_int().await?;
        let mut reader = reader.take(packet_len as u64);

        state = measure!(
            "handle_incoming[handle_packet]()",
            handle_packet(state, &mut reader, chan_writer).await?
        );
    }
}

pub async fn handle_outgoing(
    connection_writer: &mut OwnedWriteHalf,
    chan_reader: &mut Receiver<Arc<S2c>>,
) -> io::Result<()> {
    let mut connection_writer = BufWriter::new(connection_writer);

    while let Some(packet) = chan_reader.recv().await {
        measure!("handle_outgoing[LOOP_ITER]()", {
            let mut buffer = vec![];
            packet.write_to(&mut buffer).await?;

            log::debug!(
                "OUTGOING_PACKET with {} bytes => {:?}",
                buffer.len(),
                packet
            );

            connection_writer.write_var_int(buffer.len() as u32).await?;
            connection_writer.write_all(&*buffer).await?;

            connection_writer.flush().await?;
        });
    }

    Ok(())
}

pub async fn handle_packet(
    mut state: State,
    data: &mut impl AsyncReadOwnExt,
    chan_writer: &Sender<Arc<S2c>>,
) -> io::Result<State> {
    log::debug!("Reading C2s packet, using state: {:?}", state);
    match C2s::read(state, data).await? {
        C2s::Handshake { next_state, .. } => {
            state = State::from_int(next_state).map_err(io::Error::other)?
        }
        C2s::StatusRequest => {
            let response = S2c::StatusResponse {
                text: "Powered by mars.rs".to_owned(),
                version: Version {
                    name: "Mars".to_owned(),
                    protocol: 762,
                },
                players: Players { online: 5, max: 10 },
            };

            S2c::send_to(Arc::new(response), chan_writer).await?;
        }
        C2s::PingRequest { timestamp } => {
            let response = S2c::PongResponse { timestamp };

            S2c::send_to(Arc::new(response), chan_writer).await?;
        }
        C2s::LoginStart { name, uuid } => {
            let uuid = uuid.unwrap_or(utils::generate_offline_uuid(name.as_str()));
            let response = S2c::LoginSuccess { name, uuid };

            S2c::send_to(Arc::new(response), chan_writer).await?;
            state = State::Play;
        }
    };

    Ok(state)
}
