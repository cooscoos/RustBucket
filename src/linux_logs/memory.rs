#[derive(Debug)]
pub struct Memory {
    _total: f32, // currently unused
    _free: f32,  // currently unused
    pub available: f32,
    pub used: f32,
}

impl Memory {
    pub fn default(total: u32, free: u32, available: u32, buffers: u32, cached: u32) -> Self {
        Self {
            _total: Memory::as_gb(total),
            _free: Memory::as_gb(free),
            available: Memory::as_gb(available),
            used: Memory::as_gb(total - free - buffers - cached),
        }
    }

    // Return kb memory as gb
    fn as_gb(kb_in: u32) -> f32 {
        (kb_in as f32) / (1024_f32.powf(2.0))
    }
}
