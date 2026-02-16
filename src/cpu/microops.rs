use crate::utils;
use crate::address_bus::MemoryMapped;
use crate::cpu::{BusActivityEntry, BusActivityType, Register, RegisterPair, Cpu};
use crate::utils::get_t_cycle_increment;

impl Cpu {
    pub(super) fn step_one_machine_cycle(&mut self) {
        let double_speed_mode = self.address_bus.speed_switch().cgb_double_speed();
        let t_cycle_increment = get_t_cycle_increment(double_speed_mode);
        self.instruction_clock_cycles = self.instruction_clock_cycles.wrapping_add(t_cycle_increment);
        self.address_bus.sync();
    }

    pub(super) fn step_machine_cycles(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.step_one_machine_cycle();
        }
    }

    fn record_bus_activity(&mut self, bus_activity_entry: BusActivityEntry) {
        let double_speed_mode = self.address_bus.speed_switch().cgb_double_speed();
        let t_cycle_increment = get_t_cycle_increment(double_speed_mode);
        let current_machine_cycle = (self.instruction_clock_cycles / t_cycle_increment) as usize;
        let recorded_cycles = self.opcode_bus_activity.len();
        let cycles_with_no_activity = (current_machine_cycle - 1) - recorded_cycles;

        for _ in 0..cycles_with_no_activity {
            self.opcode_bus_activity.push(None);
        }

        self.opcode_bus_activity.push(Some(bus_activity_entry));
    }

    fn record_bus_read(&mut self, address: u16, value: u8) {
        let bus_activity_entry = BusActivityEntry {
            address,
            value,
            activity_type: BusActivityType::Read
        };
        self.record_bus_activity(bus_activity_entry);
    }

    fn record_bus_write(&mut self, address: u16, value: u8) {
        let bus_activity_entry = BusActivityEntry {
            address,
            value,
            activity_type: BusActivityType::Write
        };
        self.record_bus_activity(bus_activity_entry);
    }

    pub(super) fn read_byte_from_memory(&mut self, address: u16) -> u8 {
        self.step_one_machine_cycle();
        let byte = self.address_bus.read_byte(address);

        if address == 0x00FE && self.address_bus.in_bios() {
            self.address_bus.set_in_bios(false);
        }

        if self.address_bus.processor_test_mode() {
            self.record_bus_read(address, byte);
        }
        
        byte
    }

    pub(super) fn read_word_from_memory(&mut self, address: u16) -> u16 {
        let first_byte = self.read_byte_from_memory(address);
        let second_byte = self.read_byte_from_memory(address + 1);
        utils::as_word(first_byte, second_byte)
    }

    pub(super) fn store_byte_in_memory(&mut self, address: u16, byte: u8) {
        self.step_one_machine_cycle();
        self.address_bus.write_byte(address, byte);
        
        if self.address_bus.processor_test_mode() {
            self.record_bus_write(address, byte);
        }
    }

    pub(super) fn store_word_in_memory(&mut self, address: u16, word: u16) {
        let (first_byte, second_byte) = utils::as_bytes(word);
        self.store_byte_in_memory(address, first_byte);
        self.store_byte_in_memory(address + 1, second_byte);
    }

    pub(super) fn read_from_register(&self, register: &Register) -> u8 {
        match register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::F => self.registers.f,
            Register::H => self.registers.h,
            Register::L => self.registers.l
        } 
    }

    pub(super) fn store_in_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.registers.a = value,
            Register::B => self.registers.b = value,
            Register::C => self.registers.c = value,
            Register::D => self.registers.d = value,
            Register::E => self.registers.e = value,
            Register::F => self.registers.f = value,
            Register::H => self.registers.h = value,
            Register::L => self.registers.l = value
        } 
    }

    pub(super) fn read_from_register_pair(&self, register_pair: &RegisterPair) -> u16 {
        let first_byte = self.read_from_register(&register_pair.first);
        let second_byte = self.read_from_register(&register_pair.second);
        ((first_byte as u16) << 8) | (second_byte as u16 & 0xFF)
    }

    pub(super) fn store_in_register_pair(&mut self, register_pair: RegisterPair, value: u16) {
        self.store_in_register(register_pair.first, ((value >> 8) & 0xFF) as u8);
        self.store_in_register(register_pair.second, (value & 0xFF) as u8);
    }

    pub(super) fn set_flag_z(&mut self, flag: bool) {
        if flag {
            self.registers.f = self.registers.f | 0x80;
        } else {
            self.registers.f = self.registers.f & 0x7F;
        }
    }

    pub(super) fn set_flag_n(&mut self, flag: bool) {
        if flag {
            self.registers.f = self.registers.f | 0x40;
        } else {
            self.registers.f = self.registers.f & 0xBF;
        }
    }

    pub(super) fn set_flag_h(&mut self, flag: bool) {
        if flag {
            self.registers.f = self.registers.f | 0x20;
        } else {
            self.registers.f = self.registers.f & 0xDF;
        }
    }

    pub(super) fn set_flag_c(&mut self, flag: bool) {
        if flag {
            self.registers.f = self.registers.f | 0x10;
        } else {
            self.registers.f = self.registers.f & 0xEF;
        }
    }

    pub(super) fn is_z_flag_set(&self) -> bool {
        let value = self.read_from_register(&Register::F);
        (value & 0x80) == 0x80
    }

    pub(super) fn is_n_flag_set(&self) -> bool {
        let value = self.read_from_register(&Register::F);
        (value & 0x40) == 0x40
    }

    pub(super) fn is_h_flag_set(&self) -> bool {
        let value = self.read_from_register(&Register::F);
        (value & 0x20) == 0x20
    }

    pub(super) fn is_c_flag_set(&self) -> bool {
        let value = self.read_from_register(&Register::F);
        (value & 0x10) == 0x10
    }

    fn is_oam_address(&self, address: u16) -> bool {
        !self.address_bus.processor_test_mode() && address >= 0xFE00 && address <= 0xFEFF
    }

    pub(super) fn check_oam_bug_write(&mut self, address: u16) {
        if self.is_oam_address(address) {
            self.address_bus.trigger_oam_bug_write();
        }
    }

    pub(super) fn check_oam_bug_read(&mut self, address: u16) {
        if self.is_oam_address(address) {
            self.address_bus.trigger_oam_bug_read();
        }
    }

    pub(super) fn check_oam_bug_mixed(&mut self, address: u16) {
        if self.is_oam_address(address) {
            self.address_bus.trigger_oam_bug_mixed();
        }
    }
}