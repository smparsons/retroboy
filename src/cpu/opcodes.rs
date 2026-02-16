use crate::cpu::{Cpu, Register, REGISTER_AF, REGISTER_BC, REGISTER_DE, REGISTER_HL, handle_illegal_opcode};

impl Cpu {
    fn emulate_halt_bug(&mut self) {
        // Mimics halt bug behavior, which runs the instruction after HALT twice.
        if self.halt_bug {
            self.registers.program_counter -= 1;
            self.halt_bug = false;
        }
    }

    fn enable_interrupts_after_delay(&mut self) {
        if self.interrupts.enable_delay > 0 {
            if self.interrupts.enable_delay == 1 {
                self.interrupts.enabled = true;
            }
            self.interrupts.enable_delay -= 1;
        }
    }

    fn reset_instruction_clock_cycles(&mut self) {
        self.instruction_clock_cycles = 0;
    }

    fn reset_last_opcode_bus_activity(&mut self) {
        if self.address_bus.processor_test_mode() {
            self.opcode_bus_activity.clear();
        }
    }

    fn prefetch_next_opcode(&mut self) {
        let next_opcode = self.read_next_instruction_byte();
        self.registers.opcode = next_opcode;
    }

    pub(crate) fn step(&mut self) {
        if self.address_bus.hdma().in_progress() {
            self.address_bus.hdma_step();
        }

        if self.halted {
            self.step_one_machine_cycle();
        }
        else {
            self.execute_opcode();
        }

        self.interrupt_step();

        if !self.halted {
            self.prefetch_next_opcode();
        }
    }

    fn execute_opcode(&mut self) {
        self.reset_instruction_clock_cycles();
        self.reset_last_opcode_bus_activity();

        let opcode = self.registers.opcode;

        self.emulate_halt_bug();
        self.enable_interrupts_after_delay();

        match opcode {
            0x00 =>
                (),
            0x01 => {
                let word = self.read_next_instruction_word();
                self.store_in_register_pair(REGISTER_BC, word);
            },
            0x02 => {
                let address = self.read_from_register_pair(&REGISTER_BC);
                self.load_source_register_in_memory(Register::A, address);
            },
            0x03 =>
                self.increment_register_pair(REGISTER_BC),
            0x04 =>
                self.increment_register(Register::B),
            0x05 =>
                self.decrement_register(Register::B),
            0x06 =>
                self.load_immediate_value(Register::B),
            0x07 => {
                self.rotate_register_left(Register::A);
                self.set_flag_z(false);
            },
            0x08 => {
                let address = self.read_next_instruction_word();
                self.store_word_in_memory(address, self.registers.stack_pointer);
            },
            0x09 => {
                let word = self.read_from_register_pair(&REGISTER_BC);
                self.add_value_to_register_pair(REGISTER_HL, word);
            },
            0x0A => {
                let address = self.read_from_register_pair(&REGISTER_BC);
                self.load_memory_byte_in_destination_register(address, Register::A);
            },
            0x0B =>
                self.decrement_register_pair(REGISTER_BC),
            0x0C =>
                self.increment_register(Register::C),
            0x0D =>
                self.decrement_register(Register::C),
            0x0E =>
                self.load_immediate_value(Register::C),
            0x0F => {
                self.rotate_register_right(Register::A);
                self.set_flag_z(false);
            },
            0x10 =>
                self.address_bus.toggle_speed_switch(),
            0x11 => {
                let word = self.read_next_instruction_word();
                self.store_in_register_pair(REGISTER_DE, word);
            },
            0x12 => {
                let address = self.read_from_register_pair(&REGISTER_DE);
                self.load_source_register_in_memory(Register::A, address);
            },
            0x13 =>
                self.increment_register_pair(REGISTER_DE),
            0x14 =>
                self.increment_register(Register::D),
            0x15 =>
                self.decrement_register(Register::D),
            0x16 =>
                self.load_immediate_value(Register::D),
            0x17 => {
                self.rotate_register_left_through_carry(Register::A);
                self.set_flag_z(false);
            },
            0x18 => {
                let byte = self.read_next_instruction_byte() as i8;
                let original_program_counter = self.registers.program_counter;
                self.registers.program_counter = original_program_counter.wrapping_add_signed(byte.into());
                self.step_one_machine_cycle();
            },
            0x19 => {
                let word = self.read_from_register_pair(&REGISTER_DE);
                self.add_value_to_register_pair(REGISTER_HL, word);
            },
            0x1A => {
                let address = self.read_from_register_pair(&REGISTER_DE);
                self.load_memory_byte_in_destination_register(address, Register::A)
            },
            0x1B =>
                self.decrement_register_pair(REGISTER_DE),
            0x1C =>
                self.increment_register(Register::E),
            0x1D =>
                self.decrement_register(Register::E),
            0x1E =>
                self.load_immediate_value(Register::E),
            0x1F => {
                self.rotate_register_right_through_carry(Register::A);
                self.set_flag_z(false);
            },
            0x20 =>
                self.conditional_relative_jump(!self.is_z_flag_set()),
            0x21 => {
                let word = self.read_next_instruction_word();
                self.store_in_register_pair(REGISTER_HL, word);
            },
            0x22 => {
                let mut address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::A, address);
                self.check_oam_bug_write(address);
                address = address.wrapping_add(1);
                self.store_in_register_pair(REGISTER_HL, address);
            },
            0x23 =>
                self.increment_register_pair(REGISTER_HL),
            0x24 =>
                self.increment_register(Register::H),
            0x25 =>
                self.decrement_register(Register::H),
            0x26 =>
                self.load_immediate_value(Register::H),
            0x27 =>
                self.bcd_adjust(),
            0x28 =>
                self.conditional_relative_jump(self.is_z_flag_set()),
            0x29 => {
                let word = self.read_from_register_pair(&REGISTER_HL);
                self.add_value_to_register_pair(REGISTER_HL, word);
            },
            0x2A => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::A);
                self.check_oam_bug_mixed(address);
                self.store_in_register_pair(REGISTER_HL, address.wrapping_add(1));
            },
            0x2B =>
                self.decrement_register_pair(REGISTER_HL),
            0x2C =>
                self.increment_register(Register::L),
            0x2D =>
                self.decrement_register(Register::L),
            0x2E =>
                self.load_immediate_value(Register::L),
            0x2F => {
                self.registers.a = self.registers.a ^ 0xFF;
                self.set_flag_n(true);
                self.set_flag_h(true);
            },
            0x30 =>
                self.conditional_relative_jump(!self.is_c_flag_set()),
            0x31 => {
                let word = self.read_next_instruction_word();
                self.registers.stack_pointer = word;            
            },
            0x32 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::A, address);
                self.check_oam_bug_write(address);
                self.store_in_register_pair(REGISTER_HL, address.wrapping_sub(1));
            },
            0x33 => {
                let sp = self.registers.stack_pointer;
                self.registers.stack_pointer = sp.wrapping_add(1);
                self.step_one_machine_cycle();
                self.check_oam_bug_write(sp);
            },
            0x34 =>
                self.increment_memory_byte(),
            0x35 =>
                self.decrement_memory_byte(),
            0x36 =>
                self.load_immediate_value_in_memory(REGISTER_HL),
            0x37 => {
                self.set_flag_c(true);
                self.set_flag_h(false);
                self.set_flag_n(false);
            },
            0x38 =>
                self.conditional_relative_jump(self.is_c_flag_set()),
            0x39 => {
                let stack_pointer = self.registers.stack_pointer;
                self.add_value_to_register_pair(REGISTER_HL, stack_pointer)
            }
            0x3A => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::A);
                self.check_oam_bug_mixed(address);
                self.store_in_register_pair(REGISTER_HL, address.wrapping_sub(1));
            },
            0x3B => {
                let sp = self.registers.stack_pointer;
                self.registers.stack_pointer = sp.wrapping_sub(1);
                self.step_one_machine_cycle();
                self.check_oam_bug_write(sp);
            },
            0x3C =>
                self.increment_register(Register::A),
            0x3D =>
                self.decrement_register(Register::A),
            0x3E =>
                self.load_immediate_value(Register::A),
            0x3F => {
                let c_flag_set = self.is_c_flag_set();
                self.set_flag_c(!c_flag_set);
                self.set_flag_n(false);
                self.set_flag_h(false);
            },
            0x40 =>
                self.load_source_register_in_destination_register(Register::B, Register::B),
            0x41 =>
                self.load_source_register_in_destination_register(Register::C, Register::B),
            0x42 =>
                self.load_source_register_in_destination_register(Register::D, Register::B),
            0x43 =>
                self.load_source_register_in_destination_register(Register::E, Register::B),
            0x44 =>
                self.load_source_register_in_destination_register(Register::H, Register::B),
            0x45 =>
                self.load_source_register_in_destination_register(Register::L, Register::B),
            0x46 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::B)
            },
            0x47 =>
                self.load_source_register_in_destination_register(Register::A, Register::B),
            0x48 =>
                self.load_source_register_in_destination_register(Register::B, Register::C),
            0x49 =>
                self.load_source_register_in_destination_register(Register::C, Register::C),
            0x4A =>
                self.load_source_register_in_destination_register(Register::D, Register::C),
            0x4B =>
                self.load_source_register_in_destination_register(Register::E, Register::C),
            0x4C =>
                self.load_source_register_in_destination_register(Register::H, Register::C),
            0x4D =>
                self.load_source_register_in_destination_register(Register::L, Register::C),
            0x4E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::C)
            },
            0x4F =>
                self.load_source_register_in_destination_register(Register::A, Register::C),
            0x50 => 
                self.load_source_register_in_destination_register(Register::B, Register::D),
            0x51 =>
                self.load_source_register_in_destination_register(Register::C, Register::D),
            0x52 =>
                self.load_source_register_in_destination_register(Register::D, Register::D),
            0x53 =>
                self.load_source_register_in_destination_register(Register::E, Register::D),
            0x54 =>
                self.load_source_register_in_destination_register(Register::H, Register::D),
            0x55 =>
                self.load_source_register_in_destination_register(Register::L, Register::D),
            0x56 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::D)
            },
            0x57 =>
                self.load_source_register_in_destination_register(Register::A, Register::D),
            0x58 =>
                self.load_source_register_in_destination_register(Register::B, Register::E),
            0x59 =>
                self.load_source_register_in_destination_register(Register::C, Register::E),
            0x5A =>
                self.load_source_register_in_destination_register(Register::D, Register::E),
            0x5B =>
                self.load_source_register_in_destination_register(Register::E, Register::E),
            0x5C =>
                self.load_source_register_in_destination_register(Register::H, Register::E),
            0x5D =>
                self.load_source_register_in_destination_register(Register::L, Register::E),
            0x5E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::E)
            },
            0x5F =>
                self.load_source_register_in_destination_register(Register::A, Register::E),
            0x60 =>
                self.load_source_register_in_destination_register(Register::B, Register::H),
            0x61 =>
                self.load_source_register_in_destination_register(Register::C, Register::H),
            0x62 =>
                self.load_source_register_in_destination_register(Register::D, Register::H),
            0x63 =>
                self.load_source_register_in_destination_register(Register::E, Register::H),
            0x64 =>
                self.load_source_register_in_destination_register(Register::H, Register::H),
            0x65 =>
                self.load_source_register_in_destination_register(Register::L, Register::H),
            0x66 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::H)
            },
            0x67 =>
                self.load_source_register_in_destination_register(Register::A, Register::H),
            0x68 =>
                self.load_source_register_in_destination_register(Register::B, Register::L),
            0x69 =>
                self.load_source_register_in_destination_register(Register::C, Register::L),
            0x6A =>
                self.load_source_register_in_destination_register(Register::D, Register::L),
            0x6B =>
                self.load_source_register_in_destination_register(Register::E, Register::L),
            0x6C =>
                self.load_source_register_in_destination_register(Register::H, Register::L),
            0x6D =>
                self.load_source_register_in_destination_register(Register::L, Register::L),
            0x6E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::L)
            },
            0x6F =>
                self.load_source_register_in_destination_register(Register::A, Register::L),
            0x70 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::B, address);
            },
            0x71 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::C, address);
            },
            0x72 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::D, address);
            },
            0x73 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::E, address);
            },
            0x74 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::H, address);
            },
            0x75 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::L, address);
            },
            0x76 => {
                if self.interrupts_fired() {
                    self.halted = false;

                    if self.interrupts.enabled {
                        self.registers.program_counter -= 1;
                    }
                    else {
                        self.halt_bug = true;
                    }
                }
                else {
                    self.halted = true;
                }
            },
            0x77 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_source_register_in_memory(Register::A, address);
            },
            0x78 =>
                self.load_source_register_in_destination_register(Register::B, Register::A),
            0x79 =>
                self.load_source_register_in_destination_register(Register::C, Register::A),
            0x7A =>
                self.load_source_register_in_destination_register(Register::D, Register::A),
            0x7B =>
                self.load_source_register_in_destination_register(Register::E, Register::A),
            0x7C =>
                self.load_source_register_in_destination_register(Register::H, Register::A),
            0x7D =>
                self.load_source_register_in_destination_register(Register::L, Register::A),
            0x7E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.load_memory_byte_in_destination_register(address, Register::A)
            },
            0x7F =>
                self.load_source_register_in_destination_register(Register::A, Register::A),
            0x80 => {
                let value = self.read_from_register(&Register::B);
                self.add_value_to_register(Register::A, value);
            },
            0x81 => {
                let value = self.read_from_register(&Register::C);
                self.add_value_to_register(Register::A, value);
            },
            0x82 => {
                let value = self.read_from_register(&Register::D);
                self.add_value_to_register(Register::A, value);
            },
            0x83 => {
                let value = self.read_from_register(&Register::E);
                self.add_value_to_register(Register::A, value);
            },
            0x84 => {
                let value = self.read_from_register(&Register::H);
                self.add_value_to_register(Register::A, value);
            },
            0x85 => {
                let value = self.read_from_register(&Register::L);
                self.add_value_to_register(Register::A, value);
            },
            0x86 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.add_value_to_register(Register::A, value);
            },
            0x87 => {
                let value = self.read_from_register(&Register::A);
                self.add_value_to_register(Register::A, value);
            },
            0x88 => {
                let value = self.read_from_register(&Register::B);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x89 => {
                let value = self.read_from_register(&Register::C);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8A => {
                let value = self.read_from_register(&Register::D);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8B => {
                let value = self.read_from_register(&Register::E);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8C => {
                let value = self.read_from_register(&Register::H);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8D => {
                let value = self.read_from_register(&Register::L);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x8F => {
                let value = self.read_from_register(&Register::A);
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0x90 => {
                let value = self.read_from_register(&Register::B);
                self.subtract_value_from_register(Register::A, value);
            },
            0x91 => {
                let value = self.read_from_register(&Register::C);
                self.subtract_value_from_register(Register::A, value);
            },
            0x92 => {
                let value = self.read_from_register(&Register::D);
                self.subtract_value_from_register(Register::A, value);
            },
            0x93 => {
                let value = self.read_from_register(&Register::E);
                self.subtract_value_from_register(Register::A, value);
            },
            0x94 => {
                let value = self.read_from_register(&Register::H);
                self.subtract_value_from_register(Register::A, value);
            },
            0x95 => {
                let value = self.read_from_register(&Register::L);
                self.subtract_value_from_register(Register::A, value);
            },
            0x96 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.subtract_value_from_register(Register::A, value);
            },
            0x97 => {
                let value = self.read_from_register(&Register::A);
                self.subtract_value_from_register(Register::A, value);
            },
            0x98 => {
                let value = self.read_from_register(&Register::B);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x99 => {
                let value = self.read_from_register(&Register::C);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9A => {
                let value = self.read_from_register(&Register::D);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9B => {
                let value = self.read_from_register(&Register::E);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9C => {
                let value = self.read_from_register(&Register::H);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9D => {
                let value = self.read_from_register(&Register::L);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9E => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0x9F => {
                let value = self.read_from_register(&Register::A);
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0xA0 => {
                let value = self.read_from_register(&Register::B);
                self.logical_and_with_register(Register::A, value);
            },
            0xA1 => {
                let value = self.read_from_register(&Register::C);
                self.logical_and_with_register(Register::A, value);
            },
            0xA2 => {
                let value = self.read_from_register(&Register::D);
                self.logical_and_with_register(Register::A, value);
            },
            0xA3 => {
                let value = self.read_from_register(&Register::E);
                self.logical_and_with_register(Register::A, value);
            },
            0xA4 => {
                let value = self.read_from_register(&Register::H);
                self.logical_and_with_register(Register::A, value);
            },
            0xA5 => {
                let value = self.read_from_register(&Register::L);
                self.logical_and_with_register(Register::A, value);
            },
            0xA6 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.logical_and_with_register(Register::A, value);
            },
            0xA7 => {
                let value = self.read_from_register(&Register::A);
                self.logical_and_with_register(Register::A, value);
            },
            0xA8 => {
                let value = self.read_from_register(&Register::B);
                self.logical_xor_with_register(Register::A, value);
            },
            0xA9 => {
                let value = self.read_from_register(&Register::C);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAA => {
                let value = self.read_from_register(&Register::D);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAB => {
                let value = self.read_from_register(&Register::E);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAC => {
                let value = self.read_from_register(&Register::H);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAD => {
                let value = self.read_from_register(&Register::L);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAE => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.logical_xor_with_register(Register::A, value);
            },
            0xAF => {
                let value = self.read_from_register(&Register::A);
                self.logical_xor_with_register(Register::A, value);
            },
            0xB0 => {
                let value = self.read_from_register(&Register::B);
                self.logical_or_with_register(Register::A, value);
            },
            0xB1 => {
                let value = self.read_from_register(&Register::C);
                self.logical_or_with_register(Register::A, value);
            },
            0xB2 => {
                let value = self.read_from_register(&Register::D);
                self.logical_or_with_register(Register::A, value);
            },
            0xB3 => {
                let value = self.read_from_register(&Register::E);
                self.logical_or_with_register(Register::A, value);
            },
            0xB4 => {
                let value = self.read_from_register(&Register::H);
                self.logical_or_with_register(Register::A, value);
            },
            0xB5 => {
                let value = self.read_from_register(&Register::L);
                self.logical_or_with_register(Register::A, value);
            },
            0xB6 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.logical_or_with_register(Register::A, value);
            },
            0xB7 => {
                let value = self.read_from_register(&Register::A);
                self.logical_or_with_register(Register::A, value);
            },
            0xB8 => {
                let value = self.read_from_register(&Register::B);
                self.compare_value_with_register(Register::A, value);
            },
            0xB9 => {
                let value = self.read_from_register(&Register::C);
                self.compare_value_with_register(Register::A, value);
            },
            0xBA => {
                let value = self.read_from_register(&Register::D);
                self.compare_value_with_register(Register::A, value);
            },
            0xBB => {
                let value = self.read_from_register(&Register::E);
                self.compare_value_with_register(Register::A, value);
            },
            0xBC => {
                let value = self.read_from_register(&Register::H);
                self.compare_value_with_register(Register::A, value);
            },
            0xBD => {
                let value = self.read_from_register(&Register::L);
                self.compare_value_with_register(Register::A, value);
            },
            0xBE => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                let value = self.read_byte_from_memory(address);
                self.compare_value_with_register(Register::A, value);
            },
            0xBF => {
                let value = self.read_from_register(&Register::A);
                self.compare_value_with_register(Register::A, value);
            },
            0xC0 =>
                self.conditional_stack_return(!self.is_z_flag_set()),
            0xC1 =>
                self.pop_word_into_register_pair_from_stack(REGISTER_BC),
            0xC2 =>
                self.conditional_jump_using_immediate_word(!self.is_z_flag_set()),
            0xC3 => {
                self.registers.program_counter = self.read_next_instruction_word();
                self.step_one_machine_cycle();
            },
            0xC4 =>
                self.conditional_call_using_immediate_word(!self.is_z_flag_set()),
            0xC5 =>
                self.push_register_pair_to_stack(REGISTER_BC),
            0xC6 => {
                let value = self.read_next_instruction_byte();
                self.add_value_to_register(Register::A, value);
            },
            0xC7 =>
                self.restart(0x0),
            0xC8 =>
                self.conditional_stack_return(self.is_z_flag_set()),
            0xC9 =>
                self.stack_return(),
            0xCA =>
                self.conditional_jump_using_immediate_word(self.is_z_flag_set()),
            0xCB =>
                self.execute_cb_opcode(),
            0xCC =>
                self.conditional_call_using_immediate_word(self.is_z_flag_set()),
            0xCD =>
                self.call(),
            0xCE => {
                let value = self.read_next_instruction_byte();
                self.add_value_and_carry_to_register(Register::A, value);
            },
            0xCF =>
                self.restart(0x8),
            0xD0 =>
                self.conditional_stack_return(!self.is_c_flag_set()),
            0xD1 =>
                self.pop_word_into_register_pair_from_stack(REGISTER_DE),
            0xD2 =>
                self.conditional_jump_using_immediate_word(!self.is_c_flag_set()),
            0xD3 =>
                handle_illegal_opcode(opcode),
            0xD4 =>
                self.conditional_call_using_immediate_word(!self.is_c_flag_set()),
            0xD5 =>
                self.push_register_pair_to_stack(REGISTER_DE),
            0xD6 => {
                let value = self.read_next_instruction_byte();
                self.subtract_value_from_register(Register::A, value);
            },
            0xD7 =>
                self.restart(0x10),
            0xD8 =>
                self.conditional_stack_return(self.is_c_flag_set()),
            0xD9 => {
                self.stack_return();
                self.interrupts.enabled = true;
            },
            0xDA =>
                self.conditional_jump_using_immediate_word(self.is_c_flag_set()),
            0xDB =>
                handle_illegal_opcode(opcode),
            0xDC =>
                self.conditional_call_using_immediate_word(self.is_c_flag_set()),
            0xDD =>
                handle_illegal_opcode(opcode),
            0xDE => {
                let value = self.read_next_instruction_byte();
                self.subtract_value_and_carry_from_register(Register::A, value);
            },
            0xDF =>
                self.restart(0x18),
            0xE0 => {
                let address = 0xFF00 + self.read_next_instruction_byte() as u16;
                self.load_source_register_in_memory(Register::A, address);
            },
            0xE1 =>
                self.pop_word_into_register_pair_from_stack(REGISTER_HL),
            0xE2 => {
                let address = 0xFF00 + self.read_from_register(&Register::C) as u16;
                self.load_source_register_in_memory(Register::A, address);
            },
            0xE3 =>
                handle_illegal_opcode(opcode),
            0xE4 =>
                handle_illegal_opcode(opcode),
            0xE5 =>
                self.push_register_pair_to_stack(REGISTER_HL),
            0xE6 => {
                let value = self.read_next_instruction_byte();
                self.logical_and_with_register(Register::A, value);
            },
            0xE7 =>
                self.restart(0x20),
            0xE8 => {
                let signed_byte = self.read_next_instruction_byte() as i8;
                let sum = self.registers.stack_pointer.wrapping_add_signed(signed_byte.into());
                let stack_pointer = self.registers.stack_pointer;
                
                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h((sum & 0xF) < (stack_pointer & 0xF));
                self.set_flag_c((sum & 0xFF) < (stack_pointer & 0xFF));

                self.registers.stack_pointer = sum;

                self.step_one_machine_cycle();
                self.step_one_machine_cycle();
            },
            0xE9 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.registers.program_counter = address;
            },
            0xEA => {
                let address = self.read_next_instruction_word();
                self.load_source_register_in_memory(Register::A, address);
            },
            0xEB =>
                handle_illegal_opcode(opcode),
            0xEC =>
                handle_illegal_opcode(opcode),
            0xED =>
                handle_illegal_opcode(opcode),
            0xEE => {
                let value = self.read_next_instruction_byte();
                self.logical_xor_with_register(Register::A, value);
            },
            0xEF =>
                self.restart(0x28),
            0xF0 => {
                let address = 0xFF00 + self.read_next_instruction_byte() as u16;
                self.load_memory_byte_in_destination_register(address, Register::A);
            },
            0xF1 => {
                let word = self.pop_word_from_stack();
                self.store_in_register_pair(REGISTER_AF, word & 0xFFF0);
            }
            0xF2 => {
                let address = 0xFF00 + self.read_from_register(&Register::C) as u16;
                self.load_memory_byte_in_destination_register(address, Register::A);
            },
            0xF3 => {
                self.interrupts.enabled = false;
            },
            0xF4 =>
                handle_illegal_opcode(opcode),
            0xF5 =>
                self.push_register_pair_to_stack(REGISTER_AF),
            0xF6 => {
                let value = self.read_next_instruction_byte();
                self.logical_or_with_register(Register::A, value);
            },
            0xF7 =>
                self.restart(0x30),
            0xF8 => {
                let signed_byte = self.read_next_instruction_byte() as i8;
                let stack_pointer = self.registers.stack_pointer;
                let sum = stack_pointer.wrapping_add_signed(signed_byte.into());

                self.store_in_register_pair(REGISTER_HL, sum);
                
                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h((sum & 0xF) < (stack_pointer & 0xF));
                self.set_flag_c((sum & 0xFF) < (stack_pointer & 0xFF));

                self.step_one_machine_cycle();
            },
            0xF9 => {
                let word = self.read_from_register_pair(&REGISTER_HL);
                self.registers.stack_pointer = word;
                self.step_one_machine_cycle();
            },
            0xFA => {
                let address = self.read_next_instruction_word();
                self.load_memory_byte_in_destination_register(address, Register::A);
            },
            0xFB => {
                self.interrupts.enable_delay = 1;
            },
            0xFC =>
                handle_illegal_opcode(opcode),
            0xFD =>
                handle_illegal_opcode(opcode),
            0xFE => {
                let value = self.read_next_instruction_byte();
                self.compare_value_with_register(Register::A, value);
            },
            0xFF =>
                self.restart(0x38),
        }
    }

    fn execute_cb_opcode(&mut self) {
        let opcode = self.read_next_instruction_byte();
        match opcode {
            0x00 =>
                self.rotate_register_left(Register::B),
            0x01 =>
                self.rotate_register_left(Register::C),
            0x02 =>
                self.rotate_register_left(Register::D),
            0x03 =>
                self.rotate_register_left(Register::E),
            0x04 =>
                self.rotate_register_left(Register::H),
            0x05 =>
                self.rotate_register_left(Register::L),
            0x06 =>
                self.rotate_memory_byte_left(),
            0x07 =>
                self.rotate_register_left(Register::A),
            0x08 =>
                self.rotate_register_right(Register::B),
            0x09 =>
                self.rotate_register_right(Register::C),
            0x0A =>
                self.rotate_register_right(Register::D),
            0x0B =>
                self.rotate_register_right(Register::E),
            0x0C =>
                self.rotate_register_right(Register::H),
            0x0D =>
                self.rotate_register_right(Register::L),
            0x0E =>
                self.rotate_memory_byte_right(),
            0x0F =>
                self.rotate_register_right(Register::A),
            0x10 =>
                self.rotate_register_left_through_carry(Register::B),
            0x11 =>
                self.rotate_register_left_through_carry(Register::C),
            0x12 =>
                self.rotate_register_left_through_carry(Register::D),
            0x13 =>
                self.rotate_register_left_through_carry(Register::E),
            0x14 =>
                self.rotate_register_left_through_carry(Register::H),
            0x15 =>
                self.rotate_register_left_through_carry(Register::L),
            0x16 =>
                self.rotate_memory_byte_left_through_carry(),
            0x17 =>
                self.rotate_register_left_through_carry(Register::A),
            0x18 =>
                self.rotate_register_right_through_carry(Register::B),
            0x19 =>
                self.rotate_register_right_through_carry(Register::C),
            0x1A =>
                self.rotate_register_right_through_carry(Register::D),
            0x1B =>
                self.rotate_register_right_through_carry(Register::E),
            0x1C =>
                self.rotate_register_right_through_carry(Register::H),
            0x1D =>
                self.rotate_register_right_through_carry(Register::L),    
            0x1E =>
                self.rotate_memory_byte_right_through_carry(),
            0x1F =>
                self.rotate_register_right_through_carry(Register::A),
            0x20 =>
                self.shift_register_left(Register::B),
            0x21 =>
                self.shift_register_left(Register::C),
            0x22 =>
                self.shift_register_left(Register::D),
            0x23 =>
                self.shift_register_left(Register::E),
            0x24 =>
                self.shift_register_left(Register::H),
            0x25 =>
                self.shift_register_left(Register::L),
            0x26 =>
                self.shift_memory_byte_left(),
            0x27 =>
                self.shift_register_left(Register::A),
            0x28 =>
                self.shift_register_right_maintaining_msb(Register::B),
            0x29 =>
                self.shift_register_right_maintaining_msb(Register::C),
            0x2A =>
                self.shift_register_right_maintaining_msb(Register::D),
            0x2B =>
                self.shift_register_right_maintaining_msb(Register::E),
            0x2C =>
                self.shift_register_right_maintaining_msb(Register::H),
            0x2D =>
                self.shift_register_right_maintaining_msb(Register::L),
            0x2E =>
                self.shift_memory_byte_right_maintaining_msb(),
            0x2F =>
                self.shift_register_right_maintaining_msb(Register::A),
            0x30 =>
                self.swap_nibbles_in_register(Register::B),
            0x31 =>
                self.swap_nibbles_in_register(Register::C),
            0x32 =>
                self.swap_nibbles_in_register(Register::D),
            0x33 =>
                self.swap_nibbles_in_register(Register::E),
            0x34 =>
                self.swap_nibbles_in_register(Register::H),
            0x35 =>
                self.swap_nibbles_in_register(Register::L),
            0x36 => {
                let address = self.read_from_register_pair(&REGISTER_HL);
                self.swap_nibbles_in_memory_byte(address);
            },
            0x37 =>
                self.swap_nibbles_in_register(Register::A),
            0x38 =>
                self.shift_register_right(Register::B),
            0x39 =>
                self.shift_register_right(Register::C),
            0x3A =>
                self.shift_register_right(Register::D),
            0x3B =>
                self.shift_register_right(Register::E),
            0x3C =>
                self.shift_register_right(Register::H),
            0x3D =>
                self.shift_register_right(Register::L),
            0x3E =>
                self.shift_memory_byte_right(),
            0x3F =>
                self.shift_register_right(Register::A),
            0x40 =>
                self.test_register_bit(Register::B, 0),
            0x41 =>
                self.test_register_bit(Register::C, 0),
            0x42 =>
                self.test_register_bit(Register::D, 0),
            0x43 =>
                self.test_register_bit(Register::E, 0),   
            0x44 =>
                self.test_register_bit(Register::H, 0),
            0x45 =>
                self.test_register_bit(Register::L, 0),
            0x46 =>
                self.test_memory_bit(0),
            0x47 =>
                self.test_register_bit(Register::A, 0),
            0x48 =>
                self.test_register_bit(Register::B, 1),
            0x49 =>
                self.test_register_bit(Register::C, 1),
            0x4A =>
                self.test_register_bit(Register::D, 1),
            0x4B =>
                self.test_register_bit(Register::E, 1),
            0x4C =>
                self.test_register_bit(Register::H, 1),
            0x4D =>
                self.test_register_bit(Register::L, 1),
            0x4E =>
                self.test_memory_bit(1),
            0x4F =>
                self.test_register_bit(Register::A, 1),
            0x50 =>
                self.test_register_bit(Register::B, 2),
            0x51 =>
                self.test_register_bit(Register::C, 2),  
            0x52 =>
                self.test_register_bit(Register::D, 2),
            0x53 =>
                self.test_register_bit(Register::E, 2),
            0x54 =>
                self.test_register_bit(Register::H, 2),
            0x55 =>
                self.test_register_bit(Register::L, 2),
            0x56 =>
                self.test_memory_bit(2),
            0x57 =>
                self.test_register_bit(Register::A, 2),
            0x58 =>
                self.test_register_bit(Register::B, 3),
            0x59 =>
                self.test_register_bit(Register::C, 3),
            0x5A =>
                self.test_register_bit(Register::D, 3),
            0x5B =>
                self.test_register_bit(Register::E, 3),
            0x5C =>
                self.test_register_bit(Register::H, 3),
            0x5D =>
                self.test_register_bit(Register::L, 3),
            0x5E =>
                self.test_memory_bit(3),
            0x5F =>
                self.test_register_bit(Register::A, 3),
            0x60 =>
                self.test_register_bit(Register::B, 4),
            0x61 =>
                self.test_register_bit(Register::C, 4),
            0x62 =>
                self.test_register_bit(Register::D, 4),
            0x63 =>
                self.test_register_bit(Register::E, 4),
            0x64 =>
                self.test_register_bit(Register::H, 4),
            0x65 =>
                self.test_register_bit(Register::L, 4),
            0x66 =>
                self.test_memory_bit(4),
            0x67 =>
                self.test_register_bit(Register::A, 4),
            0x68 =>
                self.test_register_bit(Register::B, 5),
            0x69 =>
                self.test_register_bit(Register::C, 5),
            0x6A =>
                self.test_register_bit(Register::D, 5),
            0x6B =>
                self.test_register_bit(Register::E, 5),
            0x6C =>
                self.test_register_bit(Register::H, 5),
            0x6D =>
                self.test_register_bit(Register::L, 5),
            0x6E =>
                self.test_memory_bit(5),
            0x6F =>
                self.test_register_bit(Register::A, 5),
            0x70 =>
                self.test_register_bit(Register::B, 6),
            0x71 =>
                self.test_register_bit(Register::C, 6),
            0x72 =>
                self.test_register_bit(Register::D, 6),
            0x73 =>
                self.test_register_bit(Register::E, 6),
            0x74 =>
                self.test_register_bit(Register::H, 6),
            0x75 =>
                self.test_register_bit(Register::L, 6),
            0x76 =>
                self.test_memory_bit(6),
            0x77 =>
                self.test_register_bit(Register::A, 6),
            0x78 =>
                self.test_register_bit(Register::B, 7),
            0x79 =>
                self.test_register_bit(Register::C, 7),
            0x7A =>
                self.test_register_bit(Register::D, 7),
            0x7B =>
                self.test_register_bit(Register::E, 7),
            0x7C =>
                self.test_register_bit(Register::H, 7),
            0x7D =>
                self.test_register_bit(Register::L, 7),
            0x7E =>
                self.test_memory_bit(7),
            0x7F =>
                self.test_register_bit(Register::A, 7),
            0x80 =>
                self.reset_register_bit(Register::B, 0),
            0x81 =>
                self.reset_register_bit(Register::C, 0),
            0x82 =>
                self.reset_register_bit(Register::D, 0),
            0x83 =>
                self.reset_register_bit(Register::E, 0),
            0x84 =>
                self.reset_register_bit(Register::H, 0),
            0x85 =>
                self.reset_register_bit(Register::L, 0),
            0x86 =>
                self.reset_memory_bit(0),
            0x87 =>
                self.reset_register_bit(Register::A, 0),
            0x88 =>
                self.reset_register_bit(Register::B, 1),
            0x89 =>
                self.reset_register_bit(Register::C, 1),
            0x8A =>
                self.reset_register_bit(Register::D, 1),
            0x8B =>
                self.reset_register_bit(Register::E, 1),
            0x8C =>
                self.reset_register_bit(Register::H, 1),
            0x8D =>
                self.reset_register_bit(Register::L, 1),
            0x8E =>
                self.reset_memory_bit(1),
            0x8F =>
                self.reset_register_bit(Register::A, 1),
            0x90 =>
                self.reset_register_bit(Register::B, 2),
            0x91 =>
                self.reset_register_bit(Register::C, 2),
            0x92 =>
                self.reset_register_bit(Register::D, 2),
            0x93 =>
                self.reset_register_bit(Register::E, 2),
            0x94 =>
                self.reset_register_bit(Register::H, 2),
            0x95 =>
                self.reset_register_bit(Register::L, 2),
            0x96 =>
                self.reset_memory_bit(2),
            0x97 =>
                self.reset_register_bit(Register::A, 2),
            0x98 =>
                self.reset_register_bit(Register::B, 3),
            0x99 =>
                self.reset_register_bit(Register::C, 3),
            0x9A =>
                self.reset_register_bit(Register::D, 3),
            0x9B =>
                self.reset_register_bit(Register::E, 3),
            0x9C =>
                self.reset_register_bit(Register::H, 3),
            0x9D =>
                self.reset_register_bit(Register::L, 3),
            0x9E =>
                self.reset_memory_bit(3),
            0x9F =>
                self.reset_register_bit(Register::A, 3),
            0xA0 =>
                self.reset_register_bit(Register::B, 4),
            0xA1 =>
                self.reset_register_bit(Register::C, 4),
            0xA2 =>
                self.reset_register_bit(Register::D, 4),
            0xA3 =>
                self.reset_register_bit(Register::E, 4),
            0xA4 =>
                self.reset_register_bit(Register::H, 4),
            0xA5 =>
                self.reset_register_bit(Register::L, 4),
            0xA6 =>
                self.reset_memory_bit(4),
            0xA7 =>
                self.reset_register_bit(Register::A, 4),
            0xA8 =>
                self.reset_register_bit(Register::B, 5),
            0xA9 =>
                self.reset_register_bit(Register::C, 5),
            0xAA =>
                self.reset_register_bit(Register::D, 5),
            0xAB =>
                self.reset_register_bit(Register::E, 5),
            0xAC =>
                self.reset_register_bit(Register::H, 5),
            0xAD =>
                self.reset_register_bit(Register::L, 5),
            0xAE =>
                self.reset_memory_bit(5),
            0xAF =>
                self.reset_register_bit(Register::A, 5),
            0xB0 =>
                self.reset_register_bit(Register::B, 6),
            0xB1 =>
                self.reset_register_bit(Register::C, 6),
            0xB2 =>
                self.reset_register_bit(Register::D, 6),
            0xB3 =>
                self.reset_register_bit(Register::E, 6),
            0xB4 =>
                self.reset_register_bit(Register::H, 6),
            0xB5 =>
                self.reset_register_bit(Register::L, 6),
            0xB6 =>
                self.reset_memory_bit(6),
            0xB7 =>
                self.reset_register_bit(Register::A, 6),
            0xB8 =>
                self.reset_register_bit(Register::B, 7),
            0xB9 =>
                self.reset_register_bit(Register::C, 7),
            0xBA =>
                self.reset_register_bit(Register::D, 7),
            0xBB =>
                self.reset_register_bit(Register::E, 7),
            0xBC =>
                self.reset_register_bit(Register::H, 7),
            0xBD =>
                self.reset_register_bit(Register::L, 7),
            0xBE =>
                self.reset_memory_bit(7),
            0xBF =>
                self.reset_register_bit(Register::A, 7),
            0xC0 =>
                self.set_register_bit(Register::B, 0),
            0xC1 =>
                self.set_register_bit(Register::C, 0),
            0xC2 =>
                self.set_register_bit(Register::D, 0),
            0xC3 =>
                self.set_register_bit(Register::E, 0),
            0xC4 =>
                self.set_register_bit(Register::H, 0),
            0xC5 =>
                self.set_register_bit(Register::L, 0),
            0xC6 =>
                self.set_memory_bit(0),
            0xC7 =>
                self.set_register_bit(Register::A, 0),
            0xC8 =>
                self.set_register_bit(Register::B, 1),
            0xC9 =>
                self.set_register_bit(Register::C, 1),
            0xCA =>
                self.set_register_bit(Register::D, 1),
            0xCB =>
                self.set_register_bit(Register::E, 1),
            0xCC =>
                self.set_register_bit(Register::H, 1),
            0xCD =>
                self.set_register_bit(Register::L, 1),
            0xCE =>
                self.set_memory_bit(1),
            0xCF =>
                self.set_register_bit(Register::A, 1),
            0xD0 =>
                self.set_register_bit(Register::B, 2),
            0xD1 =>
                self.set_register_bit(Register::C, 2),
            0xD2 =>
                self.set_register_bit(Register::D, 2),
            0xD3 =>
                self.set_register_bit(Register::E, 2),
            0xD4 =>
                self.set_register_bit(Register::H, 2),
            0xD5 =>
                self.set_register_bit(Register::L, 2),
            0xD6 =>
                self.set_memory_bit(2),
            0xD7 =>
                self.set_register_bit(Register::A, 2),
            0xD8 =>
                self.set_register_bit(Register::B, 3),
            0xD9 =>
                self.set_register_bit(Register::C, 3),
            0xDA =>
                self.set_register_bit(Register::D, 3),
            0xDB =>
                self.set_register_bit(Register::E, 3),
            0xDC =>
                self.set_register_bit(Register::H, 3),
            0xDD =>
                self.set_register_bit(Register::L, 3),
            0xDE =>
                self.set_memory_bit(3),
            0xDF =>
                self.set_register_bit(Register::A, 3),
            0xE0 =>
                self.set_register_bit(Register::B, 4),
            0xE1 =>
                self.set_register_bit(Register::C, 4),
            0xE2 =>
                self.set_register_bit(Register::D, 4),
            0xE3 =>
                self.set_register_bit(Register::E, 4),
            0xE4 =>
                self.set_register_bit(Register::H, 4),
            0xE5 =>
                self.set_register_bit(Register::L, 4),
            0xE6 =>
                self.set_memory_bit(4),
            0xE7 =>
                self.set_register_bit(Register::A, 4),
            0xE8 =>
                self.set_register_bit(Register::B, 5),
            0xE9 =>
                self.set_register_bit(Register::C, 5),
            0xEA =>
                self.set_register_bit(Register::D, 5),
            0xEB =>
                self.set_register_bit(Register::E, 5),
            0xEC =>
                self.set_register_bit(Register::H, 5),
            0xED =>
                self.set_register_bit(Register::L, 5),
            0xEE =>
                self.set_memory_bit(5),
            0xEF =>
                self.set_register_bit(Register::A, 5),
            0xF0 =>
                self.set_register_bit(Register::B, 6),
            0xF1 =>
                self.set_register_bit(Register::C, 6),
            0xF2 =>
                self.set_register_bit(Register::D, 6),
            0xF3 =>
                self.set_register_bit(Register::E, 6),
            0xF4 =>
                self.set_register_bit(Register::H, 6),
            0xF5 =>
                self.set_register_bit(Register::L, 6),
            0xF6 =>
                self.set_memory_bit(6),
            0xF7 =>
                self.set_register_bit(Register::A, 6),
            0xF8 =>
                self.set_register_bit(Register::B, 7),
            0xF9 =>
                self.set_register_bit(Register::C, 7),
            0xFA =>
                self.set_register_bit(Register::D, 7),
            0xFB =>
                self.set_register_bit(Register::E, 7),
            0xFC =>
                self.set_register_bit(Register::H, 7),
            0xFD =>
                self.set_register_bit(Register::L, 7),
            0xFE =>
                self.set_memory_bit(7),
            0xFF =>
                self.set_register_bit(Register::A, 7),
        }
    }
}

#[cfg(test)]
mod tests;
