use crate::address_bus::effects::empty_cartridge_effects;
use crate::address_bus::test_utils::*;
use crate::address_bus::constants::*;

use super::*;

fn setup_test_address_bus() -> AddressBus {
    let mut address_bus = initialize_test_address_bus();

    address_bus.bios[0] = 0xAF;
    address_bus.bios[1] = 0xF1;
    address_bus.bios[2] = 0x03;
    address_bus.bios[3] = 0x4D;

    let mut rom = build_rom(CART_TYPE_MBC1, ROM_SIZE_64KB, RAM_SIZE_2KB);
    
    rom[0] = 0x1E;
    rom[1] = 0xF2;
    rom[2] = 0x01;
    rom[3] = 0x09;

    rom[0x20AF] = 0x11;
    rom[0x20B0] = 0x17;
    rom[0x20B1] = 0xEE;

    rom[0x5ACC] = 0x13;
    rom[0x5ACD] = 0x9C;
    rom[0x5ACE] = 0x55;

    address_bus.load_rom_buffer(rom, empty_cartridge_effects()).unwrap(); 

    let mut ram = vec![0; 0x800];
    ram[0] = 0xC2;
    ram[1] = 0x22;
    ram[2] = 0x35;
    address_bus.set_cartridge_ram(ram);

    address_bus.gpu.set_video_ram_byte(0, 0xB1);
    address_bus.gpu.set_video_ram_byte(1, 0xD2);
    address_bus.gpu.set_video_ram_byte(2, 0xAA);

    address_bus.working_ram[0] = 0xF1;
    address_bus.working_ram[1] = 0x22;
    address_bus.working_ram[2] = 0x2B;
    
    address_bus.working_ram[0x1001] = 0x11;

    address_bus.working_ram[0x15F0] = 0x2B;
    address_bus.working_ram[0x15F1] = 0x7C;

    address_bus.working_ram[0x2001] = 0x22;

    address_bus.gpu.set_object_attribute_memory_byte(0x7A, 0x44);
    address_bus.gpu.set_object_attribute_memory_byte(0x7B, 0x45);
    address_bus.gpu.set_object_attribute_memory_byte(0x7C, 0x9B);

    address_bus.zero_page_ram[0x20] = 0xBB;
    address_bus.zero_page_ram[0x21] = 0x44;
    address_bus.zero_page_ram[0x5B] = 0x5F;

    address_bus.interrupts.set_enabled(0x1F);

    address_bus.timers.set_divider(0x3A);
    address_bus.timers.set_counter(0x04);
    address_bus.timers.set_modulo(0x02);
    address_bus.timers.set_control(0x07);

    address_bus.gpu.set_lcdc(0x80);
    address_bus.gpu.set_scy(0x55);
    address_bus.gpu.set_scx(0xA1);
    address_bus.gpu.set_wy(0xBB);
    address_bus.gpu.set_wx(0xDD);
    address_bus.gpu.palettes_mut().set_bgp(0xC1);
    address_bus.gpu.set_ly(0x2B);
    address_bus.gpu.set_lyc(0xAB);
    address_bus.gpu.set_stat(0xD2);
    address_bus.gpu.palettes_mut().set_obp0(0x1B);
    address_bus.gpu.palettes_mut().set_obp1(0xE4);
    address_bus.gpu.set_stat_interrupt(true);

    address_bus.joypad.set_column(0x10);
    address_bus.joypad.set_select_buttons(0x4);

    address_bus.apu.set_enabled(true);
    address_bus.apu.set_sound_panning(0xF2);
    address_bus.apu.set_master_volume(0xC1);
    address_bus.apu.set_audio_buffer_clock(84);

    address_bus.apu.channel1_mut().sweep_mut().set_initial_settings(0xDD);
    address_bus.apu.channel1_mut().length_mut().set_initial_settings(0xB0);
    address_bus.apu.channel1_mut().envelope_mut().set_initial_settings(0xAA);
    address_bus.apu.channel1_mut().period_mut().set_low(0xB2);
    address_bus.apu.channel1_mut().period_mut().set_high(0xC2);

    address_bus.apu.channel2_mut().length_mut().set_initial_settings(0xC0);
    address_bus.apu.channel2_mut().envelope_mut().set_initial_settings(0xC1);
    address_bus.apu.channel2_mut().period_mut().set_low(0x14);
    address_bus.apu.channel2_mut().period_mut().set_high(0x24);

    address_bus.apu.channel3_mut().set_enabled(true);
    address_bus.apu.channel3_mut().set_dac_enabled(true);
    address_bus.apu.channel3_mut().set_volume(0x60);
    address_bus.apu.channel3_mut().period_mut().set_low(0xFF);
    address_bus.apu.channel3_mut().period_mut().set_high(0x44);
    address_bus.apu.channel3_mut().period_mut().set_divider(0x301);
    address_bus.apu.channel3_mut().period_mut().set_reloaded(true);
    address_bus.apu.channel3_mut().write_to_wave_ram(0x0, 0xB1);
    address_bus.apu.channel3_mut().write_to_wave_ram(0x1, 0xD2);

    address_bus.apu.channel4_mut().length_mut().set_initial_settings(0x1A);
    address_bus.apu.channel4_mut().envelope_mut().set_initial_settings(0xD2);
    address_bus.apu.channel4_mut().set_polynomial(0xCE);
    address_bus.apu.channel4_mut().set_control(0xC0);

    address_bus.serial.set_interrupt(true);

    address_bus.in_bios = false;

    address_bus
}

#[test]
fn reads_from_bios() {
    let mut address_bus = setup_test_address_bus();
    address_bus.in_bios = true;
    assert_eq!(address_bus.read_byte(0x02), 0x03);
}

#[test]
fn reads_from_rom_in_bank_zero() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0x02), 0x01);
}

#[test]
fn reads_from_rom_in_bank_zero_scenario_two() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0x20B1), 0xEE);
}

#[test]
fn reads_from_rom_in_subsequent_bank() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0x5ACE), 0x55);
}

#[test]
fn reads_from_video_ram() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0x8002), 0xAA);
}

#[test]
fn disallow_access_to_external_ram_if_not_enabled() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xA001), 0xFF);
}

#[test]
fn reads_from_working_ram() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xC002), 0x2B);
}

#[test]
fn reads_from_working_ram_shadow() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xE002), 0x2B);
}

#[test]
fn reads_from_working_ram_shadow_scenario_two() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xF5F0), 0x2B);
}

#[test]
fn reads_from_separate_working_ram_bank() {
    let mut address_bus = setup_test_address_bus();
    address_bus.set_cgb_mode(true);

    address_bus.write_byte(0xFF70, 0x02);

    assert_eq!(address_bus.read_byte(0xD001), 0x22);
    assert_eq!(address_bus.read_byte(0xF001), 0x22); 
}

#[test]
fn reads_from_object_attribute_memory() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFE7B), 0x45);
}

#[test]
fn reads_empty_values_outside_of_object_attribute_memory() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFEEE), 0xFF);
}

#[test]
fn reads_from_zero_page_ram() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFFA0), 0xBB);
}

#[test]
fn reads_from_interrupts_enabled_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFFFF), 0x1F);
}

#[test]
fn reads_from_interrupt_flags_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF0F), 0xEA);
}

#[test]
fn reads_from_timer_divider_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF04), 0x3A);
}

#[test]
fn reads_from_timer_counter_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF05), 0x04);
}

#[test]
fn reads_from_timer_modulo_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF06), 0x02);
}

#[test]
fn reads_from_timer_control_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF07), 0x07);
}

#[test]
fn reads_lcdc_register_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF40), 0x80);
}

#[test]
fn reads_scroll_registers_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF42), 0x55);
    assert_eq!(address_bus.read_byte(0xFF43), 0xA1);
}

#[test]
fn reads_window_position_registers_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF4A), 0xBB);
    assert_eq!(address_bus.read_byte(0xFF4B), 0xDD);
}

#[test]
fn reads_palette_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF47), 0xC1);
}

#[test]
fn reads_ly_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF44), 0x2B);
}

#[test]
fn reads_lyc_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF45), 0xAB);
}

#[test]
fn reads_stat_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF41), 0xD2);
}

#[test]
fn reads_obp0_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF48), 0x1B);
}

#[test]
fn reads_obp1_from_gpu() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF49), 0xE4);
}

#[test]
fn writes_obp0_value_to_gpu() {
    let mut address_bus = setup_test_address_bus();
    address_bus.write_byte(0xFF48, 0xE4);
    assert_eq!(address_bus.read_byte(0xFF48), 0xE4);
}

#[test]
fn writes_obp1_value_to_gpu() {
    let mut address_bus = setup_test_address_bus();
    address_bus.write_byte(0xFF49, 0x1B);
    assert_eq!(address_bus.read_byte(0xFF49), 0x1B);
}

#[test]
fn reads_joyp_register() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF00), 0xD4);
}

#[test]
fn loads_rom_buffer_into_emulator() {
    let mut address_bus = setup_test_address_bus();

    let mut rom = build_rom(CART_TYPE_MBC1, ROM_SIZE_64KB, RAM_SIZE_2KB);
    rom[0] = 0xA0;
    rom[1] = 0xCC;
    rom[2] = 0x3B;
    rom[3] = 0x4C;
    rom[0x146] = 0x00;
    rom[0x147] = 0x01;
    rom[0x7FFF] = 0xD4;
    rom[0x8000] = 0xBB;
    rom[0x8001] = 0xD1;
    let header = address_bus.load_rom_buffer( rom, empty_cartridge_effects()).unwrap();

    assert_eq!(address_bus.read_byte(0x0000), 0xA0);
    assert_eq!(address_bus.read_byte(0x0001), 0xCC);
    assert_eq!(address_bus.read_byte(0x0002), 0x3B);
    assert_eq!(address_bus.read_byte(0x0003), 0x4C);
    assert_eq!(address_bus.read_byte(0x7FFF), 0xD4);
    assert_eq!(address_bus.read_byte(0x8000), 0xB1);

    assert_eq!(header.sgb_support, false);
    assert_eq!(header.type_code, 0x01);
}

#[test]
fn writes_to_video_ram() {
    let mut address_bus = setup_test_address_bus();
    address_bus.write_byte(0x8002, 0xC1);
    assert_eq!(address_bus.gpu.get_video_ram_byte(2), 0xC1);
}

#[test]
fn reads_from_audio_master_control() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF26), 0xF4);
}

#[test]
fn writes_to_audio_master_control() {
    let mut address_bus = setup_test_address_bus();
    address_bus.write_byte(0xFF26, 0x0);
    assert_eq!(address_bus.apu.enabled(), false);
}

#[test]
fn reads_from_sound_panning() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF25), 0xF2);
}

#[test]
fn reads_from_master_volume() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF24), 0xC1);
}

#[test]
fn reads_from_ch1_sweep() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF10), 0xDD);
}

#[test]
fn reads_from_ch1_length_and_duty() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF11), 0xBF);
}

#[test]
fn reads_from_ch1_volume() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF12), 0xAA);
}

#[test]
fn reads_from_ch1_period_low() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF13), 0xFF);
}

#[test]
fn reads_from_ch1_period_high() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF14), 0xFF);
}

#[test]
fn reads_from_ch2_length_and_duty() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF16), 0xFF);
}

#[test]
fn reads_from_ch2_volume() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF17), 0xC1);
}

#[test]
fn reads_from_ch2_period_low() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF18), 0xFF);
}

#[test]
fn reads_from_ch2_period_high() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF19), 0xBF);
}

#[test]
fn reads_from_ch3_dac_enabled() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF1A), 0xFF);
}

#[test]
fn reads_from_ch3_output() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF1C), 0xFF);
}

#[test]
fn reads_from_ch3_period_high() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF1E), 0xFF);
}

#[test]
fn reads_from_wave_pattern_ram() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF30), 0xB1);
    assert_eq!(address_bus.read_byte(0xFF31), 0xB1);
}

#[test]
fn reads_from_ch4_length() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF20), 0xFF);
}

#[test]
fn reads_from_ch4_envelope() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF21), 0xD2);
}

#[test]
fn reads_from_ch4_polynomial() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF22), 0xCE);
}

#[test]
fn reads_from_ch4_control() {
    let address_bus = setup_test_address_bus();
    assert_eq!(address_bus.read_byte(0xFF23), 0xFF);
}

#[test]
fn reads_from_key1() {
    let mut address_bus = setup_test_address_bus();
    address_bus.set_cgb_mode(true);
    address_bus.speed_switch_mut().set_cgb_mode(true);
    assert_eq!(address_bus.read_byte(0xFF4D), 0x0);
}