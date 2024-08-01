use crate::{apu, dma};
use crate::emulator::Emulator;
use crate::keys;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum MBCMode {
    ROM,
    RAM
}

#[derive(Debug)]
pub struct Memory {
    pub in_bios: bool,
    pub bios: [u8; 0x100],
    pub rom: Vec<u8>,
    pub video_ram: [u8; 0x2000],
    pub object_attribute_memory: [u8; 0xa0],
    pub working_ram: [u8; 0x3e00],
    pub external_ram: [u8; 0x8000],
    pub zero_page_ram: [u8; 0x80],
    pub cartridge_header: CartridgeHeader,
    pub ram_enabled: bool,
    pub rom_bank_number: u8,
    pub ram_bank_number: u8,
    pub mbc_mode: MBCMode
}

#[derive(Debug)]
pub struct CartridgeHeader {
    pub sgb_support: bool,
    pub type_code: u8
}

const ENTRY_POINT_ADDRESS: usize = 0x100;
const SGB_SUPPORT_ADDRESS: usize = 0x146;
const CARTRIDGE_TYPE_ADDRESS: usize = 0x147;

pub const CART_TYPE_ROM_ONLY: u8 = 0;
pub const CART_TYPE_MBC1: u8 = 1;
pub const CART_TYPE_MBC1_WITH_RAM: u8 = 2;
pub const CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY: u8 = 3;

pub const SUPPORTED_CARTRIDGE_TYPES: [u8; 4] = [CART_TYPE_ROM_ONLY,
    CART_TYPE_MBC1,
    CART_TYPE_MBC1_WITH_RAM,
    CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY]; 

pub fn initialize_memory() -> Memory {
    Memory {
        in_bios: true,
        bios: [0; 0x100],
        rom: Vec::new(),
        video_ram: [0; 0x2000],
        object_attribute_memory: [0; 0xa0],
        working_ram: [0; 0x3e00],
        external_ram: [0; 0x8000],
        zero_page_ram: [0; 0x80],
        cartridge_header: CartridgeHeader {
            sgb_support: false,
            type_code: 0,
        },
        ram_enabled: false,
        rom_bank_number: 1,
        ram_bank_number: 0,
        mbc_mode: MBCMode::ROM 
    }
}

fn address_accessible(emulator: &Emulator, address: u16) -> bool {
    let accessing_oam = address >= 0xFE00 && address < 0xFEA0;
    (emulator.dma.in_progress && !accessing_oam) || !emulator.dma.in_progress
}

pub fn read_byte(emulator: &Emulator, address: u16) -> u8 {
    if address_accessible(emulator, address) {
        let memory = &emulator.memory;

        match address & 0xF000 {
            0x0000 if address < 0x0100 && memory.in_bios => memory.bios[address as usize],
            0x0000..=0x3FFF => memory.rom[address as usize],
            0x4000..=0x7FFF => {
                let calculated_address = (memory.rom_bank_number as u16 * 0x4000) + (address & 0x3FFF);
                memory.rom[calculated_address as usize]
            },
            0x8000..=0x9FFF => memory.video_ram[(address & 0x1FFF) as usize],
            0xA000..=0xBFFF => {
                let calculated_address = (memory.ram_bank_number as u16 * 0x2000) + (address & 0x1FFF);
                if memory.ram_enabled {
                    memory.external_ram[calculated_address as usize] 
                }
                else {
                    0xFF
                }
            },
            0xC000..=0xEFFF => memory.working_ram[(address & 0x1FFF) as usize],
            0xF000 => match address & 0x0F00 {
                0x000..=0xD00 => memory.working_ram[(address & 0x1FFF) as usize],
                0xE00 if address < 0xFEA0 => memory.object_attribute_memory[(address & 0xFF) as usize],
                0xF00 if address == 0xFFFF => emulator.interrupts.enabled,
                0xF00 if address >= 0xFF80 => memory.zero_page_ram[(address & 0x7F) as usize],
                _ => match address & 0xFF {
                    0x00 => keys::read_joyp_byte(&emulator.keys),
                    0x10 => emulator.apu.channel1.sweep.initial_settings | 0b10000000,
                    0x11 => emulator.apu.channel1.length.initial_settings | 0b00111111,
                    0x12 => emulator.apu.channel1.envelope.initial_settings,
                    0x14 => emulator.apu.channel1.period.high | 0b10111111,
                    0x16 => emulator.apu.channel2.length.initial_settings | 0b00111111,
                    0x17 => emulator.apu.channel2.envelope.initial_settings,
                    0x19 => emulator.apu.channel2.period.high | 0b10111111,
                    0x1A => if emulator.apu.channel3.dac_enabled { 0b11111111 } else { 0b01111111 },
                    0x1C => emulator.apu.channel3.volume | 0b10011111,
                    0x1E => emulator.apu.channel3.period.high | 0b10111111,
                    0x21 => emulator.apu.channel4.envelope.initial_settings,
                    0x22 => emulator.apu.channel4.polynomial,
                    0x23 => emulator.apu.channel4.control | 0b10111111,
                    0x24 => emulator.apu.master_volume,
                    0x25 => emulator.apu.sound_panning,
                    0x26 => apu::get_audio_master_control(&emulator),
                    0x30..=0x3F => apu::get_wave_ram_byte(&emulator, (address & 0xF) as u8),
                    0x40 => emulator.gpu.registers.lcdc,
                    0x41 => emulator.gpu.registers.stat,
                    0x42 => emulator.gpu.registers.scy,
                    0x43 => emulator.gpu.registers.scx,
                    0x44 => emulator.gpu.registers.ly,
                    0x45 => emulator.gpu.registers.lyc,
                    0x46 => dma::get_source(emulator),
                    0x47 => emulator.gpu.registers.palette,
                    0x48 => emulator.gpu.registers.obp0,
                    0x49 => emulator.gpu.registers.obp1,
                    0x4A => emulator.gpu.registers.wy,
                    0x4B => emulator.gpu.registers.wx,
                    0x0F => emulator.interrupts.flags,
                    0x04 => emulator.timers.divider,
                    0x05 => emulator.timers.counter,
                    0x06 => emulator.timers.modulo,
                    0x07 => emulator.timers.control,
                    _ => 0xFF
                }
            },
            _ => 0x00,
        }
    }
    else {
        0xFF
    }
}

pub fn write_byte(emulator: &mut Emulator, address: u16, value: u8) {
    if address_accessible(emulator, address) {
        let memory = &mut emulator.memory;

        match address & 0xF000 {
            0x0000 if address < 0x0100 && memory.in_bios => memory.bios[address as usize] = value,
            0x0000..=0x1FFF => {
                match memory.cartridge_header.type_code {
                    CART_TYPE_MBC1_WITH_RAM | CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY => {
                        memory.ram_enabled = (value & 0xF) == 0xA;
                    }
                    _ => ()
                }
            },
            0x2000..=0x3FFF => {
                match memory.cartridge_header.type_code {
                    CART_TYPE_MBC1 | CART_TYPE_MBC1_WITH_RAM | CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY => {
                        let bank_value = if value == 0 { 1 as u8 } else { value };
                        memory.rom_bank_number = (memory.rom_bank_number & 0x60) + (bank_value & 0x1F);
                    },
                    _ => ()
                }
            },
            0x4000..=0x5FFF => {
                match memory.cartridge_header.type_code {
                    CART_TYPE_MBC1 | CART_TYPE_MBC1_WITH_RAM | CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY => {
                        if memory.mbc_mode == MBCMode::RAM {
                            memory.ram_bank_number = value & 0x3;
                        }
                        else {
                            memory.rom_bank_number = ((value & 0x3) << 5) + (memory.rom_bank_number & 0x1F);
                        }
                    },
                    _ => ()
                }
            },
            0x6000..=0x7FFF => {
                match memory.cartridge_header.type_code {
                    CART_TYPE_MBC1_WITH_RAM | CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY => {
                        memory.mbc_mode = if value == 1 { MBCMode::RAM } else { MBCMode::ROM }
                    }
                    _ => ()
                }
            },
            0x8000..=0x9FFF => memory.video_ram[(address & 0x1FFF) as usize] = value,
            0xA000..=0xBFFF => 
                if memory.ram_enabled {
                    memory.external_ram[(address & 0x1FFF) as usize] = value
                },
            0xC000..=0xEFFF => memory.working_ram[(address & 0x1FFF) as usize] = value,
            0xF000 => match address & 0x0F00 {
                0x000..=0xD00 => memory.working_ram[(address & 0x1FFF) as usize] = value,
                0xE00 if address < 0xFEA0 => memory.object_attribute_memory[(address & 0xFF) as usize]= value,
                0xF00 if address == 0xFFFF => emulator.interrupts.enabled = value,
                0xF00 if address >= 0xFF80 => memory.zero_page_ram[(address & 0x7F) as usize] = value,
                _ => match address & 0xFF {
                    0x00 => keys::write_joyp_byte(&mut emulator.keys, value),
                    0x10 => apu::set_ch1_sweep_settings(emulator, value),
                    0x11 => apu::set_ch1_length_settings(emulator, value),
                    0x12 => apu::set_ch1_envelope_settings(emulator, value),
                    0x13 => apu::set_ch1_period_low(emulator, value),
                    0x14 => apu::set_ch1_period_high(emulator, value),
                    0x16 => apu::set_ch2_length_settings(emulator, value),
                    0x17 => apu::set_ch2_envelope_settings(emulator, value),
                    0x18 => apu::set_ch2_period_low(emulator, value),
                    0x19 => apu::set_ch2_period_high(emulator, value),
                    0x1A => apu::set_ch3_dac_enabled(emulator, value),
                    0x1B => apu::set_ch3_length_settings(emulator, value),
                    0x1C => apu::set_ch3_volume(emulator, value),
                    0x1D => apu::set_ch3_period_low(emulator, value),
                    0x1E => apu::set_ch3_period_high(emulator, value),
                    0x20 => apu::set_ch4_length_settings(emulator, value),
                    0x21 => apu::set_ch4_envelope_settings(emulator, value),
                    0x22 => apu::set_ch4_polynomial(emulator, value),
                    0x23 => apu::set_ch4_control(emulator, value),
                    0x24 => apu::set_master_volume(emulator, value),
                    0x25 => apu::set_sound_panning(emulator, value),
                    0x26 => apu::set_audio_master_control(emulator, value),
                    0x30..=0x3F => apu::set_wave_ram_byte(emulator, (address & 0xF) as u8, value),
                    0x40 => emulator.gpu.registers.lcdc = value,
                    0x41 => emulator.gpu.registers.stat = value,
                    0x42 => emulator.gpu.registers.scy = value,
                    0x43 => emulator.gpu.registers.scx = value,
                    0x44 => emulator.gpu.registers.ly = value,
                    0x45 => emulator.gpu.registers.lyc = value,
                    0x46 => dma::start_dma(emulator, value),
                    0x47 => emulator.gpu.registers.palette = value,
                    0x48 => emulator.gpu.registers.obp0 = value,
                    0x49 => emulator.gpu.registers.obp1 = value,
                    0x4A => emulator.gpu.registers.wy = value,
                    0x4B => emulator.gpu.registers.wx = value,
                    0x0F => emulator.interrupts.flags = value,
                    0x04 => emulator.timers.divider = value,
                    0x05 => emulator.timers.counter = value,
                    0x06 => emulator.timers.modulo = value,
                    0x07 => emulator.timers.control = value,
                    _ => ()
                }
            },
            _ => (),
        }
    }
}

pub fn read_word(emulator: &Emulator, address: u16) -> u16 {
    let first_byte = read_byte(&emulator, address) as u16;
    let second_byte = read_byte(&emulator, address + 1) as u16;
    first_byte + (second_byte << 8)
}

pub fn write_word(emulator: &mut Emulator, address: u16, value: u16) {
    let first_byte = value & 0xFF;
    let second_byte = value >> 8;
    write_byte(emulator, address, first_byte as u8);
    write_byte(emulator, address + 1, second_byte as u8);
}

pub fn cartridge_type_supported(type_code: u8) -> bool {
    SUPPORTED_CARTRIDGE_TYPES.contains(&type_code)
}

pub fn load_rom_buffer(memory: &mut Memory, buffer: Vec<u8>) {
    if buffer.len() > ENTRY_POINT_ADDRESS {
        memory.cartridge_header.sgb_support = buffer[SGB_SUPPORT_ADDRESS] == 0x03;
        memory.cartridge_header.type_code = buffer[CARTRIDGE_TYPE_ADDRESS];
    } 
    memory.rom = buffer; 
}

pub fn load_bios_buffer_slice(memory: &mut Memory, buffer_slice: &[u8]) {
    let mut buffer: [u8; 256] = [0; 256];
    buffer.copy_from_slice(buffer_slice);
    memory.bios = buffer;
}

#[cfg(test)]
mod tests;
