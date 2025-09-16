use crate::cpu::Cpu;
use crate::address_bus::AddressBus;
use crate::serializable::Serializable;
use getset::{Getters, MutGetters};
use serializable_derive::Serializable;
use std::io::Result;

pub use crate::address_bus::effects::CartridgeEffects;
pub use crate::address_bus::{CartridgeHeader, RTCState};
pub use crate::cpu::{Registers, BusActivityEntry, BusActivityType};
pub use crate::joypad::Key;

#[derive(Serializable, Getters, MutGetters)]
#[getset(get = "pub(crate)", get_mut = "pub(crate)")]
pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new(renderer: fn(&[u8]), processor_test_mode: bool) -> Emulator {
        let address_bus = AddressBus::new(renderer, processor_test_mode);
        Emulator {
            cpu: Cpu::new(address_bus)
        }
    }

    fn address_bus(&self) -> &AddressBus {
        self.cpu.address_bus()
    }

    fn address_bus_mut(&mut self) -> &mut AddressBus {
        self.cpu.address_bus_mut()
    }

    pub fn load_rom(&mut self, rom: &[u8], cartridge_effects: Box<dyn CartridgeEffects>) -> Result<CartridgeHeader> {
        let buffer = rom.to_vec();
        self.address_bus_mut().load_rom_buffer(buffer, cartridge_effects) 
    }

    pub fn set_cartridge_ram(&mut self, ram: &[u8]) {
        self.cpu.address_bus_mut().set_cartridge_ram(ram.to_vec());
    }

    pub fn cartridge_ram(&self) -> Vec<u8> {
        self.address_bus().get_cartridge_ram()
    }

    pub fn set_cgb_mode(&mut self, is_cgb: bool) {
        self.address_bus_mut().set_cgb_mode(is_cgb);
        self.address_bus_mut().load_bios(is_cgb);
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.address_bus_mut().apu_mut().set_sample_rate(sample_rate);
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }

    pub fn step_until_next_audio_buffer(&mut self) -> (&[f32], &[f32]) {
        self.address_bus_mut().apu_mut().clear_audio_buffers();

        while !self.address_bus().apu().audio_buffers_full() {
            self.step();
        }

        let apu = self.address_bus().apu();
        let left_samples_slice = apu.get_left_sample_queue();
        let right_samples_slice = apu.get_right_sample_queue();

        (left_samples_slice, right_samples_slice)        
    }

    pub fn handle_key_press(&mut self, key: &Key) {
        self.address_bus_mut().handle_key_press(key);
    }

    pub fn handle_key_release(&mut self, key: &Key) {
        self.address_bus_mut().handle_key_release(key);
    }

    pub fn register_gameshark_cheat(&mut self, cheat_id: &str, cheat: &str) -> Option<String> {
        self.address_bus_mut().cheats_mut().register_gameshark_cheat(cheat_id, cheat)
    }

    pub fn register_gamegenie_cheat(&mut self, cheat_id: &str, cheat: &str) -> Option<String> {
        self.address_bus_mut().cheats_mut().register_gamegenie_cheat(cheat_id, cheat)
    }

    pub fn unregister_cheat(&mut self, cheat_id: &str) {
        self.address_bus_mut().cheats_mut().unregister(cheat_id);
    }

    pub fn set_register_state(&mut self, registers: &Registers) {
        self.cpu_mut().registers_mut()
            .set_a(registers.a())
            .set_b(registers.b())
            .set_c(registers.c())
            .set_d(registers.d())
            .set_e(registers.e())
            .set_f(registers.f())
            .set_h(registers.h())
            .set_l(registers.l())
            .set_opcode(registers.opcode())
            .set_program_counter(registers.program_counter())
            .set_stack_pointer(registers.stack_pointer());
    }

    pub fn register_state(&self) -> &Registers {
        self.cpu().registers()
    }

    pub fn set_processor_test_ram(&mut self, index: u16, value: u8) {
        let test_ram = self.address_bus_mut().processor_test_ram_mut();
        test_ram[index as usize] = value;
    }

    pub fn processor_test_ram_byte(&self, index: u16) -> u8 {
        self.address_bus().processor_test_ram()[index as usize]
    }

    pub fn opcode_bus_activity(&self) -> &Vec<Option<BusActivityEntry>> {
        self.cpu().opcode_bus_activity()
    }
}

mod save_state;
