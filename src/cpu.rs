use crate::address_bus::AddressBus;
use crate::serializable::Serializable;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use serializable_derive::Serializable;
use std::io::{Read, Write};

#[derive(Debug, Serializable, CopyGetters, Setters, Default)]
#[getset(get_copy = "pub", set = "pub")]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: u8,
    opcode: u8,
    program_counter: u16,
    stack_pointer: u16
}

#[derive(Debug, Serializable, CopyGetters, Setters)]
pub struct Interrupts {
    enable_delay: u8,
    enabled: bool
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BusActivityType {
    Read,
    Write
}

#[derive(Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BusActivityEntry {
    address: u16,
    value: u8,
    activity_type: BusActivityType
}

#[derive(Getters, MutGetters, CopyGetters)]
pub struct Cpu {
    #[getset(get = "pub(crate)", get_mut = "pub(crate)", set = "pub(super)")]
    registers: Registers,
    halted: bool,
    halt_bug: bool,
    interrupts: Interrupts,
    instruction_clock_cycles: u8,
    #[getset(get = "pub(crate)")]
    opcode_bus_activity: Vec<Option<BusActivityEntry>>,
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    address_bus: AddressBus
}

pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L
}

pub struct RegisterPair {
    pub first: Register,
    pub second: Register
}

pub const REGISTER_AF: RegisterPair = RegisterPair { first: Register::A, second: Register::F };
pub const REGISTER_HL: RegisterPair = RegisterPair { first: Register::H, second: Register::L };
pub const REGISTER_BC: RegisterPair = RegisterPair { first: Register::B, second: Register::C };
pub const REGISTER_DE: RegisterPair = RegisterPair { first: Register::D, second: Register::E }; 

pub(super) fn handle_illegal_opcode(opcode: u8) {
    panic!("Encountered illegal opcode {:#04X}", opcode);
}

impl Cpu {
    pub(super) fn new(address_bus: AddressBus) -> Self {
        Cpu {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                f: 0,
                opcode: 0,
                program_counter: 0,
                stack_pointer: 0
            },
            halted: false,
            halt_bug: false,
            interrupts: Interrupts {
                enable_delay: 0,
                enabled: false
            },
            instruction_clock_cycles: 0,
            opcode_bus_activity: Vec::new(),
            address_bus
        }
    }

    pub(super) fn read_next_instruction_byte(&mut self) -> u8 {
        let byte = self.read_byte_from_memory(self.registers.program_counter);
        self.registers.program_counter += 1;
        byte
    }

    pub(super) fn read_next_instruction_word(&mut self) -> u16 {
        let word = self.read_word_from_memory(self.registers.program_counter);
        self.registers.program_counter += 2;
        word
    }
}

impl Serializable for Cpu {
    fn serialize(&self, writer: &mut dyn Write)-> std::io::Result<()> {
        self.registers.serialize(writer)?;
        self.halted.serialize(writer)?;
        self.halt_bug.serialize(writer)?;
        self.interrupts.serialize(writer)?;
        self.instruction_clock_cycles.serialize(writer)?;
        self.address_bus.serialize(writer)?;
        Ok(())
    }

    fn deserialize(&mut self, reader: &mut dyn Read)-> std::io::Result<()> {
        self.registers.deserialize(reader)?;
        self.halted.deserialize(reader)?;
        self.halt_bug.deserialize(reader)?;
        self.interrupts.deserialize(reader)?;
        self.instruction_clock_cycles.deserialize(reader)?;
        self.address_bus.deserialize(reader)?;
        Ok(())
    }
}

mod microops;
mod alu;
mod bitops;
mod loads;
mod jumps;
pub(crate) mod interrupts;
pub(crate) mod timers;
mod opcodes;
