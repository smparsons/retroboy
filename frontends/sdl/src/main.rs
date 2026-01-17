use audio::AudioState;
use renderer::{GAME_BOY_HEIGHT, GAME_BOY_WIDTH};
use retroboy::emulator::Emulator;
use sdl2::audio::AudioDevice;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::fs;
use std::time::{Duration, Instant};

const DEFAULT_SCALE_FACTOR: u32 = 4;
const DEFAULT_WINDOW_WIDTH: u32 = GAME_BOY_WIDTH * DEFAULT_SCALE_FACTOR;
const DEFAULT_WINDOW_HEIGHT: u32 = GAME_BOY_HEIGHT * DEFAULT_SCALE_FACTOR;

struct WindowState {
    width: u32,
    height: u32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        }
    }
}

struct FrameTiming {
    last_frame: Instant,
    frame_duration: Duration,
}

impl FrameTiming {
    fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_duration: Duration::from_nanos(1_000_000_000 / 60),
        }
    }

    fn should_render(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_frame) >= self.frame_duration {
            self.last_frame = now;
            true
        } else {
            false
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

fn show_file_dialog() -> Option<String> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Game Boy ROMs", &["gb", "gbc"])
        .add_filter("All files", &["*"])
        .set_title("Select a Game Boy ROM")
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
}

fn load_rom_data(rom_data_opt: Option<Vec<u8>>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match rom_data_opt {
        Some(data) => Ok(data),
        None => {
            let rom_path = show_file_dialog().ok_or("No ROM file selected")?;
            Ok(fs::read(&rom_path)?)
        }
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
    let mut timing = FrameTiming::new();
    let mut window_state = WindowState::default();

    loop {
        if handle_events(&mut event_pump, emulator, &mut window_state) {
            break;
        }

        audio::pump(audio_device, emulator);

        if timing.should_render() {
            renderer::render_frame(canvas, texture, window_state.width, window_state.height)?;
        } else {
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    Ok(())
}

fn handle_events(
    event_pump: &mut sdl2::EventPump,
    emulator: &mut Emulator,
    window_state: &mut WindowState,
) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => return true,
            Event::KeyDown {
                keycode: Some(keycode),
                repeat: false,
                ..
            } => {
                input::apply_key_action(emulator, keycode, true);
            }
            Event::KeyUp {
                keycode: Some(keycode),
                repeat: false,
                ..
            } => {
                input::apply_key_action(emulator, keycode, false);
            }
            Event::Window { win_event, .. } => {
                if let sdl2::event::WindowEvent::Resized(width, height) = win_event {
                    window_state.width = width as u32;
                    window_state.height = height as u32;
                }
            }
            _ => {}
        }
    }
    false
}

fn setup_emulator(rom_data: &[u8], cgb_mode: bool) -> Result<Emulator, Box<dyn std::error::Error>> {
    let mut emulator = Emulator::new(renderer::callback, false);
    emulator.set_cgb_mode(cgb_mode);
    emulator.set_sample_rate(44100);

    match emulator.load_rom(rom_data, Box::new(cartridge_effects::DesktopCartridgeEffects {})) {
        Ok(_) => println!("ROM loaded successfully!"),
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            std::process::exit(1);
        }
    }

    Ok(emulator)
}

fn init_sdl() -> Result<
    (
        sdl2::Sdl,
        sdl2::render::Canvas<sdl2::video::Window>,
        AudioDevice<AudioState>,
    ),
    Box<dyn std::error::Error>,
> {
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
    let audio_device = audio::create_device(&audio_subsystem)?;

    Ok((sdl_context, canvas, audio_device))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rom_data_opt, cgb_mode) = parse_args()?;

    renderer::init()?;

    let (sdl_context, mut canvas, mut audio_device) = init_sdl()?;
    let rom_data = load_rom_data(rom_data_opt)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        GAME_BOY_WIDTH,
        GAME_BOY_HEIGHT,
    )?;
    let mut emulator = setup_emulator(&rom_data, cgb_mode)?;

    audio_device.resume();

    run_game_loop(
        sdl_context,
        &mut canvas,
        &mut texture,
        &mut audio_device,
        &mut emulator,
    )?;

    Ok(())
}

mod audio;
mod cartridge_effects;
mod input;
mod renderer;
