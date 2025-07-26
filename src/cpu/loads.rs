use crate::cpu::{Register, RegisterPair, CpuState, read_next_instruction_byte};
use crate::cpu::microops;

pub fn load_immediate_value(cpu_state: &mut CpuState, register: Register) {
    let immediate_byte = read_next_instruction_byte(cpu_state);
    microops::store_in_register(cpu_state, register, immediate_byte);
}

pub fn load_source_register_in_destination_register(cpu_state: &mut CpuState, source: Register, destination: Register) {
    let source_value = microops::read_from_register(cpu_state, &source);
    microops::store_in_register(cpu_state, destination, source_value);
}

pub fn load_memory_byte_in_destination_register(cpu_state: &mut CpuState, address: u16, destination: Register) {
    let byte = microops::read_byte_from_memory(cpu_state, address);
    microops::store_in_register(cpu_state, destination, byte);
}

pub fn load_source_register_in_memory(cpu_state: &mut CpuState, source: Register, address: u16) {
    let byte = microops::read_from_register(cpu_state, &source);
    microops::store_byte_in_memory(cpu_state, address, byte);
}

pub fn load_immediate_value_in_memory(cpu_state: &mut CpuState, register_pair: RegisterPair) {
    let address = microops::read_from_register_pair(cpu_state, &register_pair);
    let immediate_byte = read_next_instruction_byte(cpu_state);
    microops::store_byte_in_memory(cpu_state, address, immediate_byte);
}

pub fn push_word_to_stack(cpu_state: &mut CpuState, word: u16) {
    microops::step_one_machine_cycle(cpu_state);
    cpu_state.registers.stack_pointer = cpu_state.registers.stack_pointer - 1;
    microops::store_byte_in_memory(cpu_state, cpu_state.registers.stack_pointer, (word >> 8) as u8);
    cpu_state.registers.stack_pointer = cpu_state.registers.stack_pointer - 1;
    microops::store_byte_in_memory(cpu_state, cpu_state.registers.stack_pointer, (word & 0xFF) as u8);
}

pub fn push_register_pair_to_stack(cpu_state: &mut CpuState, register_pair: RegisterPair) {
    let word = microops::read_from_register_pair(cpu_state, &register_pair);
    push_word_to_stack(cpu_state, word);
}

pub fn pop_word_from_stack(cpu_state: &mut CpuState) -> u16 {
    let first_byte = microops::read_byte_from_memory(cpu_state, cpu_state.registers.stack_pointer) as u16;
    cpu_state.registers.stack_pointer = cpu_state.registers.stack_pointer + 1;
    let second_byte = microops::read_byte_from_memory(cpu_state, cpu_state.registers.stack_pointer) as u16;
    cpu_state.registers.stack_pointer = cpu_state.registers.stack_pointer + 1;
    (second_byte << 8) + first_byte
}

pub fn pop_word_into_register_pair_from_stack(cpu_state: &mut CpuState, register_pair: RegisterPair) {
    let word = pop_word_from_stack(cpu_state);
    microops::store_in_register_pair(cpu_state, register_pair, word);
}