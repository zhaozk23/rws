use crate::opcode::Opcode;
pub struct Frame {
    pub(crate) fin: bool,
    pub(crate) rsv1: bool,
    pub(crate) rsv2: bool,
    pub(crate) rsv3: bool,
    pub(crate) opcode: Opcode,
    pub(crate) payload: Vec<u8>,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            fin: false,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: Opcode::CONT,
            payload: Vec::new(),
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "opcode: {}", self.opcode)?;
        writeln!(f, "payload_len: {}", self.payload.len())?;
        writeln!(f, "payload:     ")?;
        for &p in self.payload.iter() {
            write!(f, "0x{:02X} ", p)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
