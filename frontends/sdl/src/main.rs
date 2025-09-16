use retroboy::emulator::{Emulator, Key, CartridgeEffects, RTCState};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const GAME_BOY_WIDTH: u32 = 160;
const GAME_BOY_HEIGHT: u32 = 144;
const DEFAULT_SCALE_FACTOR: u32 = 4;
const DEFAULT_WINDOW_WIDTH: u32 = GAME_BOY_WIDTH * DEFAULT_SCALE_FACTOR;
const DEFAULT_WINDOW_HEIGHT: u32 = GAME_BOY_HEIGHT * DEFAULT_SCALE_FACTOR;

struct WindowState {
    width: u32,
    height: u32,
}

static FRAME_BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

fn renderer(buffer: &[u8]) {
    if let Some(frame_buffer) = FRAME_BUFFER.get() {
        if let Ok(mut fb) = frame_buffer.lock() {
            fb.copy_from_slice(buffer);
        }
    }
}

struct AudioState {
    left_samples: VecDeque<f32>,
    right_samples: VecDeque<f32>,
}

impl AudioState {
    fn queue_samples(&mut self, left: &[f32], right: &[f32]) {
        self.left_samples.extend(left.iter());
        self.right_samples.extend(right.iter());
    }
    
    fn samples_queued(&self) -> usize {
        self.left_samples.len()
    }
}

impl AudioCallback for AudioState {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for (i, sample) in out.iter_mut().enumerate() {
            if i % 2 == 0 {
                *sample = self.left_samples.pop_front().unwrap_or(0.0);
            } else {
                *sample = self.right_samples.pop_front().unwrap_or(0.0);
            }
        }
    }
}

fn parse_args() -> Result<(Option<Vec<u8>>, bool), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut cgb_mode = false;

    let rom_data = if args.len() == 1 {
        None
    } else if args.len() == 2 {
        if args[1] == "--cgb" {
            cgb_mode = true;
            None 
        } else {
            Some(fs::read(&args[1])?)
        }
    } else if args.len() == 3 {
        cgb_mode = args[2] == "--cgb";
        Some(fs::read(&args[1])?)
    } else {
        eprintln!("Usage: {} [rom_file] [--cgb]", args[0]);
        eprintln!("  rom_file: Path to ROM file (optional - will show file dialog if not provided)");
        eprintln!("  --cgb: Run in Color Game Boy mode");
        std::process::exit(1);
    };

    Ok((rom_data, cgb_mode))
}

fn show_file_dialog() -> Result<Option<String>, Box<dyn std::error::Error>> {
    use rfd::FileDialog;

    let file = FileDialog::new()
        .add_filter("Game Boy ROMs", &["gb", "gbc"])
        .add_filter("All files", &["*"])
        .set_title("Select a Game Boy ROM")
        .pick_file();

    match file {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

fn run_game_loop(
    sdl_context: sdl2::Sdl,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &mut sdl2::render::Texture,
    audio_device: &mut AudioDevice<AudioState>,
    emulator: &mut Emulator,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event_pump = sdl_context.event_pump()?;
    const AUDIO_QUEUE_LOW_THRESHOLD: usize = 1024;

    let frame_duration = Duration::from_nanos(1_000_000_000 / 60);
    let mut last_frame = Instant::now();
    let mut window_state = WindowState {
        width: DEFAULT_WINDOW_WIDTH,
        height: DEFAULT_WINDOW_HEIGHT,
    };

    'running: loop {
        if handle_events(&mut event_pump, emulator, &mut window_state) {
            break 'running;
        }

        let samples_queued = {
            let audio_lock = audio_device.lock();
            audio_lock.samples_queued()
        };

        if samples_queued < AUDIO_QUEUE_LOW_THRESHOLD {
            let (left_samples, right_samples) = emulator.step_until_next_audio_buffer();
            {
                let mut audio_lock = audio_device.lock();
                audio_lock.queue_samples(left_samples, right_samples);
            }
        }

        let now = Instant::now();
        if now.duration_since(last_frame) >= frame_duration {
            update_texture(texture)?;
            canvas.clear();

            let (scaled_width, scaled_height, x_offset, y_offset) = calculate_scaled_dimensions(&window_state);
            canvas.copy(texture, None, Some(Rect::new(x_offset, y_offset, scaled_width, scaled_height)))?;
            canvas.present();
            last_frame = now;
        } else {
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    Ok(())
}

fn update_texture(texture: &mut sdl2::render::Texture) -> Result<(), Box<dyn std::error::Error>> {
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        if let Some(frame_buffer) = FRAME_BUFFER.get() {
            if let Ok(frame_buffer) = frame_buffer.lock() {
                for y in 0..GAME_BOY_HEIGHT {
                    let src_row_start = (y * GAME_BOY_WIDTH * 4) as usize;
                    let dst_row_start = y as usize * pitch;

                    for x in 0..GAME_BOY_WIDTH {
                        let src_idx = src_row_start + (x * 4) as usize;
                        let dst_idx = dst_row_start + (x * 3) as usize;

                        if src_idx + 2 < frame_buffer.len() && dst_idx + 2 < buffer.len() {
                            buffer[dst_idx] = frame_buffer[src_idx];
                            buffer[dst_idx + 1] = frame_buffer[src_idx + 1];
                            buffer[dst_idx + 2] = frame_buffer[src_idx + 2];
                        }
                    }
                }
            }
        }
    })?;
    Ok(())
}

fn calculate_scaled_dimensions(window_state: &WindowState) -> (u32, u32, i32, i32) {
    let scale_x = window_state.width / GAME_BOY_WIDTH;
    let scale_y = window_state.height / GAME_BOY_HEIGHT;

    let scale = scale_x.min(scale_y).max(1);

    let scaled_width = GAME_BOY_WIDTH * scale;
    let scaled_height = GAME_BOY_HEIGHT * scale;

    let x_offset = ((window_state.width as i32) - (scaled_width as i32)) / 2;
    let y_offset = ((window_state.height as i32) - (scaled_height as i32)) / 2;

    (scaled_width, scaled_height, x_offset, y_offset)
}

fn handle_events(event_pump: &mut sdl2::EventPump, emulator: &mut Emulator, window_state: &mut WindowState) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => return true,
            Event::KeyDown { keycode: Some(keycode), repeat: false, .. } => {
                if let Some(key) = map_keycode_to_game_boy_key(keycode) {
                    emulator.handle_key_press(&key);
                }
            }
            Event::KeyUp { keycode: Some(keycode), repeat: false, .. } => {
                if let Some(key) = map_keycode_to_game_boy_key(keycode) {
                    emulator.handle_key_release(&key);
                }
            }
            Event::Window { win_event, .. } => {
                match win_event {
                    sdl2::event::WindowEvent::FocusGained => {
                        // Window gained focus - good for debugging
                    }
                    sdl2::event::WindowEvent::FocusLost => {
                        // Window lost focus
                    }
                    sdl2::event::WindowEvent::Resized(width, height) => {
                        window_state.width = width as u32;
                        window_state.height = height as u32;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    false
}

fn setup_emulator(rom_data: &[u8], cgb_mode: bool) -> Result<Emulator, Box<dyn std::error::Error>> {
    let mut emulator = Emulator::new(renderer, false);
    emulator.set_cgb_mode(cgb_mode);
    emulator.set_sample_rate(44100);

    match emulator.load_rom(rom_data, Box::new(DesktopCartridgeEffects {})) {
        Ok(_) => println!("ROM loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            std::process::exit(1);
        }
    }

    Ok(emulator)
}

fn init_sdl() -> Result<(sdl2::Sdl, sdl2::render::Canvas<sdl2::video::Window>, AudioDevice<AudioState>), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let mut window = video_subsystem
        .window("RetroBoy", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()?;

    window.show();
    window.raise();

    let canvas = window.into_canvas().build()?;

    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: Some(1024),
    };

    let audio_device: AudioDevice<AudioState> = audio_subsystem.open_playback(None, &audio_spec, |spec| {
        println!("Audio device opened with spec: {:?}", spec);
        AudioState {
            left_samples: VecDeque::new(),
            right_samples: VecDeque::new(),
        }
    })?;

    Ok((sdl_context, canvas, audio_device))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rom_data_opt, cgb_mode) = parse_args()?;

    let frame_buffer = Arc::new(Mutex::new(vec![0u8; (GAME_BOY_WIDTH * GAME_BOY_HEIGHT * 4) as usize]));
    FRAME_BUFFER.set(frame_buffer).map_err(|_| "Failed to set frame buffer")?;

    let (sdl_context, mut canvas, mut audio_device) = init_sdl()?;

    let rom_data = match rom_data_opt {
        Some(data) => data,
        None => {
            let rom_path = show_file_dialog()?;
            let rom_path = match rom_path {
                Some(path) => path,
                None => {
                    eprintln!("No ROM file selected.");
                    std::process::exit(1);
                }
            };

            fs::read(&rom_path)?
        }
    };

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        GAME_BOY_WIDTH,
        GAME_BOY_HEIGHT,
    )?;
    let mut emulator = setup_emulator(&rom_data, cgb_mode)?;

    audio_device.resume();

    run_game_loop(sdl_context, &mut canvas, &mut texture, &mut audio_device, &mut emulator)?;

    Ok(())
}

fn map_keycode_to_game_boy_key(keycode: Keycode) -> Option<Key> {
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

struct DesktopCartridgeEffects {}

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