use crate::websocket::opcode::Opcode;
#[repr(u8)]
pub enum MessageKind {
    TEXT = Opcode::TEXT as u8,
    BIN = Opcode::BIN as u8,
}
impl From<Opcode> for MessageKind {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::TEXT => Self::TEXT,
            Opcode::BIN => Self::BIN,
            _ => {
                unreachable!()
            }
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
// type MessageChunk = Vec<u8>;
pub struct Message {
    pub kind: MessageKind,
    pub chunks: Vec<u8>,
}
impl Message {
    pub fn new() -> Self {
        Message {
            kind: MessageKind::TEXT,
            chunks: Vec::new(),
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
        writeln!(f, "message length: {}", self.chunks.len())?;
        for &p in self.chunks.iter() {
            write!(f, "0x{:02X} ", p)?;
        }
        writeln!(f)?;
        writeln!(f, "-----------------------------")?;
        Ok(())
    }
}
