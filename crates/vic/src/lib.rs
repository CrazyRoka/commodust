mod address_generator;
mod border;
mod raster;
mod registers;
mod sequencer;
mod vic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoStandard {
    /// MOS 6569 (Europe/Australia - 50 Hz)
    Pal,
    // MOS 6567R8 (North America/Japan - 60 Hz)
    Ntsc,
}

impl VideoStandard {
    #[inline(always)]
    pub const fn cycles_per_line(&self) -> u8 {
        match self {
            Self::Pal => 63,
            Self::Ntsc => 65,
        }
    }

    #[inline(always)]
    pub const fn total_lines(&self) -> u16 {
        match self {
            Self::Pal => 312,
            Self::Ntsc => 263,
        }
    }

    #[inline(always)]
    pub const fn total_cycles_per_frame(&self) -> u32 {
        self.cycles_per_line() as u32 * self.total_lines() as u32
    }
}
