use crate::apu::noise::NoiseChannel;
use crate::apu::wave::WaveChannel;
use crate::apu::pulse::PulseChannel;
use crate::apu::utils::{bounded_wrapping_add, as_dac_output};
use crate::utils::{get_bit, get_t_cycle_increment, is_bit_set};
use crate::serializable::Serializable;
use crate::address_bus::MemoryMapped;
use utils::{calculate_left_stereo_sample, calculate_right_stereo_sample};
use std::io::{Read, Write};
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(Debug, CopyGetters, Setters, Getters, MutGetters)]
pub struct Apu {
    #[getset(get_copy = "pub(super)", set = "pub(super)")]
    enabled: bool,

    sound_panning: u8,
    
    master_volume: u8,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    channel1: PulseChannel,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    channel2: PulseChannel,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    channel3: WaveChannel,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    channel4: NoiseChannel,
    
    divider_apu: u8,
    
    last_divider_time: u8,
    
    #[getset(set = "pub(super)")]
    audio_buffer_clock: u8,
    
    channel_clock: u8,
    
    left_sample_queue: Vec<f32>,
    
    right_sample_queue: Vec<f32>,
    
    summed_channel1_sample: f32,
    
    summed_channel2_sample: f32,
    
    summed_channel3_sample: f32,
    
    summed_channel4_sample: f32,
    
    enqueue_rate: u32,
    
    #[getset(set = "pub(super)")]
    cgb_mode: bool,
    
    #[getset(set = "pub(super)")]
    cgb_double_speed: bool
}

pub struct ApuParams {
    pub divider: u8
}

const CH3_DAC_ENABLED_INDEX: u8 = 7;
const APU_ENABLED_INDEX: u8 = 7;
const MAX_DIV_APU_STEPS: u8 = 7;

const CPU_RATE: u32 = 4194304;
const DEFAULT_SAMPLE_RATE: u32 = 44100;
const MAX_AUDIO_BUFFER_SIZE: usize = 512;

const CHANNEL_STEP_RATE: u8 = 4;

fn calculate_sample_weight(steps_per_enqueue: u8, steps_since_enqueue: u8) -> f32 {
    let step_index = steps_per_enqueue - steps_since_enqueue;
    ((step_index as f32).ln() + 1.0) / ((steps_per_enqueue as f32).ln() + 1.0)
}

fn generate_dac_output(summed_channel_sample: f32, steps_since_enqueue: u8) -> f32 {
    let avg_channel_sample = summed_channel_sample / steps_since_enqueue as f32;
    as_dac_output(avg_channel_sample)
}

fn in_length_period_first_half(current_divider_apu: u8) -> bool {
    let length_period_first_half_steps = vec![1,3,5,7];
    length_period_first_half_steps.contains(&current_divider_apu)
}

impl Apu {
    pub(super) fn new() -> Self {
        Apu {
            enabled: false,
            sound_panning: 0,
            master_volume: 0,
            channel1: PulseChannel::new(),
            channel2: PulseChannel::new(),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            divider_apu: 0,
            last_divider_time: 0,
            audio_buffer_clock: 0,
            channel_clock: 0,
            left_sample_queue: Vec::new(),
            right_sample_queue: Vec::new(),
            summed_channel1_sample: 0.0,
            summed_channel2_sample: 0.0,
            summed_channel3_sample: 0.0,
            summed_channel4_sample: 0.0,
            enqueue_rate: CPU_RATE / DEFAULT_SAMPLE_RATE,
            cgb_mode: false,
            cgb_double_speed: false
        }
    }

    pub(super) fn reset(&mut self) {
        self.enabled = false;
        self.sound_panning = 0;
        self.master_volume = 0;
        self.divider_apu = 0;
        self.last_divider_time = 0;
        self.channel1 = PulseChannel::reset(&self.channel1, self.cgb_mode);
        self.channel2 = PulseChannel::reset(&self.channel2, self.cgb_mode);
        self.channel3 = WaveChannel::reset(&self.channel3, self.cgb_mode);
        self.channel4 = NoiseChannel::reset(&self.channel4, self.cgb_mode);
    }

    fn should_step_div_apu(&self, divider: u8) -> bool {
        let bit_to_check = if self.cgb_double_speed { 5 } else { 4 };
        get_bit(self.last_divider_time, bit_to_check) == 1 &&
            get_bit(divider, bit_to_check) == 0
    }

    fn step_div_apu(&mut self, divider: u8) {
        if self.should_step_div_apu(divider) {
            let current_divider_apu = self.divider_apu;

            let envelope_step = 7;
            let length_steps = vec![0, 2, 4, 6];
            let sweep_steps = vec![2, 6];

            if current_divider_apu == envelope_step {
                self.channel1.step_envelope();
                self.channel2.step_envelope();
                self.channel4.step_envelope(); 
            }

            if length_steps.contains(&current_divider_apu) {
                self.channel1.step_length();
                self.channel2.step_length();
                self.channel3.step_length();
                self.channel4.step_length();
            }
            
            if sweep_steps.contains(&current_divider_apu) {
                self.channel1.step_sweep();
            }

            self.divider_apu = bounded_wrapping_add(self.divider_apu, MAX_DIV_APU_STEPS);
        }
    }

    pub(crate) fn audio_buffers_full(&self) -> bool {
        self.left_sample_queue.len() >= MAX_AUDIO_BUFFER_SIZE &&
            self.right_sample_queue.len() >= MAX_AUDIO_BUFFER_SIZE
    }

    pub(crate) fn clear_audio_buffers(&mut self) {
        self.left_sample_queue.clear();
        self.right_sample_queue.clear();
    }

    pub(crate) fn get_left_sample_queue(&self) -> &[f32] {
        &self.left_sample_queue.as_slice()
    }

    pub(crate) fn get_right_sample_queue(&self) -> &[f32] {
        &self.right_sample_queue.as_slice()
    }

    fn track_digital_outputs(&mut self, weight: f32) {
        let channel1_output = self.channel1.digital_output();
        let channel2_output = self.channel2.digital_output();
        let channel3_output = self.channel3.digital_output();
        let channel4_output = self.channel4.digital_output();

        self.summed_channel1_sample += channel1_output * weight;
        self.summed_channel2_sample += channel2_output * weight;
        self.summed_channel3_sample += channel3_output * weight;
        self.summed_channel4_sample += channel4_output * weight;
    }

    pub(super) fn clear_summed_samples(&mut self) {
        self.summed_channel1_sample = 0.0;
        self.summed_channel2_sample = 0.0;
        self.summed_channel3_sample = 0.0;
        self.summed_channel4_sample = 0.0;
    }

    fn enqueue_left_sample(&mut self,
        channel1_dac_output: f32,
        channel2_dac_output: f32,
        channel3_dac_output: f32,
        channel4_dac_output: f32) {
        let left_master_volume = (self.master_volume & 0b01110000) >> 4;

        let left_sample = calculate_left_stereo_sample(self.sound_panning,
            left_master_volume,
            channel1_dac_output,
            channel2_dac_output,
            channel3_dac_output,
            channel4_dac_output);

        self.left_sample_queue.push(left_sample);
    }

    fn enqueue_right_sample(&mut self,
        channel1_dac_output: f32,
        channel2_dac_output: f32,
        channel3_dac_output: f32,
        channel4_dac_output: f32) {
        let right_master_volume = self.master_volume & 0b111;

        let right_sample = calculate_right_stereo_sample(self.sound_panning,
            right_master_volume,
            channel1_dac_output,
            channel2_dac_output,
            channel3_dac_output,
            channel4_dac_output);

        self.right_sample_queue.push(right_sample);
    }

    fn enqueue_audio_samples(&mut self) {
        let t_cycle_increment = get_t_cycle_increment(self.cgb_double_speed);

        self.audio_buffer_clock = self.audio_buffer_clock.wrapping_add(t_cycle_increment);
        let steps_since_enqueue = self.audio_buffer_clock / t_cycle_increment;
        let steps_per_enqueue = self.enqueue_rate as u8 + 1 / t_cycle_increment;

        let weight = calculate_sample_weight(steps_per_enqueue, steps_since_enqueue);
        self.track_digital_outputs(weight);

        if self.audio_buffer_clock as u32 >= self.enqueue_rate {
            self.audio_buffer_clock = 0;

            let channel1_dac_output = generate_dac_output(self.summed_channel1_sample, steps_since_enqueue);
            let channel2_dac_output = generate_dac_output(self.summed_channel2_sample, steps_since_enqueue);
            let channel3_dac_output = generate_dac_output(self.summed_channel3_sample, steps_since_enqueue);
            let channel4_dac_output = generate_dac_output(self.summed_channel4_sample, steps_since_enqueue);

            self.enqueue_left_sample(
                channel1_dac_output,
                channel2_dac_output,
                channel3_dac_output,
                channel4_dac_output);

            self.enqueue_right_sample(
                channel1_dac_output,
                channel2_dac_output,
                channel3_dac_output,
                channel4_dac_output);

            self.clear_summed_samples();
        }
    }

    pub(super) fn step(&mut self, params: ApuParams) {
        let t_cycle_increment = get_t_cycle_increment(self.cgb_double_speed);
        self.channel_clock = self.channel_clock.wrapping_add(t_cycle_increment);

        if self.enabled {
            if self.channel_clock >= CHANNEL_STEP_RATE {
                let clock_cycles = self.channel_clock;
                self.channel_clock = 0;

                self.channel1.step(clock_cycles);
                self.channel2.step(clock_cycles);
                self.channel3.step(clock_cycles);
                self.channel4.step(clock_cycles);
            }

            self.step_div_apu(params.divider);
        }
        self.enqueue_audio_samples();

        self.last_divider_time = params.divider;
    }

    pub(crate) fn set_sample_rate(&mut self, sample_rate: u32) {
        self.enqueue_rate = CPU_RATE / sample_rate;
    }

    fn set_ch1_period_high(&mut self, new_period_high_value: u8) {
        if self.enabled {
            let original_period_high_value = self.channel1.period().high();
            self.channel1.period_mut().set_high(new_period_high_value);

            let length_period_first_half = in_length_period_first_half(self.divider_apu);

            let clock_length_on_enable = self.channel1.should_clock_length_on_enable(original_period_high_value) &&
                length_period_first_half;

            if clock_length_on_enable {
                self.channel1.step_length();
            }

            if self.channel1.should_trigger() {
                self.channel1.trigger(true);

                if self.channel1.should_clock_length_on_trigger() && length_period_first_half {
                    self.channel1.step_length();
                }
            }
        }
    }

    fn set_ch2_period_high(&mut self, new_period_high_value: u8) {
        if self.enabled {
            let original_period_high_value = self.channel2.period().high();
            self.channel2.period_mut().set_high(new_period_high_value);

            let length_period_first_half = in_length_period_first_half(self.divider_apu);

            let clock_length_on_enable = self.channel2.should_clock_length_on_enable(original_period_high_value) &&
                length_period_first_half;

            if clock_length_on_enable {
                self.channel2.step_length();
            }

            if self.channel2.should_trigger() {
                self.channel2.trigger(false);

                if self.channel2.should_clock_length_on_trigger() && length_period_first_half {
                    self.channel2.step_length();
                }
            }
        }
    }

    fn set_ch3_period_high(&mut self, new_period_high_value: u8) {
        if self.enabled {
            let original_period_high_value = self.channel3.period().high();
            self.channel3.period_mut().set_high(new_period_high_value);

            let length_period_first_half = in_length_period_first_half(self.divider_apu);

            let clock_length_on_enable = self.channel3.should_clock_length_on_enable(original_period_high_value) &&
                length_period_first_half;

            if clock_length_on_enable {
                self.channel3.step_length();
            }

            if self.channel3.should_trigger() {
                self.channel3.trigger(self.cgb_mode);

                if self.channel3.should_clock_length_on_trigger() && length_period_first_half {
                    self.channel3.step_length();
                }
            }
        }
    }

    fn set_ch4_control(&mut self, new_control_value: u8) {
        if self.enabled {
            let original_control_value = self.channel4.control();
            self.channel4.set_control(new_control_value);

            let length_period_first_half = in_length_period_first_half(self.divider_apu);

            let clock_length_on_enable = self.channel4.should_clock_length_on_enable(original_control_value) &&
                length_period_first_half;

            if clock_length_on_enable {
                self.channel4.step_length();
            }

            if self.channel4.should_trigger() {
                self.channel4.trigger();

                if self.channel4.should_clock_length_on_trigger() && length_period_first_half {
                    self.channel4.step_length();
                }
            }
        }
    }

    fn set_ch1_envelope_settings(&mut self, new_envelope_settings: u8) {
        if self.enabled {
            self.channel1.envelope_mut().set_initial_settings(new_envelope_settings);
            self.channel1.envelope_mut().reset_settings();

            let should_disable = self.channel1.envelope().should_disable_dac();

            self.channel1.set_dac_enabled(!should_disable);

            if should_disable {
                self.channel1.set_enabled(false);
            }
        }
    }

    fn set_ch2_envelope_settings(&mut self, new_envelope_settings: u8) {
        if self.enabled {
            self.channel2.envelope_mut().set_initial_settings(new_envelope_settings);
            self.channel2.envelope_mut().reset_settings();

            let should_disable = self.channel2.envelope().should_disable_dac();
        
            self.channel2.set_dac_enabled(!should_disable);
        
            if should_disable {
                self.channel2.set_enabled(false);
            }
        }
    }

    fn set_ch3_dac_enabled(&mut self, new_dac_enabled_register_value: u8) {
        if self.enabled {
            let should_disable = !is_bit_set(new_dac_enabled_register_value, CH3_DAC_ENABLED_INDEX);

            self.channel3.set_dac_enabled(!should_disable);
            
            if should_disable {
                self.channel3.set_enabled(false);
            }
        }
    }

    fn set_ch4_envelope_settings(&mut self, new_envelope_settings: u8) {
        if self.enabled {
            self.channel4.envelope_mut().set_initial_settings(new_envelope_settings);
            self.channel4.envelope_mut().reset_settings();

            let should_disable = self.channel4.envelope().should_disable_dac();
        
            self.channel4.set_dac_enabled(!should_disable);
        
            if should_disable {
                self.channel4.set_enabled(false);
            }
        }
    }

    pub(super) fn audio_master_control(&self) -> u8 {
        let apu_enabled = if self.enabled { 1 } else { 0 };
        let mask = 0b01110000;
        let ch4_enabled = if self.channel4.enabled() { 1 } else { 0 };
        let ch3_enabled = if self.channel3.enabled() { 1 } else { 0 };
        let ch2_enabled = if self.channel2.enabled() { 1 } else { 0 };
        let ch1_enabled = if self.channel1.enabled() { 1 } else { 0 };
        let result = (apu_enabled << 7)
            | mask
            | (ch4_enabled << 3)
            | (ch3_enabled << 2)
            | (ch2_enabled << 1)
            | ch1_enabled;
        result
    }

    pub(super) fn get_wave_ram_byte(&self, localized_address: u8) -> u8 {
        let mut address = localized_address;

        if self.channel3.enabled() {
            address = self.channel3.wave_position() / 2;
            if self.channel3.period().reloaded() || self.cgb_mode {
                self.channel3.read_from_wave_ram(address)
            }
            else {
                0xFF
            }
        }
        else {
            self.channel3.read_from_wave_ram(address)
        }
    }

    pub(super) fn set_wave_ram_byte(&mut self, localized_address: u8, new_value: u8) {
        let mut address = localized_address;

        if self.channel3.enabled() {
            address = self.channel3.wave_position() / 2;
            if self.channel3.period().reloaded() || self.cgb_mode {
                self.channel3.write_to_wave_ram(address, new_value);
            }
        }
        else {
            self.channel3.write_to_wave_ram(address, new_value);
        }
    }

    fn set_audio_master_control(&mut self, new_audio_master_control: u8) {
        self.enabled = is_bit_set(new_audio_master_control, APU_ENABLED_INDEX);

        if !self.enabled {
            self.reset();
        }
    }

    fn set_ch1_sweep_settings(&mut self, new_sweep_settings: u8) {
        if self.enabled {
            self.channel1.sweep_mut().update_initial_settings(new_sweep_settings);
            if self.channel1.sweep().should_disable_channel() {
                self.channel1.set_enabled(false);
            }
        }
    }

    fn set_ch1_length_settings(&mut self, new_length_settings: u8) {
        if self.enabled || !self.cgb_mode {
            let new_initial_settings = if self.enabled {
                new_length_settings
            }
            else {
                new_length_settings & 0x3F
            };
            self.channel1.length_mut().set_initial_settings(new_initial_settings);
            self.channel1.length_mut().initialize_timer();
        }
    }

    fn set_ch1_period_low(&mut self, new_period_low: u8) {
        if self.enabled {
            self.channel1.period_mut().set_low(new_period_low);
        }
    }

    fn set_ch2_length_settings(&mut self, new_length_settings: u8) {
        if self.enabled || !self.cgb_mode {
            let new_initial_settings = if self.enabled {
                new_length_settings
            }
            else {
                new_length_settings & 0x3F
            };
            self.channel2.length_mut().set_initial_settings(new_initial_settings);
            self.channel2.length_mut().initialize_timer();
        }
    }

    fn set_ch2_period_low(&mut self, new_period_low: u8) {
        if self.enabled {
            self.channel2.period_mut().set_low(new_period_low);
        }
    }

    fn set_ch3_length_settings(&mut self, new_length_settings: u8) {
        if self.enabled || !self.cgb_mode {
            self.channel3.length_mut().set_initial_settings(new_length_settings);
            self.channel3.length_mut().initialize_timer();
        }
    }

    fn set_ch3_period_low(&mut self, new_period_low: u8) {
        if self.enabled {
            self.channel3.period_mut().set_low(new_period_low);
        }
    }

    fn set_ch3_volume(&mut self, new_volume: u8) {
        if self.enabled {
            self.channel3.set_volume(new_volume);
        }
    }

    fn set_ch4_length_settings(&mut self, new_length_settings: u8) {
        if self.enabled || !self.cgb_mode {
            self.channel4.length_mut().set_initial_settings(new_length_settings);
            self.channel4.length_mut().initialize_timer();
        }
    }

    fn set_ch4_polynomial(&mut self, new_polynomial: u8) {
        if self.enabled {
            self.channel4.set_polynomial(new_polynomial);
        }
    }

    pub(super) fn set_master_volume(&mut self, new_master_volume: u8) {
        if self.enabled {
            self.master_volume = new_master_volume;
        }
    }

    pub(super) fn set_sound_panning(&mut self, new_sound_panning: u8) {
        if self.enabled {
            self.sound_panning = new_sound_panning;
        }
    }
}

impl MemoryMapped for Apu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF10 => self.channel1.sweep().initial_settings() | 0b10000000,
            0xFF11 => self.channel1.length().initial_settings() | 0b00111111,
            0xFF12 => self.channel1.envelope().initial_settings(),
            0xFF14 => self.channel1.period().high() | 0b10111111,
            0xFF16 => self.channel2.length().initial_settings() | 0b00111111,
            0xFF17 => self.channel2.envelope().initial_settings(),
            0xFF19 => self.channel2.period().high() | 0b10111111,
            0xFF1A => if self.channel3.dac_enabled() { 0b11111111 } else { 0b01111111 },
            0xFF1C => self.channel3.volume() | 0b10011111,
            0xFF1E => self.channel3.period().high() | 0b10111111,
            0xFF21 => self.channel4.envelope().initial_settings(),
            0xFF22 => self.channel4.polynomial(),
            0xFF23 => self.channel4.control() | 0b10111111,
            0xFF24 => self.master_volume,
            0xFF25 => self.sound_panning,
            0xFF26 => self.audio_master_control(),
            0xFF30..=0xFF3F => self.get_wave_ram_byte((address & 0xF) as u8),
            _ if address < 0xFF10 || address > 0xFF3F => panic!("Invalid APU address: 0x{:04X}", address),
            _ => 0xFF
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.set_ch1_sweep_settings(value),
            0xFF11 => self.set_ch1_length_settings(value),
            0xFF12 => self.set_ch1_envelope_settings(value),
            0xFF13 => self.set_ch1_period_low(value),
            0xFF14 => self.set_ch1_period_high(value),
            0xFF16 => self.set_ch2_length_settings(value),
            0xFF17 => self.set_ch2_envelope_settings(value),
            0xFF18 => self.set_ch2_period_low(value),
            0xFF19 => self.set_ch2_period_high(value),
            0xFF1A => self.set_ch3_dac_enabled(value),
            0xFF1B => self.set_ch3_length_settings(value),
            0xFF1C => self.set_ch3_volume(value),
            0xFF1D => self.set_ch3_period_low(value),
            0xFF1E => self.set_ch3_period_high(value),
            0xFF20 => self.set_ch4_length_settings(value),
            0xFF21 => self.set_ch4_envelope_settings(value),
            0xFF22 => self.set_ch4_polynomial(value),
            0xFF23 => self.set_ch4_control(value),
            0xFF24 => self.set_master_volume(value),
            0xFF25 => self.set_sound_panning(value),
            0xFF26 => self.set_audio_master_control(value),
            0xFF30..=0xFF3F => self.set_wave_ram_byte((address & 0xF) as u8, value),
            _ if address < 0xFF10 || address > 0xFF3F => panic!("Invalid APU address: 0x{:04X}", address),
            _ => ()
        }
    }
}

impl Serializable for Apu {
    fn serialize(&self, writer: &mut dyn Write)-> std::io::Result<()> {
        self.enabled.serialize(writer)?;
        self.sound_panning.serialize(writer)?;
        self.master_volume.serialize(writer)?;
        self.channel1.serialize(writer)?;
        self.channel2.serialize(writer)?;
        self.channel3.serialize(writer)?;
        self.channel4.serialize(writer)?;
        self.divider_apu.serialize(writer)?;
        self.last_divider_time.serialize(writer)?;
        self.audio_buffer_clock.serialize(writer)?;
        self.channel_clock.serialize(writer)?;
        self.enqueue_rate.serialize(writer)?;
        self.cgb_mode.serialize(writer)?;
        self.cgb_double_speed.serialize(writer)?;
        Ok(())
    }

    fn deserialize(&mut self, reader: &mut dyn Read)-> std::io::Result<()> {
        self.enabled.deserialize(reader)?;
        self.sound_panning.deserialize(reader)?;
        self.master_volume.deserialize(reader)?;
        self.channel1.deserialize(reader)?;
        self.channel2.deserialize(reader)?;
        self.channel3.deserialize(reader)?;
        self.channel4.deserialize(reader)?;
        self.divider_apu.deserialize(reader)?;
        self.last_divider_time.deserialize(reader)?;
        self.audio_buffer_clock.deserialize(reader)?;
        self.channel_clock.deserialize(reader)?;
        self.enqueue_rate.deserialize(reader)?;
        self.cgb_mode.deserialize(reader)?;
        self.cgb_double_speed.deserialize(reader)?;

        self.summed_channel1_sample = 0.0;
        self.summed_channel2_sample = 0.0;
        self.summed_channel3_sample = 0.0;
        self.summed_channel4_sample = 0.0;

        self.left_sample_queue.clear();
        self.right_sample_queue.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests;

mod pulse;
mod wave;
mod noise;
mod length;
mod sweep;
mod envelope;
mod period;
mod utils;
