use crate::cpu::{Register, CpuState, REGISTER_HL};
use crate::cpu::microops;

fn rotate_left(cpu_state: &mut CpuState, byte: u8) -> u8 {
    let most_significant_bit = byte >> 7;
    let rotated_value = byte << 1 | most_significant_bit;
    microops::set_flag_z(cpu_state, rotated_value == 0);
    microops::set_flag_n(cpu_state, false);
    microops::set_flag_h(cpu_state, false);
    microops::set_flag_c(cpu_state, most_significant_bit == 0x01);
    rotated_value
}

fn rotate_left_through_carry(cpu_state: &mut CpuState, byte: u8) -> u8 {
    let c_flag = if microops::is_c_flag_set(cpu_state) { 0x1 } else { 0x0 };
    let most_significant_bit = byte >> 7;
    let rotated_value = byte << 1 | c_flag;
    microops::set_flag_z(cpu_state, rotated_value == 0);
    microops::set_flag_n(cpu_state, false);
    microops::set_flag_h(cpu_state, false);
    microops::set_flag_c(cpu_state, most_significant_bit == 0x01);
    rotated_value
}

fn rotate_right(cpu_state: &mut CpuState, byte: u8) -> u8 {
    let least_significant_bit = byte & 0x1;
    let rotated_value: u8 = least_significant_bit << 7 | byte >> 1;
    microops::set_flag_z(cpu_state, rotated_value == 0);
    microops::set_flag_n(cpu_state, false);
    microops::set_flag_h(cpu_state, false);
    microops::set_flag_c(cpu_state, least_significant_bit == 0x01);
    rotated_value
}

fn rotate_right_through_carry(cpu_state: &mut CpuState, byte: u8) -> u8 {
    let c_flag = if microops::is_c_flag_set(cpu_state) { 0x1 } else { 0x0 };
    let least_significant_bit = byte & 0x1;
    let rotated_value = c_flag << 7 | byte >> 1;
    microops::set_flag_z(cpu_state, rotated_value == 0);
    microops::set_flag_n(cpu_state, false);
    microops::set_flag_h(cpu_state, false);
    microops::set_flag_c(cpu_state, least_significant_bit == 0x01);
    rotated_value
}

pub fn rotate_register_left(cpu_state: &mut CpuState, register: Register) {
    let byte = microops::read_from_register(cpu_state, &register);
    let rotated_value = rotate_left(cpu_state, byte);
    microops::store_in_register(cpu_state, register, rotated_value);
}

pub fn rotate_register_left_through_carry(cpu_state: &mut CpuState, register: Register) {
    let byte = microops::read_from_register(cpu_state, &register);
    let rotated_value = rotate_left_through_carry(cpu_state, byte);
    microops::store_in_register(cpu_state, register, rotated_value);
}

pub fn rotate_register_right(cpu_state: &mut CpuState, register: Register) {
    let byte = microops::read_from_register(cpu_state, &register);
    let rotated_value = rotate_right(cpu_state, byte);
    microops::store_in_register(cpu_state, register, rotated_value);
}

pub fn rotate_register_right_through_carry(cpu_state: &mut CpuState, register: Register) {
    let byte = microops::read_from_register(cpu_state, &register);
    let rotated_value = rotate_right_through_carry(cpu_state, byte);
    microops::store_in_register(cpu_state, register, rotated_value);
}

pub fn rotate_memory_byte_left(cpu_state: &mut CpuState) {
    let address = microops::read_from_register_pair(cpu_state, &REGISTER_HL);
    let byte = microops::read_byte_from_memory(cpu_state, address);
    let rotated_value = rotate_left(cpu_state, byte);
    microops::store_byte_in_memory(cpu_state, address, rotated_value);
}

pub fn rotate_memory_byte_left_through_carry(cpu_state: &mut CpuState) {
    let address = microops::read_from_register_pair(cpu_state, &REGISTER_HL);
    let byte = microops::read_byte_from_memory(cpu_state, address);
    let rotated_value = rotate_left_through_carry(cpu_state, byte);
    microops::store_byte_in_memory(cpu_state, address, rotated_value);
}

pub fn rotate_memory_byte_right(cpu_state: &mut CpuState) {
    let address = microops::read_from_register_pair(cpu_state, &REGISTER_HL);
    let byte = microops::read_byte_from_memory(cpu_state, address);
    let rotated_value = rotate_right(cpu_state, byte);
    microops::store_byte_in_memory(cpu_state, address, rotated_value);
}

pub fn rotate_memory_byte_right_through_carry(cpu_state: &mut CpuState) {
    let address = microops::read_from_register_pair(cpu_state, &REGISTER_HL);
    let byte = microops::read_byte_from_memory(cpu_state, address);
    let rotated_value = rotate_right_through_carry(cpu_state, byte);
    microops::store_byte_in_memory(cpu_state, address, rotated_value);
}