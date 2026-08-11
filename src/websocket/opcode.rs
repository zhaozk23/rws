#[repr(u8)]
#[derive(PartialEq)]
pub enum Opcode {
    CONT = 0x0,
    TEXT = 0x1,
    BIN = 0x2,
    CLOSE = 0x8,
    PING = 0x9,
    PONG = 0xA,
}

impl Opcode {
    pub fn is_control(&self) -> bool {
        matches!(self, Opcode::CLOSE | Opcode::PING | Opcode::PONG)
    }
}
impl TryFrom<u8> for Opcode {
    type Error = ();
    fn try_from(value: u8) -> std::prelude::v1::Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Opcode::CONT),
            0x1 => Ok(Opcode::TEXT),
            0x2 => Ok(Opcode::BIN),
            0x8 => Ok(Opcode::CLOSE),
            0x9 => Ok(Opcode::PING),
            0xA => Ok(Opcode::PONG),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::CONT => write!(f, "cont"),
            Opcode::TEXT => write!(f, "text"),
            Opcode::BIN => write!(f, "bin"),
            Opcode::CLOSE => write!(f, "close"),
            Opcode::PING => write!(f, "ping"),
            Opcode::PONG => write!(f, "pong"),
        }
    }
}
