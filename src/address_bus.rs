use crate::apu::{Apu, ApuParams};
use crate::bios::{CGB_BOOT, DMG_BOOTIX};
use crate::cpu::interrupts::{initialize_interrupt_registers, InterruptRegisters};
use crate::cpu::timers::TimerRegisters;
use crate::gpu::{Gpu, GpuParams};
use crate::joypad::{Key, Joypad};
use crate::address_bus::cartridge::{initialize_cartridge_mapper, CartridgeMapper};
use crate::address_bus::cheats::CheatState;
use crate::address_bus::dma::DMAState;
use crate::address_bus::effects::empty_cartridge_effects;
use crate::address_bus::hdma::HDMAState;
use crate::serializable::Serializable;
use crate::serial::Serial;
use crate::utils::is_bit_set;
use crate::address_bus::speed_switch::SpeedSwitch;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::io::{self, Read, Write};

pub(crate) trait MemoryMapped {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);
}

pub use crate::address_bus::cartridge::CartridgeHeader;
pub use crate::address_bus::effects::CartridgeEffects;
pub use crate::address_bus::mbc3::RTCState;

#[derive(CopyGetters, Getters, MutGetters, Setters)]
pub struct AddressBus {
    #[getset(get_copy = "pub(crate)", set = "pub(super)")]
    in_bios: bool,

    bios: Vec<u8>,
    
    working_ram: [u8; 0x10000],
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    zero_page_ram: [u8; 0x80],
    
    svbk: u8,
    
    cartridge_mapper: Box<dyn CartridgeMapper>,
    
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    processor_test_ram: [u8; 0xFFFF],
    
    #[getset(get_copy = "pub(crate)", set = "pub(super)")]
    processor_test_mode: bool,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    interrupts: InterruptRegisters,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    timers: TimerRegisters,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    gpu: Gpu,

    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    joypad: Joypad,
    
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    apu: Apu,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    dma: DMAState,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    hdma: HDMAState,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    serial: Serial,
    
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    cheats: CheatState,
    
    #[getset(get = "pub(super)", get_mut = "pub(super)")]
    speed_switch: SpeedSwitch,
    
    #[getset(get_copy = "pub(crate)")]
    cgb_mode: bool,
}

impl AddressBus {
    pub(super) fn new(renderer: fn(&[u8]), processor_test_mode: bool) -> AddressBus {
        AddressBus {
            in_bios: true,
            bios: [0; 0x100].to_vec(),
            working_ram: [0; 0x10000],
            zero_page_ram: [0; 0x80],
            svbk: 0,
            cartridge_mapper: initialize_cartridge_mapper(empty_cartridge_effects()),
            processor_test_ram: [0; 0xFFFF],
            processor_test_mode,
            interrupts: initialize_interrupt_registers(),
            timers: TimerRegisters::new(),
            gpu: Gpu::new(renderer),
            joypad: Joypad::new(),
            apu: Apu::new(),
            dma: DMAState::new(),
            hdma: HDMAState::new(),
            serial: Serial::new(),
            cheats: CheatState::new(),
            speed_switch: SpeedSwitch::new(),
            cgb_mode: false,
        }
    }

    pub(crate) fn load_bios(&mut self, is_cgb: bool) {
        self.bios = if is_cgb {
            CGB_BOOT.to_vec()
        }
        else {
            DMG_BOOTIX.to_vec()
        }
    }

    pub(super) fn get_working_ram_bank(&self) -> u8 {
        if self.cgb_mode {
            let masked_value = self.svbk & 0b111;
            if masked_value == 0 { 1 } else { masked_value }
        }
        else {
            1
        }
    }

    fn address_accessible(&self, address: u16) -> bool {
        let accessing_oam = address >= 0xFE00 && address < 0xFEA0;
        (self.dma.in_progress() && !accessing_oam) || !self.dma.in_progress()
    }

    fn calculate_working_ram_index(&self, address: u16) -> usize {
        let localized_index = address & 0x1FFF;
        if localized_index <= 0xFFF {
            localized_index as usize
        }
        else {
            let bank_number = self.get_working_ram_bank();
            let index = (bank_number as u16 * 0x1000) + (address & 0x0FFF);
            index as usize 
        }
    }


    pub(crate) fn load_rom_buffer(&mut self, buffer: Vec<u8>, cartridge_effects: Box<dyn CartridgeEffects>) -> io::Result<CartridgeHeader> {
        let cartridge_result = cartridge::load_rom_buffer(buffer, cartridge_effects); 
        match cartridge_result {
            Ok(mapper) => {
                let cartridge = mapper.get_cartridge();
                let header = cartridge.header.clone();
                self.cartridge_mapper = mapper;
                Ok(header)
            },
            Err(e) => Err(e)
        }
    }

    pub(crate) fn get_cartridge_ram(&self) -> Vec<u8> {
        let cartridge = &self.cartridge_mapper.get_cartridge();
        cartridge.ram.clone()
    }

    pub(crate) fn set_cartridge_ram(&mut self, buffer: Vec<u8>) {
        self.cartridge_mapper.set_cartridge_ram(buffer);
    }

    pub(super) fn sync(&mut self) {
        self.timers.step();
        self.dma_step();
        self.gpu.step(GpuParams {
            hdma: &mut self.hdma,
        });
        self.apu.step(ApuParams {
            divider: self.timers.divider(),
        });
        self.serial.step();
    }

    pub(super) fn trigger_oam_bug_write(&mut self) {
        self.gpu.trigger_oam_bug_write();
    }

    pub(super) fn trigger_oam_bug_read(&mut self) {
        self.gpu.trigger_oam_bug_read();
    }

    pub(super) fn trigger_oam_bug_mixed(&mut self) {
        self.gpu.trigger_oam_bug_mixed();
    }

    pub(crate) fn set_cgb_mode(&mut self, value: bool) {
        self.cgb_mode = value;
        self.apu.set_cgb_mode(value);
        self.gpu.set_cgb_mode(value);
        self.serial.set_cgb_mode(value);
        self.hdma.set_cgb_mode(value);
        self.speed_switch.set_cgb_mode(value);
    }

    pub(crate) fn cartridge_mapper(&self) -> &dyn CartridgeMapper {
        self.cartridge_mapper.as_ref()
    }

    pub(crate) fn handle_key_press(&mut self, key: &Key) {
        self.joypad.handle_key_press(key);
    }

    pub(crate) fn handle_key_release(&mut self, key: &Key) {
        self.joypad.handle_key_release(key);
    }

    pub(super) fn interrupt_flags(&self) -> u8 {
        let mut flags = 0;
        if self.gpu.vblank_interrupt() {
            flags |= 0x1;
        }
        if self.gpu.stat_interrupt() {
            flags |= 0x2;
        }
        if self.timers.interrupt() {
            flags |= 0x4;
        }
        if self.serial.interrupt() {
            flags |= 0x8;
        }
        if self.joypad.interrupt() {
            flags |= 0x10;
        }
        flags | 0xE0
    }

    pub(super) fn set_interrupt_flags(&mut self, flags: u8) {
        self.gpu.set_vblank_interrupt(is_bit_set(flags, 0));
        self.gpu.set_stat_interrupt(is_bit_set(flags, 1));
        self.timers.set_interrupt(is_bit_set(flags, 2));
        self.serial.set_interrupt(is_bit_set(flags, 3));
        self.joypad.set_interrupt(is_bit_set(flags, 4));
    }

    fn svbk(&self) -> u8 {
        if self.cgb_mode {
            self.svbk
        }
        else {
            0xFF
        }
    }

    fn set_svbk(&mut self, value: u8) {
        if self.cgb_mode {
            self.svbk = value;
        }
    }
}

impl MemoryMapped for AddressBus {
    fn read_byte(&self, address: u16) -> u8 {
        if self.processor_test_mode {
            self.processor_test_ram[address as usize]
        }
        else if !self.address_accessible(address) {
            0xFF
        }
        else {
            let byte = match address & 0xF000 {
                0x0000 if address <= 0x00FE && self.in_bios => {
                    self.bios[address as usize]
                },
                0x0000 if address >= 0x0200 && address <= 0x08FF && self.cgb_mode && self.in_bios => {
                    self.bios[address as usize]
                },
                0x0000..=0x7FFF =>
                    self.cartridge_mapper.read_rom(address),
                0x8000..=0x9FFF =>
                    self.gpu.read_byte(address),
                0xA000..=0xBFFF =>
                    self.cartridge_mapper.read_ram(address & 0x1FFF),
                0xC000..=0xEFFF => {
                    let index = self.calculate_working_ram_index(address);
                    self.working_ram[index]
                }
                0xF000 => match address & 0x0F00 {
                    0x000..=0xD00 => {
                        let index = self.calculate_working_ram_index(address);
                        self.working_ram[index]
                    },
                    0xE00 if address < 0xFEA0 => self.gpu.read_byte(address),
                    0xF00 if address == 0xFFFF => self.interrupts.enabled(),
                    0xF00 if address >= 0xFF80 => self.zero_page_ram[(address & 0x7F) as usize],
                    _ => match address & 0xFF {
                        0x00 => self.joypad.read_byte(),
                        0x01..=0x02 => self.serial.read_byte(address),
                        0x10..=0x26 | 0x30..=0x3F => self.apu.read_byte(address),
                        0x40..=0x45 | 0x47..=0x4C | 0x4F | 0x68..=0x6C => self.gpu.read_byte(address),
                        0x46 => self.dma.source(),
                        0x4D => self.speed_switch.key1(),
                        0x55 => self.hdma.read_byte(address),
                        0x70 => self.svbk(),
                        0x0F => self.interrupt_flags(),
                        0x04..=0x07 => self.timers.read_byte(address),
                        _ => 0xFF
                    }
                },
                _ => 0x00,
            };
            self.apply_cheat_if_needed(address, byte)
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        if self.processor_test_mode {
            self.processor_test_ram[address as usize] = value;
        }
        else if self.address_accessible(address) {
            let _ = match address & 0xF000 {
                0x0000..=0x7FFF =>
                    self.cartridge_mapper.write_rom(address, value),
                0x8000..=0x9FFF =>
                    self.gpu.write_byte(address, value),
                0xA000..=0xBFFF =>
                    self.cartridge_mapper.write_ram(address & 0x1FFF, value),
                0xC000..=0xEFFF => {
                    let index = self.calculate_working_ram_index(address);
                    self.working_ram[index] = value;
                },
                0xF000 => match address & 0x0F00 {
                    0x000..=0xD00 => {
                        let index = self.calculate_working_ram_index(address);
                        self.working_ram[index] = value;
                    },
                    0xE00 if address < 0xFEA0 => self.gpu.write_byte(address, value),
                    0xF00 if address == 0xFFFF => { self.interrupts.set_enabled(value); },
                    0xF00 if address >= 0xFF80 => self.zero_page_ram[(address & 0x7F) as usize] = value,
                    _ => match address & 0xFF {
                        0x00 => self.joypad.write_byte(value),
                        0x01..=0x02 => self.serial.write_byte(address, value),
                        0x10..=0x26 | 0x30..=0x3F => self.apu.write_byte(address, value),
                        0x40..=0x45 | 0x47..=0x4C | 0x4F | 0x68..=0x6C => self.gpu.write_byte(address, value),
                        0x46 => self.dma.start_dma(value),
                        0x4D => self.speed_switch.set_key1(value),
                        0x51..=0x55 => self.hdma.write_byte(address, value),
                        0x70 => self.set_svbk(value),
                        0x0F => self.set_interrupt_flags(value),
                        0x04..=0x07 => self.timers.write_byte(address, value),
                        _ => ()
                    }
                },
                _ => (),
            };
        }
    }
}

impl Serializable for AddressBus {
    fn serialize(&self, writer: &mut dyn Write)-> std::io::Result<()> {
        self.in_bios.serialize(writer)?;
        self.working_ram.serialize(writer)?;
        self.zero_page_ram.serialize(writer)?;
        self.svbk.serialize(writer)?;
        self.cartridge_mapper.serialize(writer)?;
        self.interrupts.serialize(writer)?;
        self.timers.serialize(writer)?;
        self.gpu.serialize(writer)?;
        self.apu.serialize(writer)?;
        self.dma.serialize(writer)?;
        self.hdma.serialize(writer)?;
        self.serial.serialize(writer)?;
        self.speed_switch.serialize(writer)?;
        Ok(())
    }

    fn deserialize(&mut self, reader: &mut dyn Read)-> std::io::Result<()> {
        self.in_bios.deserialize(reader)?;
        self.working_ram.deserialize(reader)?;
        self.zero_page_ram.deserialize(reader)?;
        self.svbk.deserialize(reader)?;
        self.cartridge_mapper.deserialize(reader)?;
        self.interrupts.deserialize(reader)?;
        self.timers.deserialize(reader)?;
        self.gpu.deserialize(reader)?;
        self.apu.deserialize(reader)?;
        self.dma.deserialize(reader)?;
        self.hdma.deserialize(reader)?;
        self.serial.deserialize(reader)?;
        self.speed_switch.deserialize(reader)?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::address_bus::AddressBus;
    use crate::address_bus::cartridge::*;
    use crate::address_bus::constants::*;

    pub(crate) fn build_rom(cartridge_type: u8, rom_size_index: u8, ram_size_index: u8) -> Vec<u8> {
        let mut rom_buffer: Vec<u8> = Vec::new();
        let number_of_banks = as_max_banks(rom_size_index) as u32;
        rom_buffer.resize((0x4000 * number_of_banks) as usize, 0);
        rom_buffer[CARTRIDGE_TYPE_ADDRESS] = cartridge_type; 
        rom_buffer[ROM_SIZE_ADDRESS] = rom_size_index;
        rom_buffer[RAM_SIZE_ADDRESS] = ram_size_index;
        rom_buffer
    }

    pub(crate) fn initialize_test_address_bus() -> AddressBus {
        AddressBus::new(|_| {}, false)
    }
}

#[cfg(test)]
mod tests;

pub(crate) mod constants;
pub(crate) mod effects;
mod dma;
pub(crate) mod hdma;
pub(crate) mod cheats;
mod speed_switch;
mod cartridge;
mod huc1;
mod mbc1;
mod mbc3;
mod mbc5;
mod mbc_rom_only;
mod bank_utils;
