#[derive(Debug, Clone)]
pub enum Event {
    BroadcastEvent { bytes: Vec<u8> }
}
