use crate::{mmu, utils};
use crate::cpu::{Register, RegisterPair, CpuState};
use crate::emulator::Emulator;
use crate::emulator;
use crate::utils::get_t_cycle_increment;

pub fn step_one_machine_cycle(emulator: &mut Emulator) {
    let double_speed_mode = emulator.speed_switch.cgb_double_speed;
    let t_cycle_increment = get_t_cycle_increment(double_speed_mode);

    emulator.cpu.clock.total_clock_cycles = emulator.cpu.clock.total_clock_cycles.wrapping_add(t_cycle_increment as u32);
    emulator.cpu.clock.instruction_clock_cycles = emulator.cpu.clock.instruction_clock_cycles.wrapping_add(t_cycle_increment);
    
    emulator::sync(emulator);
}

pub fn step_machine_cycles(emulator: &mut Emulator, cycles: u8) {
    for _ in 0..cycles {
        step_one_machine_cycle(emulator);
    }
}

pub fn read_byte_from_memory(emulator: &mut Emulator, address: u16) -> u8 {
    step_one_machine_cycle(emulator);
    let byte = mmu::read_byte(emulator, address);
    byte
}

pub fn read_word_from_memory(emulator: &mut Emulator, address: u16) -> u16 {
    step_one_machine_cycle(emulator);
    let first_byte = mmu::read_byte(emulator, address);

    step_one_machine_cycle(emulator);
    let second_byte = mmu::read_byte(emulator, address + 1);

    utils::as_word(first_byte, second_byte)
}

pub fn store_byte_in_memory(emulator: &mut Emulator, address: u16, byte: u8) {
    step_one_machine_cycle(emulator);
    mmu::write_byte(emulator, address, byte);
}

pub fn store_word_in_memory(emulator: &mut Emulator, address: u16, word: u16) {
    let (first_byte, second_byte) = utils::as_bytes(word);

    step_one_machine_cycle(emulator);
    mmu::write_byte(emulator, address, first_byte);

    step_one_machine_cycle(emulator);
    mmu::write_byte(emulator, address + 1, second_byte);
}

pub fn read_from_register(cpu_state: &CpuState, register: &Register) -> u8 {
    match register {
        Register::A => cpu_state.registers.a,
        Register::B => cpu_state.registers.b,
        Register::C => cpu_state.registers.c,
        Register::D => cpu_state.registers.d,
        Register::E => cpu_state.registers.e,
        Register::F => cpu_state.registers.f,
        Register::H => cpu_state.registers.h,
        Register::L => cpu_state.registers.l
    } 
}

pub fn store_in_register(cpu_state: &mut CpuState, register: Register, value: u8) {
    match register {
        Register::A => cpu_state.registers.a = value,
        Register::B => cpu_state.registers.b = value,
        Register::C => cpu_state.registers.c = value,
        Register::D => cpu_state.registers.d = value,
        Register::E => cpu_state.registers.e = value,
        Register::F => cpu_state.registers.f = value,
        Register::H => cpu_state.registers.h = value,
        Register::L => cpu_state.registers.l = value
    } 
}

pub fn read_from_register_pair(cpu_state: &mut CpuState, register_pair: &RegisterPair) -> u16 {
    let first_byte = read_from_register(cpu_state, &register_pair.first);
    let second_byte = read_from_register(cpu_state, &register_pair.second);
    ((first_byte as u16) << 8) | (second_byte as u16 & 0xFF)
}

pub fn store_in_register_pair(cpu_state: &mut CpuState, register_pair: RegisterPair, value: u16) {
    store_in_register(cpu_state, register_pair.first, ((value >> 8) & 0xFF) as u8);
    store_in_register(cpu_state, register_pair.second, (value & 0xFF) as u8);
}

pub fn set_flag_z(cpu_state: &mut CpuState, flag: bool) {
    if flag {
        cpu_state.registers.f = cpu_state.registers.f | 0x80;
    } else {
        cpu_state.registers.f = cpu_state.registers.f & 0x7F;
    }
}

pub fn set_flag_n(cpu_state: &mut CpuState, flag: bool) {
    if flag {
        cpu_state.registers.f = cpu_state.registers.f | 0x40;
    } else {
        cpu_state.registers.f = cpu_state.registers.f & 0xBF;
    }
}

pub fn set_flag_h(cpu_state: &mut CpuState, flag: bool) {
    if flag {
        cpu_state.registers.f = cpu_state.registers.f | 0x20;
    } else {
        cpu_state.registers.f = cpu_state.registers.f & 0xDF;
    }
}

pub fn set_flag_c(cpu_state: &mut CpuState, flag: bool) {
    if flag {
        cpu_state.registers.f = cpu_state.registers.f | 0x10;
    } else {
        cpu_state.registers.f = cpu_state.registers.f & 0xEF;
    }
}

pub fn is_z_flag_set(cpu_state: &CpuState) -> bool {
    let value = read_from_register(cpu_state, &Register::F);
    (value & 0x80) == 0x80
}

pub fn is_n_flag_set(cpu_state: &CpuState) -> bool {
    let value = read_from_register(cpu_state, &Register::F);
    (value & 0x40) == 0x40
}

pub fn is_h_flag_set(cpu_state: &CpuState) -> bool {
    let value = read_from_register(cpu_state, &Register::F);
    (value & 0x20) == 0x20
}

pub fn is_c_flag_set(cpu_state: &CpuState) -> bool {
    let value = read_from_register(cpu_state, &Register::F);
    (value & 0x10) == 0x10
}