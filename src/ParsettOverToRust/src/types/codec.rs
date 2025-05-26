#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Unknown = 0,
    Avc = 1,
    Hevc = 2,
    Xvid = 3,
    Mpeg = 4,
    Av1 = 5,
}

impl Codec {
    pub fn as_str(&self) -> &'static str {
        match self {
            Codec::Unknown => "unknown",
            Codec::Avc => "avc",
            Codec::Hevc => "hevc",
            Codec::Xvid => "xvid",
            Codec::Mpeg => "mpeg",
            Codec::Av1 => "av1",
        }
    }
}
