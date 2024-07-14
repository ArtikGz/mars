use super::packet::S2c;

#[derive(Debug)]
pub enum Event {
    BroadcastEvent { packet: S2c },
}
