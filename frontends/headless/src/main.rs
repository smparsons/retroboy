use chrono::Local;
use clap::Parser;
use image::{ImageBuffer, Rgba};
use retroboy::emulator::{CartridgeEffects, Emulator, RTCState};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;
const BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize;

#[derive(Parser)]
#[command(name = "retroboy-headless")]
#[command(about = "Run RetroBoy emulator headlessly without display or audio")]
struct Args {
    #[arg(help = "Path to the ROM file")]
    rom: PathBuf,

    #[arg(help = "Number of seconds to run the emulator")]
    seconds: f64,

    #[arg(long, help = "Capture a screenshot at the end")]
    screenshot: bool,

    #[arg(long, help = "Directory to save the screenshot (defaults to current directory)")]
    screenshot_path: Option<PathBuf>,

    #[arg(long, help = "Run in CGB (Game Boy Color) mode")]
    cgb: bool,
}

static FRAME_BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

fn frame_callback(buffer: &[u8]) {
    if let Some(fb) = FRAME_BUFFER.get() {
        if let Ok(mut locked) = fb.lock() {
            locked.copy_from_slice(buffer);
        }
    }
}

fn init_frame_buffer() {
    let buffer = Arc::new(Mutex::new(vec![0u8; BUFFER_SIZE]));
    let _ = FRAME_BUFFER.set(buffer);
}

fn save_screenshot(rom_path: &PathBuf, output_dir: Option<&PathBuf>) -> Result<PathBuf, String> {
    let frame_data = FRAME_BUFFER
        .get()
        .ok_or("Frame buffer not initialized")?
        .lock()
        .map_err(|_| "Failed to lock frame buffer")?
        .clone();

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(SCREEN_WIDTH, SCREEN_HEIGHT, frame_data)
            .ok_or("Failed to create image buffer")?;

    let rom_stem = rom_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("screenshot");

    let timestamp = Local::now().format("%y%m%d_%H%M%S");
    let filename = format!("{}_{}.png", rom_stem, timestamp);

    let output_path = match output_dir {
        Some(dir) => {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
            dir.join(&filename)
        }
        None => PathBuf::from(&filename),
    };

    img.save(&output_path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(output_path)
}

struct HeadlessCartridgeEffects;

impl CartridgeEffects for HeadlessCartridgeEffects {
    fn current_time_millis(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }

    fn load_rtc_state(&self, _: &str) -> Option<RTCState> {
        None
    }

    fn save_rtc_state(&self, _: &str, _: &RTCState) {}

    fn load_ram(&self, _: &str) -> Option<Vec<u8>> {
        None
    }

    fn save_ram(&self, _: &str, _: &[u8]) {}
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom)
        .map_err(|e| format!("Failed to read ROM file: {}", e))?;

    init_frame_buffer();

    let mut emulator = Emulator::new(frame_callback, false);
    emulator.set_cgb_mode(args.cgb);
    emulator.set_sample_rate(44100);

    let header = emulator
        .load_rom(&rom_data, Box::new(HeadlessCartridgeEffects))
        .map_err(|e| format!("Failed to load ROM: {}", e))?;

    println!("Loaded: {}", header.title);
    println!("Running for {} seconds...", args.seconds);

    let start = Instant::now();
    let duration = Duration::from_secs_f64(args.seconds);
    let mut frame_count: u64 = 0;

    while start.elapsed() < duration {
        let _ = emulator.step_until_next_audio_buffer();
        frame_count += 1;
    }

    let elapsed = start.elapsed();
    println!(
        "Completed {} frames in {:.2}s ({:.1} fps)",
        frame_count,
        elapsed.as_secs_f64(),
        frame_count as f64 / elapsed.as_secs_f64()
    );

    if args.screenshot {
        let path = save_screenshot(&args.rom, args.screenshot_path.as_ref())?;
        println!("Screenshot saved to: {}", path.display());
    }

    Ok(())
}
