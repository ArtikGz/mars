#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Shake,
    Status,
    Login,
    Play,
}

impl State {
    pub fn from_int(n: u32) -> Result<Self, &'static str> {
        match n {
            0 => Ok(Self::Shake),
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Play),
            _ => Err("Invalid state id."),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Shake
    }
}
