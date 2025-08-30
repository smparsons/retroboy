use crate::address_bus::hdma::HDMAState;
use crate::gpu::{Gpu, GpuParams};

fn initialize_test_gpu() -> Gpu {
    let mut gpu = Gpu::new(|_| {});
    gpu.lcdc = 0b10000000;
    gpu
}

fn step_gpu(gpu: &mut Gpu) {
    gpu.step(GpuParams {
        hdma: &mut HDMAState::new(),
    });
}

#[test]
fn should_move_from_oam_to_vram_mode() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 2;
    gpu.ly = 0;
    gpu.mode_clock = 76;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 3);
    assert_eq!(gpu.mode_clock, 0);
}

#[test]
fn should_move_from_vram_to_hblank_mode() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 3;
    gpu.ly = 0;
    gpu.mode_clock = 168;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 0);
    assert_eq!(gpu.mode_clock, 0);
}

#[test]
fn should_not_move_from_oam_to_vram_mode_too_early() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 2;
    gpu.ly = 0;
    gpu.mode_clock = 40;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 2);
    assert_eq!(gpu.mode_clock, 44);
}

#[test]
fn should_move_back_to_oam_mode_from_hblank_if_not_at_last_line() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 100;
    gpu.mode_clock = 200;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 2);
    assert_eq!(gpu.mode_clock, 0);
    assert_eq!(gpu.ly, 101);
}

#[test]
fn should_move_to_vblank_mode_from_hblank_if_at_last_line() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 143;
    gpu.mode_clock = 200;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 1);
    assert_eq!(gpu.mode_clock, 0);
    assert_eq!(gpu.ly, 144);
}

#[test]
fn should_fire_vblank_interrupt_when_entering_vblank_mode() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 143;
    gpu.mode_clock = 200;
    step_gpu(&mut gpu);
    assert_eq!(gpu.vblank_interrupt, true);
}

#[test]
fn should_move_back_to_oam_mode_from_vblank_at_correct_time() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 1;
    gpu.ly = 153;
    gpu.mode_clock = 452;
    step_gpu(&mut gpu);
    assert_eq!(gpu.mode, 2);
    assert_eq!(gpu.mode_clock, 0);
    assert_eq!(gpu.ly, 0);
}

#[test]
fn should_update_stat_register_with_mode_2_status() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 1;
    gpu.ly = 153;
    gpu.mode_clock = 452;
    gpu.stat = 0b00000001;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b00000110);
}

#[test]
fn should_fire_stat_interrupt_on_switch_to_mode_2_when_enabled() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 1;
    gpu.ly = 153;
    gpu.mode_clock = 452;
    gpu.stat = 0b00100001;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat_interrupt, true);
}

#[test]
fn should_update_stat_register_with_mode_3_status() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 2;
    gpu.ly = 0;
    gpu.mode_clock = 76;
    gpu.stat = 0b00000010;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b00000011);
}

#[test]
fn should_update_stat_register_with_mode_0_status() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 3;
    gpu.ly = 0;
    gpu.mode_clock = 168;
    gpu.stat = 0b00000011;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b00000000);
}

#[test]
fn should_fire_stat_interrupt_on_switch_to_mode_0_if_enabled() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 3;
    gpu.ly = 0;
    gpu.mode_clock = 168;
    gpu.stat = 0b00001011;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat_interrupt, true);
}

#[test]
fn should_update_stat_register_with_mode_1_status() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 143;
    gpu.mode_clock = 200;
    gpu.stat = 0b00000000;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b00000001);
}

#[test]
fn should_fire_stat_interrupt_on_switch_to_mode_1_if_enabled() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 143;
    gpu.mode_clock = 200;
    gpu.stat = 0b00010000;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat_interrupt, true);
}

#[test]
fn should_fire_stat_interrupt_when_lyc_equals_ly_if_enabled() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 13;
    gpu.lyc = 14;
    gpu.mode_clock = 200;
    gpu.stat = 0b01000000;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat_interrupt, true);
}

#[test]
fn should_update_stat_register_when_lyc_equals_ly() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 13;
    gpu.lyc = 14;
    gpu.mode_clock = 200;
    gpu.stat = 0b01000000;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b01000110);
}

#[test]
fn should_update_stat_register_when_lyc_is_not_equal_to_ly() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 14;
    gpu.lyc = 14;
    gpu.mode_clock = 200;
    gpu.stat = 0b01000100;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat, 0b01000010);
}

#[test]
fn should_not_fire_stat_interrupt_when_lyc_equals_ly_if_disabled() {
    let mut gpu = initialize_test_gpu();
    gpu.mode = 0;
    gpu.ly = 13;
    gpu.lyc = 14;
    gpu.mode_clock = 200;
    gpu.stat = 0b00000000;
    step_gpu(&mut gpu);
    assert_eq!(gpu.stat_interrupt, false);
}

#[test]
fn should_set_cgb_vbk() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.set_cgb_vbk(1);
    assert_eq!(gpu.cgb_vbk, 1);
}

#[test]
fn should_get_cgb_vbk() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.cgb_vbk = 1;
    assert_eq!(gpu.cgb_vbk(), 0xFF);
}

#[test]
fn should_ignore_all_bits_other_than_bit_0_when_getting_cgb_vbk() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.cgb_vbk = 0b00101010;
    assert_eq!(gpu.cgb_vbk(), 0b11111110);
}

#[test]
fn should_not_set_cgb_vbk_if_dmg_mode() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = false;
    gpu.set_cgb_vbk(1);
    assert_eq!(gpu.cgb_vbk, 0);
}

#[test]
fn should_return_ff_when_getting_cgb_vbk_if_dmg_mode() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = false;
    gpu.set_cgb_vbk(0);
    assert_eq!(gpu.cgb_vbk(), 0xFF);
}

#[test]
fn should_read_from_bank_1_of_video_ram() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.video_ram[0x3800] = 0xA1;
    gpu.set_cgb_vbk(1);
    assert_eq!(gpu.get_video_ram_byte(0x1800), 0xA1);
}

#[test]
fn should_set_byte_in_bank_1_of_video_ram() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.set_cgb_vbk(1);
    gpu.set_video_ram_byte(0x1802, 0xA1);
    assert_eq!(gpu.video_ram[0x3802], 0xA1);
}

#[test]
fn should_read_from_bank_0_of_video_ram() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.video_ram[0x1800] = 0xA1;
    gpu.set_cgb_vbk(0);
    assert_eq!(gpu.get_video_ram_byte(0x1800), 0xA1);
}

#[test]
fn should_set_byte_in_bank_0_of_video_ram() {
    let mut gpu = initialize_test_gpu();
    gpu.cgb_mode = true;
    gpu.set_cgb_vbk(0);
    gpu.set_video_ram_byte(0x1802, 0xA1);
    assert_eq!(gpu.video_ram[0x1802], 0xA1);
}