use crate::opcode::Opcode;
pub struct FrameHeader {
    pub(crate) fin: bool,
    pub(crate) rsv1: bool,
    pub(crate) rsv2: bool,
    pub(crate) rsv3: bool,
    pub(crate) opcode: Opcode,
    pub(crate) masked: bool,
    pub(crate) payload_len: usize,
    pub(crate) mask: [u8; 4],
}
impl FrameHeader {
    pub fn new() -> Self {
        FrameHeader {
            fin: false,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: Opcode::CONT,
            masked: false,
            payload_len: 0,
            mask: [0; 4],
        }
    }
}

impl Default for FrameHeader {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for FrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FIN({})", self.fin)?;
        writeln!(f, "RSV1({})", self.rsv1)?;
        writeln!(f, "RSV2({})", self.rsv2)?;
        writeln!(f, "RSV3({})", self.rsv3)?;
        writeln!(f, "OPCODE({})", self.opcode)?;
        writeln!(f, "MASKED({})", self.masked)?;
        writeln!(f, "PAYLOAD_LEN({})", self.payload_len)?;
        if self.masked {
            writeln!(f, "MASK({:?})", self.mask)?;
        }
        Ok(())
    }
}
