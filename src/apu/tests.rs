use super::*;

fn step_div_apu(apu: &mut Apu, step: u8) {
    apu.divider_apu = step;
    // DIV-APU only steps when fourth bit in divider falls from 1 to 0.
    apu.last_divider_time = 0b10011111;
    apu.step(as_params(0b10100000));
}

fn step_apu_multiple_times(apu: &mut Apu, n: u8) {
    for _ in 0..n {
        apu.step(as_params(0));
    }
}

fn initialize_noise_channel(apu: &mut Apu) {
    apu.enabled = true;
    apu.channel4_mut().set_dac_enabled(true);
    apu.channel4_mut().set_enabled(true);
}

fn initialize_disabled_noise_channel(apu: &mut Apu) {
    apu.enabled = true;
    apu.channel4_mut().set_dac_enabled(true);
    apu.channel4_mut().set_enabled(false);
}

fn as_params(divider: u8) -> ApuParams {
    ApuParams {
        divider,
    }
}

#[test]
fn should_not_decrement_period_divider_when_apu_is_off() {
    let mut apu = Apu::new();
    apu.enabled = false;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().period().divider(), 742); 
}

#[test]
fn should_not_decrement_period_divider_if_channel_1_is_off() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(false);
    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().period().divider(), 742); 
}

#[test]
fn should_decrement_period_divider_for_channel_1() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().period().divider(), 741);
}

#[test]
fn should_decrement_period_divider_for_channel_2() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel2_mut().set_enabled(true);
    apu.channel2_mut().period_mut().set_divider(742);
    apu.channel2_mut().period_mut().set_low(26);
    apu.channel2_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel2().period().divider(), 741); 
}

#[test]
fn should_reload_period_divider_once_it_reaches_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().period().divider(), 742);
}

#[test]
fn should_properly_wrap_period_divider_value_when_decrementing_it() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    for _ in 1..=2 {
        apu.step(as_params(0));
    }
    assert_eq!(apu.channel1().period().divider(), 741);  
}

#[test]
fn should_increment_wave_duty_position_when_period_divider_reaches_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.channel1_mut().set_wave_duty_position(0);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().wave_duty_position(), 1);
}

#[test]
fn should_reset_wave_duty_position_to_zero_when_increased_above_seven() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.channel1_mut().set_wave_duty_position(7);
    apu.step(as_params(0));
    assert_eq!(apu.channel1().wave_duty_position(), 0);
}

#[test]
fn should_increment_divider_apu_every_time_bit_four_of_divider_timer_goes_from_one_to_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.last_divider_time = 0b10011111;
    apu.step(as_params(0b10100000));
    assert_eq!(apu.divider_apu, 1);
}

#[test]
fn should_not_increment_divider_apu_if_bit_four_of_divider_timer_remains_unchanged() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    apu.last_divider_time = 0b10010000;
    apu.step(as_params(0b10010001));
    assert_eq!(apu.divider_apu, 0); 
}

#[test]
fn should_wrap_div_apu_to_zero_when_increased_above_seven() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().period_mut().set_divider(1);
    let current_div_apu = 7;
    step_div_apu(&mut apu, current_div_apu);
    assert_eq!(apu.divider_apu, 0);
}

#[test]
fn should_trigger_channel_1_when_writing_to_channel_1_period_high() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(false);
    apu.channel1_mut().envelope_mut().set_initial_settings(0b10100101);
    apu.set_ch1_period_high(0b10000000);
    assert_eq!(apu.audio_master_control(), 0b11110001);
    assert_eq!(apu.channel1().enabled(), true);
    assert_eq!(apu.channel1().period().high(), 0b10000000);
    assert_eq!(apu.channel1().envelope().current_volume(), 0b1010);
    assert_eq!(apu.channel1().envelope().timer(), 0b101)
}

#[test]
fn should_trigger_channel_2_when_writing_to_channel_2_period_high() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel2_mut().set_dac_enabled(true);
    apu.channel2_mut().set_enabled(false);
    apu.set_ch2_period_high(0b10000000);
    assert_eq!(apu.audio_master_control(), 0b11110010);
    assert_eq!(apu.channel2().enabled(), true);
    assert_eq!(apu.channel2().period().high(), 0b10000000);
}

#[test]
fn should_not_trigger_channel_1_if_trigger_bit_is_not_set_when_writing_to_channel_1_period_high() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(false);
    apu.channel1_mut().period_mut().set_high(0b00000001);
    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel1().enabled(), false);
    assert_eq!(apu.channel1().period().high(), 0b00000001); 
}

#[test]
fn should_not_trigger_channel_1_if_dac_is_disabled_when_writing_to_channel_1_period_high() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(false);
    apu.channel1_mut().set_enabled(false);
    apu.channel1_mut().period_mut().set_high(0b10000000);
    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel1().enabled(), false);
    assert_eq!(apu.channel1().period().high(), 0b10000000);
}

#[test]
fn should_disable_dac_and_channel_1_when_writing_to_channel_1_volume() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(true);
    apu.set_ch1_envelope_settings(0b00000001);
    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel1().dac_enabled(), false);
    assert_eq!(apu.channel1().enabled(), false);
    assert_eq!(apu.channel1().envelope().initial_settings(), 0b00000001);
}

#[test]
fn should_disable_dac_and_channel_2_when_writing_to_channel_2_volume() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel2_mut().set_dac_enabled(true);
    apu.channel2_mut().set_enabled(true);
    apu.set_ch2_envelope_settings(0b00000001);
    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel2_mut().dac_enabled(), false);
    assert_eq!(apu.channel2_mut().enabled(), false);
    assert_eq!(apu.channel2_mut().envelope_mut().initial_settings(), 0b00000001);
}

#[test]
fn should_not_disable_dac_if_bits_three_through_seven_have_values_when_writing_to_channel_1_volume() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().envelope_mut().set_initial_settings(0b00101001);
    assert_eq!(apu.audio_master_control(), 0b11110001);
    assert_eq!(apu.channel1().dac_enabled(), true);
    assert_eq!(apu.channel1().enabled(), true);
    assert_eq!(apu.channel1().envelope().initial_settings(), 0b00101001);
}

#[test]
fn should_update_channel_1_envelope_volume_and_reset_timer_when_timer_decrements_to_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);    
    
    apu.channel1_mut().envelope_mut().set_initial_settings(0b10100101);
    apu.channel1_mut().envelope_mut().set_current_volume(0b1010);
    apu.channel1_mut().envelope_mut().set_timer(0b1);

    let current_div_apu = 7;
    step_div_apu(&mut apu, current_div_apu);
     
    assert_eq!(apu.channel1().envelope().current_volume(), 0b1001);
    assert_eq!(apu.channel1().envelope().timer(), 0b101);
}

#[test]
fn should_decrement_channel_1_envelope_timer() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);
    
    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    
    apu.channel1_mut().envelope_mut().set_initial_settings(0b10100101);
    apu.channel1_mut().envelope_mut().set_current_volume(0b1010);
    apu.channel1_mut().envelope_mut().set_timer(0b101);

    let current_div_apu = 7;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel1().envelope().current_volume(), 0b1010);
    assert_eq!(apu.channel1().envelope().timer(), 0b100);
}


#[test]
fn should_not_step_channel_1_envelope_if_divider_apu_is_on_wrong_step() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);

    apu.channel1_mut().envelope_mut().set_initial_settings(0b10100101);
    apu.channel1_mut().envelope_mut().set_current_volume(0b1010);
    apu.channel1_mut().envelope_mut().set_timer(0b101);    

    let current_div_apu = 4;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel1().envelope().current_volume(), 0b1010);
    assert_eq!(apu.channel1().envelope().timer(), 0b101);
}

#[test]
fn should_step_channel_1_length_timer() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);
    
    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    
    apu.channel1_mut().length_mut().set_initial_settings(0b01001110);
    apu.channel1_mut().length_mut().set_timer(0b00000110);
    apu.channel1_mut().period_mut().set_high(0b11000110);

    let current_div_apu = 0;
    step_div_apu(&mut apu, current_div_apu);
    assert_eq!(apu.channel1().length().timer(), 0b00000101);    
}

#[test]
fn should_not_step_channel_1_length_timer_if_divider_apu_is_on_wrong_step() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);
    
    apu.channel1_mut().length_mut().set_initial_settings(0b01001110);   
    apu.channel1_mut().length_mut().set_timer(0b00000110);
    apu.channel1_mut().period_mut().set_high(0b11000110);
    
    let current_div_apu = 1;
    step_div_apu(&mut apu, current_div_apu);    
    assert_eq!(apu.channel1().length().timer(), 0b00000110);
}

#[test]
fn should_disable_channel_1_when_length_timer_reaches_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(26);
    apu.channel1_mut().period_mut().set_high(197);

    apu.channel1_mut().length_mut().set_initial_settings(0b01001110);
    apu.channel1_mut().length_mut().set_timer(0b1);
    apu.channel1_mut().period_mut().set_high(0b11000110);

    let current_div_apu = 0;
    step_div_apu(&mut apu, current_div_apu);
    
    assert_eq!(apu.channel1().length().timer(), 0);
    assert_eq!(apu.channel1().enabled(), false);
}

#[test]
fn should_initialize_length_timer_when_channel_1_is_triggered() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(false);
    apu.set_ch1_period_high(0b11000000);
    assert_eq!(apu.audio_master_control(), 0b11110001);
    assert_eq!(apu.channel1().enabled(), true);
    assert_eq!(apu.channel1().period().high(), 0b11000000);
    assert_eq!(apu.channel1().length().timer(), 0b01000000);
}

#[test]
fn should_decrement_channel_1_sweep_timer() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(0b00011010);
    apu.channel1_mut().period_mut().set_high(0b10000101);    

    apu.channel1_mut().sweep_mut().set_initial_settings(0b00100010);    
    apu.channel1_mut().sweep_mut().set_enabled(true);
    apu.channel1_mut().sweep_mut().set_timer(0b10);
    apu.channel1_mut().sweep_mut().set_shadow_frequency(0b10100011010);

    let current_div_apu = 2;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel1().sweep().timer(), 0b01);    
}

#[test]
fn should_disable_channel_1_on_sweep_overflow() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(0b11111110);
    apu.channel1_mut().period_mut().set_high(0b10000111);    

    apu.channel1_mut().sweep_mut().set_initial_settings(0b00100010);
    apu.channel1_mut().sweep_mut().set_enabled(true);
    apu.channel1_mut().sweep_mut().set_timer(0b01);
    apu.channel1_mut().sweep_mut().set_shadow_frequency(0b11111111110);    

    let current_div_apu = 2;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel1().enabled(), false);
}

#[test]
fn should_reload_sweep_timer_and_frequency_when_timer_reaches_zero() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_enabled(true);
    apu.channel1_mut().set_dac_enabled(true);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(0b00011010);
    apu.channel1_mut().period_mut().set_high(0b10000101);

    apu.channel1_mut().sweep_mut().set_initial_settings(0b00100010);
    apu.channel1_mut().sweep_mut().set_enabled(true);
    apu.channel1_mut().sweep_mut().set_timer(0b01);
    apu.channel1_mut().sweep_mut().set_shadow_frequency(0b10100011010);    

    let current_div_apu = 2;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel1().sweep().timer(), 0b010);
    assert_eq!(apu.channel1().sweep().shadow_frequency(), 0b11001100000);
    assert_eq!(apu.channel1().period().low(), 0b01100000);
    assert_eq!(apu.channel1().period().high(), 0b10000110);
}

#[test]
fn should_properly_initialize_sweep_timer_and_shadow_frequency_on_trigger() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel1_mut().set_dac_enabled(true);
    apu.channel1_mut().set_enabled(false);

    apu.channel1_mut().period_mut().set_divider(742);
    apu.channel1_mut().period_mut().set_low(0b00011010);

    apu.channel1_mut().sweep_mut().set_initial_settings(0b00100010);    

    apu.set_ch1_period_high(0b10100010);

    assert_eq!(apu.channel1().sweep().timer(), 0b010);
    assert_eq!(apu.channel1().sweep().shadow_frequency(), 0b01000011010);
    assert_eq!(apu.channel1().sweep().enabled(), true);
}

#[test]
fn should_decrement_period_divider_for_channel_3() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().set_enabled(true);
    apu.channel3_mut().period_mut().set_divider(742);
    apu.channel3_mut().period_mut().set_low(26);
    apu.channel3_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel3().period().divider(), 740);
}

#[test]
fn should_reload_period_divider_once_it_reaches_zero_for_channel_3() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_enabled(true);
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().period_mut().set_divider(2);
    apu.channel3_mut().period_mut().set_low(26);
    apu.channel3_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel3().period().divider(), 742);
}

#[test]
fn should_increment_wave_position_when_period_divider_reaches_zero_for_channel_3() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_enabled(true);
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().set_wave_position(0);
    apu.channel3_mut().period_mut().set_divider(2);
    apu.channel3_mut().period_mut().set_low(26);
    apu.channel3_mut().period_mut().set_high(197);
    apu.step(as_params(0));
    assert_eq!(apu.channel3().wave_position(), 1);
}

#[test]
fn should_trigger_channel_3_when_writing_to_channel_3_period_high() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().set_enabled(false);
    apu.set_ch3_period_high(0b10000000);
    assert_eq!(apu.audio_master_control(), 0b11110100);
    assert_eq!(apu.channel3().enabled(), true);
    assert_eq!(apu.channel3().period().high(), 0b10000000);
}

#[test]
fn should_disable_channel_3_when_resetting_bit_7_of_dac_enabled_register() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().set_enabled(true);
    apu.set_ch3_dac_enabled(0);
    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel3().enabled(), false);
}

#[test]
fn should_initialize_length_timer_when_channel_3_is_triggered() {
    let mut apu = Apu::new();
    apu.enabled = true;
    apu.channel3_mut().set_dac_enabled(true);
    apu.channel3_mut().set_enabled(false);
    apu.set_ch3_period_high(0b11000000);
    assert_eq!(apu.audio_master_control(), 0b11110100);
    assert_eq!(apu.channel3().enabled(), true);
    assert_eq!(apu.channel3().period().high(), 0b11000000);
    assert_eq!(apu.channel3().length().timer(), 0b100000000);
}

#[test]
fn should_step_channel_3_length_timer() {
    let mut apu = Apu::new();
    apu.enabled = true;

    apu.channel3_mut().set_enabled(true);
    apu.channel3_mut().set_dac_enabled(true);

    apu.channel3_mut().period_mut().set_divider(742);
    apu.channel3_mut().period_mut().set_low(26);
    apu.channel3_mut().period_mut().set_high(197);
    
    apu.channel3_mut().length_mut().set_initial_settings(0b01001110);
    apu.channel3_mut().length_mut().set_timer(0b00000110);
    apu.channel3_mut().period_mut().set_high(0b11000110);

    let current_div_apu = 0;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel3().length().timer(), 0b00000101);
}

#[test]
fn should_not_decrement_period_divider_for_channel_4_if_only_four_instruction_cycles() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().set_period_divider(742);

    apu.step(as_params(0));

    assert_eq!(apu.channel4().period_divider(), 742);    
}

#[test]
fn should_reload_period_divider_for_channel_4_once_it_decrements_to_zero() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().set_period_divider(1);

    // Base Divisor = 0b110 = 6 which maps to 96
    // Shift Amount = 0b0011 = 3
    // 96 << 3 = 768
    apu.channel4_mut().set_polynomial(0b00110110);

    step_apu_multiple_times(&mut apu, 4);

    assert_eq!(apu.channel4().period_divider(), 768);
}

#[test]
fn should_calculate_next_lfsr_value_correctly_for_channel_4() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().set_period_divider(1);
    apu.channel4_mut().set_polynomial(0b00110110);
    apu.channel4_mut().set_lfsr(0b0010010000101100);

    step_apu_multiple_times(&mut apu, 4);
    
    assert_eq!(apu.channel4().lfsr(), 0b0101001000010110);
}

#[test]
fn should_calculate_next_lfsr_value_correctly_in_width_mode_for_channel_4() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().set_period_divider(1);
    apu.channel4_mut().set_polynomial(0b00111110);
    apu.channel4_mut().set_lfsr(0b0010010000101100);

    step_apu_multiple_times(&mut apu, 4);
    
    assert_eq!(apu.channel4().lfsr(), 0b0101001001010110);
}

#[test]
fn should_trigger_channel_4() {
    let mut apu = Apu::new();
    initialize_disabled_noise_channel(&mut apu);

    apu.set_ch4_control(0b10000000);

    assert_eq!(apu.channel4().enabled(), true);
}

#[test]
fn should_set_ch4_control() {
    let mut apu = Apu::new();
    initialize_disabled_noise_channel(&mut apu);

    apu.set_ch4_control(0b10000000);

    assert_eq!(apu.channel4.control(), 0b10000000);
}

#[test]
fn should_disable_channel_4() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.set_ch4_envelope_settings(0b00000111);

    assert_eq!(apu.audio_master_control(), 0b11110000);
    assert_eq!(apu.channel4().enabled(), false);
    assert_eq!(apu.channel4().dac_enabled(), false);
}

#[test]
fn should_step_channel_4_length_timer() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().length_mut().set_initial_settings(0b01001110);
    apu.channel4_mut().length_mut().set_timer(0b00000110);
    apu.channel4_mut().set_control(0b11000000);

    let current_div_apu = 0;
    step_div_apu(&mut apu, current_div_apu);
    
    assert_eq!(apu.channel4().length().timer(), 0b00000101);
}

#[test]
fn should_step_channel_4_envelope_timer() {
    let mut apu = Apu::new();
    initialize_noise_channel(&mut apu);

    apu.channel4_mut().envelope_mut().set_initial_settings(0b10100101);
    apu.channel4_mut().envelope_mut().set_current_volume(0b1010);
    apu.channel4_mut().envelope_mut().set_timer(0b101);    

    let current_div_apu = 7;
    step_div_apu(&mut apu, current_div_apu);

    assert_eq!(apu.channel4().envelope().current_volume(), 0b1010);
    assert_eq!(apu.channel4().envelope().timer(), 0b100);
}
