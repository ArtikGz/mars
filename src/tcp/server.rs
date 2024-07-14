use crate::log;
use crate::tcp::client;
use crate::tcp::client::Client;
use crate::tcp::event::Event;
use crate::tcp::packet::S2c;
use crate::tcp::state::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use std::task::Poll;
use std::time::Duration;
use tokio::net::unix::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{self, Mutex, RwLock};
use tokio::sync::{mpsc, MutexGuard};
use tokio::time;

type ClientId = u32; // Equivalent to in game entityid
type ClientStorage = Arc<Mutex<HashMap<ClientId, Arc<Mutex<Client>>>>>;

fn get_clients() -> &'static ClientStorage {
    static CLIENTS: OnceLock<ClientStorage> = OnceLock::new();

    CLIENTS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub async fn start_server(host: &'_ str, port: &'_ str) {
    log::debug!("Starting server...");

    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    log::info!("Serving at {}:{}", host, port);

    tokio::select! {
        _ = serve(listener) => {}
    }
}

fn generate_client_id() -> u32 {
    0
}

async fn event_loop() {
    loop {}
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
    log::debug!("Handling the connection.");

    let (mut reader, writer) = socket.into_split();

    let client_id = generate_client_id();
    let client = Arc::new(Mutex::new(Client::new(client_id, writer)));
    get_clients().lock().await.insert(client_id, client.clone());

    let (s, mut r) = mpsc::channel::<S2c>(16);

    let mut c1 = client.clone();
    let mut c2 = client.clone();

    log::debug!("Initializing client handlers.");
    tokio::select!(
        Err(e) = client::handle_incoming(&mut reader, &s) => {
            log::error!("Client crashed while handling incoming packet with an error: {}", e);
        },
        Err(e) = client::handle_outgoing(&mut c2, &mut r) => {
            log::error!("Client crashed while handling outgoing packets with an error: {}", e);
        }
    );

    log::debug!("Client connection ended.");
}
