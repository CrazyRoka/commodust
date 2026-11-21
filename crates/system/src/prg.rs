#[derive(Debug, PartialEq, Eq)]
pub enum PrgError {
    TooShort,
    TooLarge,
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
            let load_address = (high << 8) | low;

            if load_address as usize + bytes.len() - 2 > 0x10000 {
                Err(PrgError::TooLarge)
            } else {
                let data = bytes[2..].to_vec();
                Ok(Self { load_address, data })
            }
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

    #[test]
    fn reject_files_exceeding_64kb_address_space() {
        // $FFFE + 3 bytes exceeds $FFFF by 1 byte
        assert_eq!(
            Prg::from_bytes(&[0xFE, 0xFF, 0x11, 0x22, 0x33]),
            Err(PrgError::TooLarge)
        );

        // $0000 + 65,537 bytes (0x10001) catches `len as u16` truncation
        let mut oversized = vec![0xAA; 2 + 0x10001];
        oversized[0] = 0x00;
        oversized[1] = 0x00;
        assert_eq!(Prg::from_bytes(&oversized), Err(PrgError::TooLarge));
    }

    #[test]
    fn accept_files_filling_exact_end_of_address_space() {
        // $FFFE + 2 bytes lands cleanly on $FFFE..=$FFFF
        let prg = Prg::from_bytes(&[0xFE, 0xFF, 0x11, 0x22]).expect("Must accept exact $FFFF fit");
        assert_eq!(prg.load_address, 0xFFFE);
        assert_eq!(prg.data, vec![0x11, 0x22]);

        // $0000 + 65,536 bytes (0x10000) fills $0000..=$FFFF cleanly
        let mut full_64k = vec![0x55; 2 + 0x10000];
        full_64k[0] = 0x00;
        full_64k[1] = 0x00;
        let prg = Prg::from_bytes(&full_64k).expect("Must accept full 64 KB image");
        assert_eq!(prg.load_address, 0x0000);
        assert_eq!(prg.data.len(), 0x10000);
    }
}
