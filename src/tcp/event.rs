use std::sync::Arc;

use super::packet::S2c;

#[derive(Debug)]
pub enum Event {
    BroadcastEvent { packet: Arc<S2c> },
}
