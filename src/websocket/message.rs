use crate::error::WsError;
use crate::opcode::Opcode;
#[repr(u8)]
#[derive(PartialEq)]
pub enum MessageKind {
    TEXT = Opcode::TEXT as u8,
    BIN = Opcode::BIN as u8,
}
impl TryFrom<Opcode> for MessageKind {
    type Error = WsError;
    fn try_from(value: Opcode) -> Result<Self, Self::Error> {
        match value {
            Opcode::TEXT => Ok(Self::TEXT),
            Opcode::BIN => Ok(Self::BIN),
            _ => Err(WsError::InvalidOpcode),
        }
    }
}
impl std::fmt::Display for MessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageKind::BIN => write!(f, "bin"),
            MessageKind::TEXT => write!(f, "text"),
        }
    }
}

pub struct Message {
    pub kind: MessageKind,
    pub payload: Vec<u8>,
}
impl Message {
    pub fn new() -> Self {
        Message {
            kind: MessageKind::TEXT,
            payload: Vec::new(),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "message kind: {}", self.kind)?;
        writeln!(f, "message length: {}", self.payload.len())?;
        for &p in self.payload.iter() {
            write!(f, "0x{:02X} ", p)?;
        }
        writeln!(f)?;
        writeln!(f, "-----------------------------")?;
        Ok(())
    }
}
