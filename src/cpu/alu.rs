use crate::cpu::{Register, Cpu};

use super::{REGISTER_HL, RegisterPair};

impl Cpu {
    pub(super) fn add_value_to_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let sum = byte.wrapping_add(value);

        self.store_in_register(register, sum);

        self.set_flag_z(sum == 0);
        self.set_flag_n(false);
        self.set_flag_h((value & 0xF) + (byte & 0xF) > 0xF);
        self.set_flag_c((value as u16 + byte as u16) > 0xFF);
    }

    pub(super) fn add_value_to_register_pair(&mut self, register_pair: RegisterPair, value: u16) {
        let word = self.read_from_register_pair(&register_pair);
        let sum = word.wrapping_add(value);

        self.store_in_register_pair(register_pair, sum);

        self.set_flag_n(false);
        self.set_flag_h((value & 0xFFF) + (word & 0xFFF) > 0xFFF);
        self.set_flag_c((value as u32 + word as u32) > 0xFFFF);

        self.step_one_machine_cycle();
    }

    pub(super) fn add_value_and_carry_to_register(&mut self, register: Register, value: u8) {
        let carry_value = if self.is_c_flag_set() { 1 as u8 } else { 0 as u8 };
        let byte = self.read_from_register(&register);
        let sum = byte.wrapping_add(value).wrapping_add(carry_value);

        self.store_in_register(register, sum);

        self.set_flag_z(sum == 0);
        self.set_flag_n(false);
        self.set_flag_h(((value & 0xF) + (byte & 0xF) + carry_value) > 0xF);
        self.set_flag_c((value as u16 + byte as u16 + carry_value as u16) > 0xFF);
    }

    pub(super) fn subtract_value_from_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let difference = byte.wrapping_sub(value);

        self.store_in_register(register, difference);

        self.set_flag_z(difference == 0);
        self.set_flag_n(true);
        self.set_flag_h((byte & 0xF) < (value & 0xF));
        self.set_flag_c(byte < value);
    }

    pub(super) fn subtract_value_and_carry_from_register(&mut self, register: Register, value: u8) {
        let carry_value = if self.is_c_flag_set() { 1 as u8 } else { 0 as u8 };
        let byte = self.read_from_register(&register);
        let difference = byte.wrapping_sub(value).wrapping_sub(carry_value);

        self.store_in_register(register, difference);

        self.set_flag_z(difference == 0);
        self.set_flag_n(true);
        self.set_flag_h((byte & 0xF) < ((value & 0xF) + carry_value));
        self.set_flag_c(((value as u16) + (carry_value as u16)) > (byte as u16));
    }

    pub(super) fn logical_and_with_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let result = byte & value;

        self.store_in_register(register, result);

        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(true);
        self.set_flag_c(false);
    }

    pub(super) fn logical_or_with_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let result = byte | value;

        self.store_in_register(register, result);

        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
    }

    pub(super) fn logical_xor_with_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let result = byte ^ value;

        self.store_in_register(register, result);

        self.set_flag_z(result == 0);
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.set_flag_c(false);
    }

    pub(super) fn compare_value_with_register(&mut self, register: Register, value: u8) {
        let byte = self.read_from_register(&register);
        let difference = byte.wrapping_sub(value);

        self.set_flag_z(difference == 0);
        self.set_flag_n(true);
        self.set_flag_h((byte & 0xF) < (value & 0xF));
        self.set_flag_c(byte < value);
    }

    pub(super) fn increment_register(&mut self, register: Register) {
        let byte = self.read_from_register(&register);
        let sum = byte.wrapping_add(1);

        self.store_in_register(register, sum);

        self.set_flag_z(sum == 0);
        self.set_flag_n(false);
        self.set_flag_h((1 + (byte & 0xF)) > 0xF);
    }

    pub(super) fn decrement_register(&mut self, register: Register) {
        let byte = self.read_from_register(&register);
        let difference = byte.wrapping_sub(1);

        self.store_in_register(register, difference);

        self.set_flag_z(difference == 0);
        self.set_flag_n(true);
        self.set_flag_h((byte & 0xF) < 1);
    }

    pub(super) fn increment_memory_byte(&mut self) {
        let address = self.read_from_register_pair(&REGISTER_HL);
        let byte = self.read_byte_from_memory(address);
        let sum = byte.wrapping_add(1);

        self.store_byte_in_memory(address, sum);

        self.set_flag_z(sum == 0);
        self.set_flag_n(false);
        self.set_flag_h((1 + (byte & 0xF)) > 0xF);
    }

    pub(super) fn decrement_memory_byte(&mut self) {
        let address = self.read_from_register_pair(&REGISTER_HL);
        let byte = self.read_byte_from_memory(address);
        let difference = byte.wrapping_sub(1);

        self.store_byte_in_memory(address, difference);

        self.set_flag_z(difference == 0);
        self.set_flag_n(true);
        self.set_flag_h((byte & 0xF) < 1);
    }

    pub(super) fn increment_register_pair(&mut self, register_pair: RegisterPair) {
        let word = self.read_from_register_pair(&register_pair);
        let sum = word.wrapping_add(1);
        self.store_in_register_pair(register_pair, sum);
        self.step_one_machine_cycle();
        self.check_oam_bug_write(word);
    }

    pub(super) fn decrement_register_pair(&mut self, register_pair: RegisterPair) {
        let word = self.read_from_register_pair(&register_pair);
        let sum = word.wrapping_sub(1);
        self.store_in_register_pair(register_pair, sum);
        self.step_one_machine_cycle();
        self.check_oam_bug_write(word);
    }

    pub(super) fn bcd_adjust(&mut self) {
        let c_flag = self.is_c_flag_set();
        let n_flag = self.is_n_flag_set();
        let h_flag = self.is_h_flag_set();

        let mut value = self.read_from_register(&Register::A);

        if n_flag {
            if c_flag {
                value = value.wrapping_sub(0x60);
            }
            if h_flag {
                value = value.wrapping_sub(0x6);
            }
        }
        else {
            if c_flag || value > 0x99 {
                value = value.wrapping_add(0x60);
                self.set_flag_c(true);
            }
            if h_flag || (value & 0xF) > 0x9 {
                value = value.wrapping_add(0x6);
            }
        }

        self.store_in_register(Register::A, value);

        self.set_flag_z(value == 0);
        self.set_flag_h(false);
    }
}
