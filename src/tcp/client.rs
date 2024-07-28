use crate::blocks::chunk::ChunkPos;
use crate::blocks::world::get_world;
use crate::tcp::packet::C2s;
use crate::tcp::state::State;
use crate::tcp::{mapper, AsyncWriteOwnExt};
use crate::{log, measure};
use std::borrow::Borrow;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Take};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, RwLock};

use super::packet::{Players, S2c, Version};
use super::{utils, AsyncReadOwnExt};

pub async fn handle_incoming(
    state: &Mutex<State>,
    reader: &mut OwnedReadHalf,
    chan_writer: &Sender<Arc<S2c>>,
) -> io::Result<()> {
    let reader = &mut BufReader::new(reader);

    loop {
        let packet_len = reader.read_var_int().await?;

        if packet_len > 0 && reader.buffer().len() > 0 {
            let mut reader = reader.take(packet_len as u64);
            let current_state = *state.lock().await;

            match handle_packet(current_state, &mut reader, chan_writer).await? {
                Some(new_state) => {
                    *state.lock().await = new_state;
                }
                None => {}
            };

            if reader.limit() != 0 {
                log::warn!("Packet wasn't fully readed!!!");
                reader.consume(reader.limit() as usize);
            }
        }
    }
}

pub async fn handle_outgoing(
    state: &Mutex<State>,
    connection_writer: &mut OwnedWriteHalf,
    chan_reader: &mut Receiver<Arc<S2c>>,
) -> io::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(10));

    'main: loop {
        tokio::select! {
            _ = ticker.tick() => {

                if *state.lock().await == State::Play {
                    let now = SystemTime::now();
                    let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).map_err(io::Error::other)?;

                    send_packet(connection_writer, Arc::new(S2c::KeepAlive{ id: duration_since_epoch.as_nanos() as u64 })).await?;
                }
            },
            packet = chan_reader.recv() => {
                match packet {
                    Some(packet) => { send_packet(connection_writer, packet.clone()).await?; },
                    _ => break 'main,
                }
            },
        };
    }

    Ok(())
}

async fn send_packet(connection_writer: &mut OwnedWriteHalf, packet: Arc<S2c>) -> io::Result<()> {
    let mut connection_writer = BufWriter::new(connection_writer);

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

    Ok(())
}

pub async fn handle_packet(
    state: State,
    data: &mut impl AsyncReadOwnExt,
    chan_writer: &Sender<Arc<S2c>>,
) -> io::Result<Option<State>> {
    let mut result_state = None;

    log::debug!("Reading C2s packet, using state: {:?}", state);
    match C2s::read(state, data).await? {
        C2s::Handshake { next_state, .. } => {
            result_state = Some(State::from_int(next_state).map_err(io::Error::other)?)
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
                location: crate::Position { x: 0, y: 50, z: 0 },
                angle: 0.0,
            };
            S2c::send_to(Arc::new(response), chan_writer).await?;

            result_state = Some(State::Play);
        }
        C2s::Mock => {}
    };

    Ok(result_state)
}
