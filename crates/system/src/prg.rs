#[derive(Debug, PartialEq, Eq)]
pub enum PrgError {
    TooShort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prg {
    load_address: u16,
    data: Vec<u8>,
}

impl Prg {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PrgError> {
        if bytes.len() < 2 {
            Err(PrgError::TooShort)
        } else {
            let low = bytes[0] as u16;
            let high = bytes[1] as u16;

            Ok({
                Self {
                    load_address: (high << 8) | low,
                    data: bytes[2..].to_vec(),
                }
            })
        }
    }

    pub fn load_address(&self) -> u16 {
        self.load_address
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_files_shorter_than_two_bytes() {
        assert_eq!(Prg::from_bytes(&[]), Err(PrgError::TooShort));
        assert_eq!(Prg::from_bytes(&[0x01]), Err(PrgError::TooShort));
    }

    #[test]
    fn parse_valid_prg_header_and_payload() {
        // Load address $0801 (standard BASIC: 0x01, 0x08) followed by 3 bytes
        let raw = [0x01, 0x08, 0xAA, 0xBB, 0xCC];
        let prg = Prg::from_bytes(&raw).expect("Must parse valid PRG");

        assert_eq!(prg.load_address, 0x0801);
        assert_eq!(prg.data, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn parse_arbitrary_machine_code_load_address() {
        // Load address $C000 (0x00, 0xC0) with RTS ($60) opcode
        let raw = [0x00, 0xC0, 0x60];
        let prg = Prg::from_bytes(&raw).expect("Must parse valid PRG");

        assert_eq!(prg.load_address, 0xC000);
        assert_eq!(prg.data, vec![0x60]);
    }
}
