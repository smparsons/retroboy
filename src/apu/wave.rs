use crate::apu::period;
use crate::apu::period::{initalize_period, Period};
use crate::apu::length;
use crate::apu::length::{initialize_length, Length};
use crate::apu::utils::{bounded_wrapping_add, length_enabled};
use crate::emulator::{is_cgb, Emulator};
use crate::utils::is_bit_set;
use bincode::{Encode, Decode};

#[derive(Clone, Debug, Encode, Decode)]
pub struct WaveChannel {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub length: Length,
    pub volume: u8,
    pub period: Period,
    pub wave_position: u8,
    pub wave_pattern_ram: [u8; 0x10],
}

pub fn initialize_wave_channel() -> WaveChannel {
    WaveChannel {
        enabled: false,
        dac_enabled: false,
        length: initialize_length(),
        volume: 0,
        period: initalize_period(),
        wave_position: 1,
        wave_pattern_ram: [0; 0x10],
    }
}

pub fn reset_wave_channel(original_wave_channel: &WaveChannel, is_cgb: bool) -> WaveChannel {
    let mut new_wave_channel = initialize_wave_channel();

    if !is_cgb {
        // On reset (when APU is powered down), maintain length timers, as this is expected behavior for DMG
        new_wave_channel.length = length::reset_initial_settings(&original_wave_channel.length);
    }
    
    // APU powering down should also not affect wave RAM.
    new_wave_channel.wave_pattern_ram = original_wave_channel.wave_pattern_ram;
    new_wave_channel.wave_position = original_wave_channel.wave_position;

    new_wave_channel
}

const MAX_WAVE_SAMPLE_STEPS: u8 = 31;
const PERIOD_HIGH_TRIGGER_INDEX: u8 = 7;

pub fn step(channel: &mut WaveChannel, last_instruction_clock_cycles: u8) {
    if channel.enabled {
        period::step(&mut channel.period, last_instruction_clock_cycles / 2, || {
            channel.wave_position = bounded_wrapping_add(channel.wave_position, MAX_WAVE_SAMPLE_STEPS);
        });
    }
}

pub fn should_clock_length_on_enable(channel: &WaveChannel, original_period_high_value: u8) -> bool {
    let new_period_high_value = channel.period.high;
    !length_enabled(original_period_high_value) && length_enabled(new_period_high_value)
}

pub fn should_clock_length_on_trigger(channel: &WaveChannel) -> bool {
    length::at_max_wave_channel_length(&channel.length) && length_enabled(channel.period.high)
}

pub fn step_length(channel: &mut WaveChannel) {
    let length_timer_enabled = length_enabled(channel.period.high);
    if length_timer_enabled {
        length::step(&mut channel.length);
        if channel.length.timer == 0 {
            disable(channel);
        } 
    }
}

pub fn read_from_wave_ram(channel: &WaveChannel, localized_address: u8) -> u8 {
    channel.wave_pattern_ram[localized_address as usize]
}

pub fn write_to_wave_ram(channel: &mut WaveChannel, localized_address: u8, new_value: u8) {
    channel.wave_pattern_ram[localized_address as usize] = new_value;
}

pub fn digital_output(emulator: &Emulator) -> f32 {
    if emulator.apu.channel3.enabled {
        let localized_address = emulator.apu.channel3.wave_position / 2;
        let byte_offset = emulator.apu.channel3.wave_position % 2;
    
        let byte = read_from_wave_ram(&emulator.apu.channel3, localized_address);
        let sample = if byte_offset == 0 { (byte & 0xF0) >> 4 } else { byte & 0xF };
    
        let output_level = (emulator.apu.channel3.volume & 0b01100000) >> 5;
        match output_level {
            0b01 => sample as f32,
            0b10 => (sample >> 1) as f32,
            0b11 => (sample >> 2) as f32,
            _ => 7.5
        }
    }
    else {
        7.5
    }
}

fn corrupt_wave_ram_bug(channel: &mut WaveChannel) {
    // DMG has a bug that will corrupt wave RAM if the channel is re-triggered
    // right before it reads from wave RAM.
    let offset = (((channel.wave_position + 1) >> 1) & 0xF) as usize;
    if offset < 4 {
        channel.wave_pattern_ram[0] = channel.wave_pattern_ram[offset];
    }
    else {
        let copy_base_position = offset & !3;
        for copy_offset in 0..=3 {
            let copy_position = copy_base_position + copy_offset;
            channel.wave_pattern_ram[copy_offset] = channel.wave_pattern_ram[copy_position];
        }
    } 
}

pub fn trigger(emulator: &mut Emulator) {
    let is_cgb = is_cgb(emulator);
    let channel = &mut emulator.apu.channel3;
    if channel.enabled && channel.period.divider == 1 && !is_cgb {
        corrupt_wave_ram_bug(channel);
    }

    channel.wave_position = 0;

    if channel.dac_enabled {
        channel.enabled = true;
    }

    period::trigger(&mut channel.period);
    period::apply_wave_channel_trigger_delay(&mut channel.period);
    length::reload_wave_channel_timer_with_maximum(&mut channel.length);
}

pub fn disable(channel: &mut WaveChannel) {
    channel.enabled = false;
}

pub fn should_trigger(channel: &WaveChannel) -> bool {
   is_bit_set(channel.period.high, PERIOD_HIGH_TRIGGER_INDEX)
}

#[cfg(test)]
mod tests {
    use crate::emulator::initialize_screenless_emulator;
    use super::*;

    fn enable_wave_channel(channel: &mut WaveChannel) {
        channel.enabled = true;
        channel.dac_enabled = true;
    }

    #[test]
    fn should_calculate_dac_output_when_amplitude_is_zero() {
        let mut emulator = initialize_screenless_emulator();
        enable_wave_channel(&mut emulator.apu.channel3);

        emulator.apu.channel3.wave_pattern_ram[0] = 0xAC;
        emulator.apu.channel3.wave_pattern_ram[1] = 0xC0;
        emulator.apu.channel3.wave_pattern_ram[2] = 0x04;
        emulator.apu.channel3.wave_pattern_ram[3] = 0xDC;

        emulator.apu.channel3.wave_position = 3;
        emulator.apu.channel3.volume = 0b00100000;

        assert_eq!(digital_output(&emulator), 0.0);
    }

    #[test]
    fn should_calculate_dac_output_when_amplitude_is_non_zero() {
        let mut emulator = initialize_screenless_emulator();
        enable_wave_channel(&mut emulator.apu.channel3);

        emulator.apu.channel3.wave_pattern_ram[0] = 0xAC;
        emulator.apu.channel3.wave_pattern_ram[1] = 0xC0;
        emulator.apu.channel3.wave_pattern_ram[2] = 0x04;
        emulator.apu.channel3.wave_pattern_ram[3] = 0xDC;

        emulator.apu.channel3.wave_position = 5;
        emulator.apu.channel3.volume = 0b00100000;

        assert_eq!(digital_output(&emulator), 4.0); 
    }

    #[test]
    fn should_generate_no_sound_if_channel_is_muted() {
        let mut emulator = initialize_screenless_emulator();
        enable_wave_channel(&mut emulator.apu.channel3);

        emulator.apu.channel3.wave_pattern_ram[0] = 0xAC;
        emulator.apu.channel3.wave_pattern_ram[1] = 0xC0;
        emulator.apu.channel3.wave_pattern_ram[2] = 0x04;
        emulator.apu.channel3.wave_pattern_ram[3] = 0xDC;

        emulator.apu.channel3.wave_position = 5;
        emulator.apu.channel3.volume = 0;

        assert_eq!(digital_output(&emulator), 7.5); 
    }

    #[test]
    fn should_shift_sample_right_once_if_channel_is_set_to_half_of_volume() {
        let mut emulator = initialize_screenless_emulator();
        enable_wave_channel(&mut emulator.apu.channel3);

        emulator.apu.channel3.wave_pattern_ram[0] = 0xAC;
        emulator.apu.channel3.wave_pattern_ram[1] = 0xC0;
        emulator.apu.channel3.wave_pattern_ram[2] = 0x04;
        emulator.apu.channel3.wave_pattern_ram[3] = 0xDC;

        emulator.apu.channel3.wave_position = 5;
        emulator.apu.channel3.volume = 0b01000000;

        assert_eq!(digital_output(&emulator), 2.0); 
    }

    #[test]
    fn should_produce_no_audio_output_if_channel_is_disabled() {
        let mut emulator = initialize_screenless_emulator();

        emulator.apu.channel3.wave_pattern_ram[0] = 0xAC;
        emulator.apu.channel3.wave_pattern_ram[1] = 0xC0;
        emulator.apu.channel3.wave_pattern_ram[2] = 0x04;
        emulator.apu.channel3.wave_pattern_ram[3] = 0xDC;

        emulator.apu.channel3.wave_position = 5;
        emulator.apu.channel3.volume = 0b01000000;

        assert_eq!(digital_output(&emulator), 7.5); 
    }
}