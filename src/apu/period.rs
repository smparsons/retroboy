use bincode::{Encode, Decode};

#[derive(Clone, Debug, Encode, Decode)]
pub struct Period {
    pub low: u8,
    pub high: u8,
    pub divider: u16,
    pub reloaded: bool
}

pub fn initalize_period() -> Period {
    Period {
        low: 0,
        high: 0,
        divider: 0,
        reloaded: false
    }
}

const WAVE_CHANNEL_PERIOD_DELAY: u16 = 3;

pub fn step(period: &mut Period, mut divider_increment: u8, mut handle_divider_reload: impl FnMut()) {
    period.reloaded = false;
    while divider_increment > 0 {
        period.divider -= 1;
        if period.divider == 0 {
            period.divider = calculate_period_divider(&period);
            handle_divider_reload();
            period.reloaded = true;
        }
        divider_increment -= 1;
    }
    if period.divider != calculate_period_divider(period) {
        period.reloaded = false;
    }
}

pub fn calculate_period_value(period: &Period) -> u16 {
    let period_high_bits = (period.high & 0b111) as u16;
    let period_low_bits = period.low as u16;
    (period_high_bits << 8) | period_low_bits
}

pub fn calculate_period_divider(period: &Period) -> u16 {
    2048 - calculate_period_value(period)
}

pub fn trigger(period: &mut Period) {
    period.divider = calculate_period_divider(&period);
}

pub fn apply_wave_channel_trigger_delay(period: &mut Period) {
    period.divider += WAVE_CHANNEL_PERIOD_DELAY;
}