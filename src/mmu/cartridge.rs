use core::panic;
use std::io;

use crate::mmu::mbc1;
use crate::mmu::mbc1::{MBC1, initialize_mbc1};
use crate::mmu::mbc3;
use crate::mmu::mbc3::{MBC3, empty_clock, initialize_mbc3};
use crate::mmu::mbc_rom_only;

#[derive(Debug)]
pub struct CartridgeHeader {
    pub sgb_support: bool,
    pub type_code: u8,
    pub max_banks: u16
}

#[derive(Debug)]
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: [u8; 0x8000],
    pub header: CartridgeHeader,
    pub mbc1: MBC1,
    pub mbc3: MBC3,
}

const ENTRY_POINT_ADDRESS: usize = 0x100;
const SGB_SUPPORT_ADDRESS: usize = 0x146;
const CARTRIDGE_TYPE_ADDRESS: usize = 0x147;
const ROM_SIZE_ADDRESS: usize = 0x148;

pub const CART_TYPE_ROM_ONLY: u8 = 0x0;
pub const CART_TYPE_MBC1: u8 = 0x1;
pub const CART_TYPE_MBC1_WITH_RAM: u8 = 0x2;
pub const CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY: u8 = 0x3;
pub const CART_TYPE_MBC3_TIMER_BATTERY: u8 = 0xF;
pub const CART_TYPE_MBC3_TIMER_RAM_BATTERY: u8 = 0x10;
pub const CART_TYPE_MBC3: u8 = 0x11;
pub const CART_TYPE_MBC3_RAM: u8 = 0x12;
pub const CART_TYPE_MBC3_RAM_BATTERY: u8 = 0x13;

pub const SUPPORTED_CARTRIDGE_TYPES: [u8; 9] = [CART_TYPE_ROM_ONLY,
    CART_TYPE_MBC1,
    CART_TYPE_MBC1_WITH_RAM,
    CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY,
    CART_TYPE_MBC3_TIMER_BATTERY,
    CART_TYPE_MBC3_TIMER_RAM_BATTERY,
    CART_TYPE_MBC3,
    CART_TYPE_MBC3_RAM,
    CART_TYPE_MBC3_RAM_BATTERY];

pub fn initialize_cartridge() -> Cartridge {
    Cartridge {
        rom: Vec::new(),
        ram: [0; 0x8000],
        header: CartridgeHeader {
            sgb_support: false,
            type_code: 0,
            max_banks: 0,
        },
        mbc1: initialize_mbc1(),
        mbc3: initialize_mbc3(empty_clock), // TODO: Initialize with actual clock
    }
}

fn cartridge_type_supported(type_code: u8) -> bool {
    SUPPORTED_CARTRIDGE_TYPES.contains(&type_code)
}

fn as_max_banks(rom_size_index: u8) -> u16 {
    (2 as u16).pow(rom_size_index as u32 + 1)
}

fn is_mbc1(type_code: u8) -> bool {
    matches!(type_code, CART_TYPE_MBC1
        | CART_TYPE_MBC1_WITH_RAM
        | CART_TYPE_MBC1_WITH_RAM_PLUS_BATTERY)
}

fn is_mbc3(type_code: u8) -> bool {
    matches!(type_code, CART_TYPE_MBC3
        | CART_TYPE_MBC3_RAM
        | CART_TYPE_MBC3_RAM_BATTERY
        | CART_TYPE_MBC3_TIMER_BATTERY
        | CART_TYPE_MBC3_TIMER_RAM_BATTERY)
}

pub fn load_rom_buffer(buffer: Vec<u8>) -> io::Result<Cartridge> {
    if buffer.len() > ENTRY_POINT_ADDRESS {
        let type_code = buffer[CARTRIDGE_TYPE_ADDRESS];
        let sgb_support = buffer[SGB_SUPPORT_ADDRESS] == 0x03;
        let rom_size = buffer[ROM_SIZE_ADDRESS];

        if cartridge_type_supported(type_code) {
            let cartridge = Cartridge {
                rom: buffer,
                ram: [0; 0x8000],
                header: CartridgeHeader {
                    sgb_support,
                    type_code,
                    max_banks: as_max_banks(rom_size),
                },
                mbc1: initialize_mbc1(),
                mbc3: initialize_mbc3(empty_clock), // TODO: Initialize with actual clock
            };

            Ok(cartridge)
        } else {
            let error_message = format!("Unsupported cartridge type {type_code}.");
            Err(io::Error::new(io::ErrorKind::Other, error_message))
        }
    } else {
        let error_message = "Buffer is too small to contain a valid ROM.";
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}

pub fn read_rom(cartridge: &Cartridge, address: u16) -> u8 {
    let cartridge_type_code = cartridge.header.type_code;

    if cartridge_type_code == CART_TYPE_ROM_ONLY {
        mbc_rom_only::read_rom(cartridge, address)
    } else if is_mbc1(cartridge_type_code) {
        mbc1::read_rom(cartridge, address)
    } else if is_mbc3(cartridge_type_code) {
        mbc3::read_rom(cartridge, address)
    } else {
        panic!("Unsupported cartridge type: {}", cartridge.header.type_code);
    }
 }

pub fn write_rom(cartridge: &mut Cartridge, address: u16, value: u8) {
    let cartridge_type_code = cartridge.header.type_code;

    if cartridge_type_code == CART_TYPE_ROM_ONLY {
        mbc_rom_only::write_rom(cartridge, address, value);
    } else if is_mbc1(cartridge_type_code) {
        mbc1::write_rom(cartridge, address, value);
    } else if is_mbc3(cartridge_type_code) {
        mbc3::write_rom(cartridge, address, value);
    } else {
        panic!("Unsupported cartridge type: {}", cartridge.header.type_code);
    }
}

pub fn read_ram(cartridge: &Cartridge, address: u16) -> u8 {
    let cartridge_type_code = cartridge.header.type_code;

    if cartridge_type_code == CART_TYPE_ROM_ONLY {
        mbc_rom_only::read_ram(cartridge, address)
    } else if is_mbc1(cartridge_type_code) {
        mbc1::read_ram(cartridge, address)
    } else if is_mbc3(cartridge_type_code) {
        mbc3::read_ram(cartridge, address)
    } else {
        panic!("Unsupported cartridge type: {}", cartridge.header.type_code);
    }
}

pub fn write_ram(cartridge: &mut Cartridge, address: u16, value: u8) {
    let cartridge_type_code = cartridge.header.type_code;

    if cartridge_type_code == CART_TYPE_ROM_ONLY {
        mbc_rom_only::write_ram(cartridge, address, value);
    } else if is_mbc1(cartridge_type_code) {
        mbc1::write_ram(cartridge, address, value);
    } else if is_mbc3(cartridge_type_code) {
        mbc3::write_ram(cartridge, address, value);
    } else {
        panic!("Unsupported cartridge type: {}", cartridge.header.type_code);
    }
}
