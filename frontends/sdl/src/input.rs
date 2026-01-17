use retroboy::emulator::{Emulator, Key};
use sdl2::keyboard::Keycode;

pub fn map_keycode(keycode: Keycode) -> Option<Key> {
    match keycode {
        Keycode::Up => Some(Key::Up),
        Keycode::Down => Some(Key::Down),
        Keycode::Left => Some(Key::Left),
        Keycode::Right => Some(Key::Right),
        Keycode::Return => Some(Key::Start),
        Keycode::Space => Some(Key::Select),
        Keycode::X => Some(Key::B),
        Keycode::Z => Some(Key::A),
        _ => None,
    }
}

pub fn apply_key_action(emulator: &mut Emulator, keycode: Keycode, pressed: bool) {
    if let Some(key) = map_keycode(keycode) {
        if pressed {
            emulator.handle_key_press(&key);
        } else {
            emulator.handle_key_release(&key);
        }
    }
}
