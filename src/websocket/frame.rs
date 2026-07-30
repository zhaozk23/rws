use crate::websocket::opcode::Opcode;
pub struct Frame {
    pub(crate) fin: bool,
    pub(crate) opcode: Opcode,
    pub(crate) payload: Vec<u8>,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            fin: false,
            opcode: Opcode::CONT,
            payload: Vec::new(),
        }
    }
}
impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "opcode: {}", self.opcode)?;
        write!(f, "payload_len: {}", self.payload.len())?;
        write!(f, "payload:     ")?;
        for &p in self.payload.iter() {
            write!(f, "0x{:02X} ", p)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}
