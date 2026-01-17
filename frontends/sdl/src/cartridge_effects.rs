use retroboy::emulator::{CartridgeEffects, RTCState};

pub struct DesktopCartridgeEffects {}

impl CartridgeEffects for DesktopCartridgeEffects {
    fn current_time_millis(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }

    fn load_rtc_state(&self, _key: &str) -> Option<RTCState> {
        None
    }

    fn save_rtc_state(&self, _key: &str, _value: &RTCState) {}

    fn load_ram(&self, _key: &str) -> Option<Vec<u8>> {
        None
    }

    fn save_ram(&self, _key: &str, _value: &[u8]) {}
}
