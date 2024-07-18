use crate::tcp::client;
use crate::tcp::event::Event;
use crate::tcp::packet::S2c;
use crate::{log, measure};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

type ClientMap = Mutex<HashMap<u32, Sender<Arc<S2c>>>>;

fn get_clients() -> &'static ClientMap {
    static CLIENTS: OnceLock<ClientMap> = OnceLock::new();

    CLIENTS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn start_server(host: &'_ str, port: &'_ str) {
    log::debug!("Starting server...");

    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    log::info!("Serving at {}:{}", host, port);

    let (_event_channel_writer, mut event_channel_reader) = mpsc::channel(16);
    tokio::select! {
        _ = event_loop(&mut event_channel_reader) => {}
        _ = serve(listener) => {}
    }
}

fn generate_client_id() -> u32 {
    0
}

async fn event_loop(event_channel_reader: &mut Receiver<Event>) {
    while let Some(event) = event_channel_reader.recv().await {
        match event {
            Event::BroadcastEvent { packet } => {
                let clients = get_clients().lock().await;

                for (_, client) in clients.iter() {
                    client.send(packet.clone()).await.expect("closed channel");
                }
            }
        }
    }
}

async fn serve(listener: TcpListener) {
    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                log::error!("An error occurred while accepting a connection: {}.", e);
                continue;
            }
        };

        log::debug!("New client connected: {}", addr);
        tokio::task::spawn(handle_connection(socket));
    }
}

async fn handle_connection(socket: TcpStream) {
    let (mut connection_reader, mut connection_writer) = socket.into_split();
    let (message_channel_sender, mut message_channel_reader) = mpsc::channel::<Arc<S2c>>(16);

    let client_id = generate_client_id();
    get_clients()
        .lock()
        .await
        .insert(client_id, message_channel_sender.clone());

    tokio::select!(
        Err(e) = client::handle_incoming(&mut connection_reader, &message_channel_sender) => {
            log::error!("Client crashed while handling incoming packet with an error: {}", e);
        },
        Err(e) = client::handle_outgoing(&mut connection_writer, &mut message_channel_reader) => {
            log::error!("Client crashed while handling outgoing packets with an error: {}", e);
        }
    );
}
