use crate::apu::{Apu, ApuParams};
use crate::cheats::{initialize_cheats, CheatState};
use crate::cpu::{self, initialize_cpu, timers, CpuState};
use crate::cpu::interrupts::{initialize_interrupt_registers, InterruptRegisters};
use crate::cpu::timers::{initialize_timer_registers, TimerRegisters};
use crate::cpu::hdma::{HDMAState, initialize_hdma};
use crate::dma;
use crate::dma::{initialize_dma, DMAState};
use crate::gpu::{Gpu, GpuParams};
use crate::joypad::Joypad;
use crate::mmu::{self, Memory, initialize_memory};
use crate::serial::{Serial, SerialParams};
use crate::speed_switch::{initialize_speed_switch, SpeedSwitch};
use std::cell::{Ref, RefMut};
use std::io::Result;

pub use crate::mmu::effects::CartridgeEffects;
pub use crate::mmu::{CartridgeHeader, RTCState};

#[derive(PartialEq, Eq)]
pub enum Mode {
    DMG,
    CGB
}

pub struct Emulator {
    pub cpu: CpuState,
    pub interrupts: InterruptRegisters,
    pub timers: TimerRegisters,
    pub memory: Memory,
    pub gpu: Gpu,
    pub joypad: Joypad,
    pub apu: Apu,
    pub dma: DMAState,
    pub hdma: HDMAState,
    pub serial: Serial,
    pub cheats: CheatState,
    pub renderer: fn(&[u8]),
    pub mode: Mode,
    pub speed_switch: SpeedSwitch,
    pub processor_test_mode: bool
}

pub fn initialize_emulator(renderer: fn(&[u8])) -> Emulator {
    Emulator {
        cpu: initialize_cpu(),
        interrupts: initialize_interrupt_registers(),
        timers: initialize_timer_registers(),
        memory: initialize_memory(),
        gpu: Gpu::new(),
        joypad: Joypad::new(),
        apu: Apu::new(),
        dma: initialize_dma(),
        hdma: initialize_hdma(),
        serial: Serial::new(),
        cheats: initialize_cheats(),
        renderer,
        mode: Mode::DMG,
        speed_switch: initialize_speed_switch(),
        processor_test_mode: false
    }
}

pub fn initialize_screenless_emulator() -> Emulator {
    initialize_emulator(|_| {})
}

pub fn is_cgb(emulator: &Emulator) -> bool {
    emulator.mode == Mode::CGB
}

pub fn in_color_bios(emulator: &Emulator) -> bool {
    emulator.memory.in_bios && is_cgb(emulator)
}

pub fn load_rom(emulator: &mut RefMut<Emulator>, rom: &[u8], cartridge_effects: Box<dyn CartridgeEffects>) -> Result<CartridgeHeader> {
    let buffer = rom.to_vec();
    mmu::load_rom_buffer(&mut emulator.memory, buffer, cartridge_effects)
}

pub fn set_cartridge_ram(emulator: &mut RefMut<Emulator>, ram: &[u8]) {
    mmu::set_cartridge_ram(&mut emulator.memory, ram.to_vec());
}

pub fn get_cartridge_ram(emulator: &Ref<Emulator>) -> Vec<u8> {
    mmu::get_cartridge_ram(&emulator.memory)
}

pub fn sync(emulator: &mut Emulator) {
    let in_color_bios = in_color_bios(emulator);

    timers::step(emulator);
    dma::step(emulator);
    emulator.gpu.step(GpuParams {
        interrupt_registers: &mut emulator.interrupts,
        hdma: &mut emulator.hdma,
        in_color_bios,
        renderer: emulator.renderer
    });
    emulator.apu.step(ApuParams {
        in_color_bios,
        divider: emulator.timers.divider,
    });
    emulator.serial.step(SerialParams {
        interrupt_registers: &mut emulator.interrupts,
    });
}

pub fn set_mode(emulator: &mut Emulator, mode: Mode) {
    emulator.mode = mode;

    let is_cgb = is_cgb(emulator);
    emulator.apu.set_cgb_mode(is_cgb);
    emulator.gpu.set_cgb_mode(is_cgb);
    emulator.serial.set_cgb_mode(is_cgb);
    
    mmu::load_bios(emulator);
}

pub fn set_sample_rate(emulator: &mut Emulator, sample_rate: u32) {
    emulator.apu.set_sample_rate(sample_rate);
}

pub fn step(emulator: &mut Emulator) {
    cpu::opcodes::step(emulator);
}

pub fn step_until_next_audio_buffer(emulator: &mut Emulator) -> (&[f32], &[f32]) {
    emulator.apu.clear_audio_buffers();

    while !emulator.apu.audio_buffers_full() {
        step(emulator);
    }

    let left_samples_slice = emulator.apu.get_left_sample_queue();
    let right_samples_slice = emulator.apu.get_right_sample_queue();

    (left_samples_slice, right_samples_slice)
}
