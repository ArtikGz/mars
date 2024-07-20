use crate::blocks::chunk::ChunkPos;
use crate::blocks::world::get_world;
use crate::tcp::packet::C2s;
use crate::tcp::state::State;
use crate::tcp::{mapper, AsyncWriteOwnExt};
use crate::{log, measure};
use std::io::{self, Read, Write};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Take};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{Receiver, Sender};

use super::packet::{Players, S2c, Version};
use super::{utils, AsyncReadOwnExt};

pub async fn handle_incoming(
    reader: &mut OwnedReadHalf,
    chan_writer: &Sender<Arc<S2c>>,
) -> io::Result<()> {
    let mut state = State::Shake;
    let reader = &mut BufReader::new(reader);

    loop {
        let packet_len = reader.read_var_int().await?;

        if packet_len > 0 && reader.buffer().len() > 0 {
            let mut reader = reader.take(packet_len as u64);

            state = handle_packet(state, &mut reader, chan_writer).await?;
        }
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

            let response = S2c::LoginPlay {};
            S2c::send_to(Arc::new(response), chan_writer).await?;

            // Generate demo world
            {
                let radius = 7 / 2;
                for x in -radius..radius {
                    for z in -radius..radius {
                        let mut world = get_world().lock().await;
                        let chunk = world.get_chunk(&ChunkPos { x, z });
                        let response = mapper::map_chunk_to_packet(chunk.unwrap().clone());

                        S2c::send_to(Arc::new(response), chan_writer).await?;
                    }
                }
            }

            let response = S2c::SetDefaultSpawnPosition {
                location: crate::Position { x: 0, y: 0, z: 0 },
                angle: 0.0,
            };
            S2c::send_to(Arc::new(response), chan_writer).await?;

            state = State::Play;
        }
        C2s::Mock => {}
    };

    Ok(state)
}
