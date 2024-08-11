use crate::emulator::Emulator;
use crate::mmu;

#[derive(Debug)]
pub struct DmaState {
    pub source: u16,
    pub offset: u8,
    pub delay: u8,
    pub in_progress: bool
}

pub const DMA_TRANSFER_BYTES: u8 = 160;

pub fn initialize_dma() -> DmaState {
    DmaState {
        source: 0x0,
        offset: 0x0,
        delay: 0,
        in_progress: false
    }
}

pub fn start_dma(emulator: &mut Emulator, source: u8) {
    emulator.dma.source = (source as u16) << 8;

    if !emulator.dma.in_progress {
        emulator.dma.offset = 0x0;
        emulator.dma.delay = 2;
        emulator.dma.in_progress = true;
    }
}

pub fn get_source(emulator: &Emulator) -> u8 {
    (emulator.dma.source >> 8) as u8
}

fn transfer_byte(emulator: &mut Emulator) {
    let address = emulator.dma.source + (emulator.dma.offset as u16);
    emulator.memory.object_attribute_memory[emulator.dma.offset as usize] = mmu::read_byte(emulator, address);
}

pub fn step(emulator: &mut Emulator) {
    if emulator.dma.in_progress {
        if emulator.dma.delay > 0 {
            emulator.dma.delay -= 1;
        }
        else {
            transfer_byte(emulator);
    
            emulator.dma.offset += 1;
    
            if emulator.dma.offset == DMA_TRANSFER_BYTES {
                emulator.dma.in_progress = false;
            } 
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::initialize_emulator;
    use crate::mmu;
    use super::*;

    #[test]
    fn should_start_dma_transfer() {
        let mut emulator = initialize_emulator();
        start_dma(&mut emulator, 0x12);
        assert_eq!(emulator.dma.source, 0x1200);
        assert_eq!(emulator.dma.offset, 0x0);
        assert_eq!(emulator.dma.in_progress, true);
    }

    #[test]
    fn should_allow_modifications_to_dma_register_if_transfer_is_already_in_progress() {
        let mut emulator = initialize_emulator();
        emulator.dma.in_progress = true;
        start_dma(&mut emulator, 0x12);
        assert_eq!(emulator.dma.source, 0x1200);
        assert_eq!(emulator.dma.offset, 0x0);
        assert_eq!(emulator.dma.in_progress, true);
    }

    #[test]
    fn should_transfer_byte_from_source_to_destination() {
        let mut emulator = initialize_emulator();

        let mut test_instructions: Vec<u8> = vec![0; 0x8000];
        test_instructions.resize(0x8000, 0);
        mmu::load_rom_buffer(&mut emulator.memory, test_instructions);

        emulator.dma.source = 0x1200;
        emulator.dma.offset = 0x0;
        emulator.dma.in_progress = true;
        emulator.memory.rom[0x1200] = 0x12;
        
        step(&mut emulator);
        
        assert_eq!(emulator.memory.object_attribute_memory[0], 0x12);
        assert_eq!(emulator.dma.source, 0x1200);
        assert_eq!(emulator.dma.offset, 1);
        assert_eq!(emulator.dma.in_progress, true);
    }

    #[test]
    fn should_stop_dma_transfer_after_transferring_160_bytes() {
        let mut emulator = initialize_emulator();

        let mut test_instructions: Vec<u8> = vec![0; 0x8000];
        test_instructions.resize(0x8000, 0);
        mmu::load_rom_buffer(&mut emulator.memory, test_instructions);

        emulator.dma.source = 0x1200;
        emulator.dma.offset = 0x0;
        emulator.dma.in_progress = true;
        emulator.memory.rom[0x129F] = 0x12;
        
        for _ in 0..DMA_TRANSFER_BYTES {
            step(&mut emulator);
        }
        
        assert_eq!(emulator.memory.object_attribute_memory[0x9F], 0x12);
        assert_eq!(emulator.dma.offset, DMA_TRANSFER_BYTES);
        assert_eq!(emulator.dma.in_progress, false);
    }

    #[test]
    fn should_do_nothing_if_no_dma_transfer_is_in_progress() {
        let mut emulator = initialize_emulator();
        step(&mut emulator);
        assert_eq!(emulator.dma.source, 0x0);
        assert_eq!(emulator.dma.offset, 0x0);
        assert_eq!(emulator.dma.in_progress, false);
    }
}
