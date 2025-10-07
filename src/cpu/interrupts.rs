use crate::cpu::Cpu;
use crate::serializable::Serializable;
use getset::{CopyGetters, Setters};
use serializable_derive::Serializable;

pub enum InterruptType {
    VBlank,
    LCDStatus,
    TimerOverflow,
    SerialLink,
    JoypadPress
}

#[derive(Serializable, Debug, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct InterruptRegisters {
    enabled: u8,
}

pub fn initialize_interrupt_registers() -> InterruptRegisters {
    InterruptRegisters {
        enabled: 0,
    }
}

fn get_interrupt_isr(interrupt_type: &InterruptType) -> u8 {
    match interrupt_type {
        InterruptType::VBlank => 0x40,
        InterruptType::LCDStatus => 0x48,
        InterruptType::TimerOverflow => 0x50,
        InterruptType::SerialLink => 0x58,
        InterruptType::JoypadPress => 0x60
    }
}

impl Cpu {
    fn get_fired_interrupt_bits(&self) -> u8 {
        let interrupts = self.address_bus.interrupts();
        interrupts.enabled & self.address_bus.interrupt_flags() & 0x1F
    }

    fn get_fired_interrupt(&self) -> Option<InterruptType> {
        let fired_interrupt_bits = self.get_fired_interrupt_bits();
        if (fired_interrupt_bits & 0x01) != 0 {
            Some(InterruptType::VBlank)
        }
        else if (fired_interrupt_bits & 0x02) != 0 {
            Some(InterruptType::LCDStatus)
        }
        else if (fired_interrupt_bits & 0x04) != 0 {
            Some(InterruptType::TimerOverflow)
        }
        else if (fired_interrupt_bits & 0x08) != 0 {
            Some(InterruptType::SerialLink)
        }
        else if (fired_interrupt_bits & 0x10) != 0 {
            Some(InterruptType::JoypadPress)
        }
        else {
            None
        }
    }

    fn turn_off_interrupt_flag(&mut self, interrupt_type: &InterruptType) {
        match interrupt_type {
            InterruptType::VBlank =>
                { self.address_bus.gpu_mut().set_vblank_interrupt(false); },
            InterruptType::LCDStatus =>
                { self.address_bus.gpu_mut().set_stat_interrupt(false); },
            InterruptType::TimerOverflow =>
                { self.address_bus.timers_mut().set_interrupt(false); },
            InterruptType::SerialLink =>
                { self.address_bus.serial_mut().set_interrupt(false); },
            InterruptType::JoypadPress =>
                { self.address_bus.joypad_mut().set_interrupt(false); }
        }
    }

    pub(super) fn interrupts_fired(&self) -> bool {
        let fired_interrupt_bits = self.get_fired_interrupt_bits();
        fired_interrupt_bits != 0
    }

    pub(super) fn interrupt_step(&mut self) -> bool {
        let interrupts_fired = self.interrupts_fired();
        let interrupts_enabled = self.interrupts.enabled; 
        if !interrupts_enabled && interrupts_fired && self.halted {
            self.halted = false;
            false 
        }
        else if interrupts_enabled && interrupts_fired {
            self.halted = false;
            let maybe_fired_interrupt = self.get_fired_interrupt();
            match maybe_fired_interrupt {
                Some(interrupt_type) => {
                    self.interrupts.enabled = false;
                    self.turn_off_interrupt_flag(&interrupt_type);
                    let isr_address = get_interrupt_isr(&interrupt_type);
                    self.step_machine_cycles(2);
                    self.restart(isr_address as u16);
                    true
                },
                None => false
            }
        }
        else {
            false
        }
    }
}
