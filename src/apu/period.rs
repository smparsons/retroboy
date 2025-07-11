use bincode::{Encode, Decode};

#[derive(Clone, Debug, Encode, Decode)]
pub struct Period {
    low: u8,
    high: u8,
    divider: u16,
    reloaded: bool
}

const WAVE_CHANNEL_PERIOD_DELAY: u16 = 3;

impl Period {
    pub fn new() -> Self {
        Period {
            low: 0,
            high: 0,
            divider: 0,
            reloaded: false
        }
    }

    pub fn step(&mut self, mut divider_increment: u8, mut handle_divider_reload: impl FnMut()) {
        self.reloaded = false;
        while divider_increment > 0 {
            self.divider -= 1;
            if self.divider == 0 {
                self.divider = self.calculate_period_divider();
                handle_divider_reload();
                self.reloaded = true;
            }
            divider_increment -= 1;
        }
        if self.divider != self.calculate_period_divider() {
            self.reloaded = false;
        }
    }

    pub fn calculate_period_value(&self) -> u16 {
        let period_high_bits = (self.high & 0b111) as u16;
        let period_low_bits = self.low as u16;
        (period_high_bits << 8) | period_low_bits
    }

    pub fn calculate_period_divider(&self) -> u16 {
        2048 - self.calculate_period_value()
    }

    pub fn trigger(&mut self) {
        self.divider = self.calculate_period_divider();
    }

    pub fn apply_wave_channel_trigger_delay(&mut self) {
        self.divider += WAVE_CHANNEL_PERIOD_DELAY;
    }

    pub fn high(&self) -> u8 {
        self.high
    }

    pub fn set_high(&mut self, value: u8) {
        self.high = value;
    }

    pub fn low(&self) -> u8 {
        self.low
    }

    pub fn set_low(&mut self, value: u8) {
        self.low = value;
    }

    pub fn divider(&self) -> u16 {
        self.divider
    }

    pub fn set_divider(&mut self, value: u16) {
        self.divider = value;
    }

    pub fn reloaded(&self) -> bool {
        self.reloaded
    }

    pub fn set_reloaded(&mut self, value: bool) {
        self.reloaded = value;
    }
}
