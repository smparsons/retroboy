use sdl2::rect::Rect;
use std::sync::{Arc, Mutex, OnceLock};

pub const GAME_BOY_WIDTH: u32 = 160;
pub const GAME_BOY_HEIGHT: u32 = 144;

const BUFFER_SIZE: usize = (GAME_BOY_WIDTH * GAME_BOY_HEIGHT * 4) as usize;

static FRAME_BUFFER: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

pub fn init() -> Result<(), &'static str> {
    let buffer = Arc::new(Mutex::new(vec![0u8; BUFFER_SIZE]));
    FRAME_BUFFER
        .set(buffer)
        .map_err(|_| "Frame buffer already initialized")
}

pub fn callback(buffer: &[u8]) {
    if let Some(fb) = FRAME_BUFFER.get() {
        if let Ok(mut locked) = fb.lock() {
            locked.copy_from_slice(buffer);
        }
    }
}

fn with_frame_buffer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&[u8]) -> R,
{
    FRAME_BUFFER
        .get()
        .and_then(|fb| fb.lock().ok())
        .map(|guard| f(&guard))
}

pub fn render_frame(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &mut sdl2::render::Texture,
    window_width: u32,
    window_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    update_texture(texture)?;
    canvas.clear();

    let (w, h, x, y) = calculate_scaled_dimensions(window_width, window_height);
    canvas.copy(texture, None, Some(Rect::new(x, y, w, h)))?;
    canvas.present();

    Ok(())
}

fn copy_rgba_to_rgb24(src: &[u8], dst: &mut [u8], pitch: usize) {
    for y in 0..GAME_BOY_HEIGHT as usize {
        let src_row = y * GAME_BOY_WIDTH as usize * 4;
        let dst_row = y * pitch;

        for x in 0..GAME_BOY_WIDTH as usize {
            let si = src_row + x * 4;
            let di = dst_row + x * 3;
            dst[di..di + 3].copy_from_slice(&src[si..si + 3]);
        }
    }
}

fn update_texture(texture: &mut sdl2::render::Texture) -> Result<(), Box<dyn std::error::Error>> {
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        with_frame_buffer(|fb| {
            copy_rgba_to_rgb24(fb, buffer, pitch);
        });
    })?;
    Ok(())
}

fn calculate_scaled_dimensions(window_width: u32, window_height: u32) -> (u32, u32, i32, i32) {
    let scale_x = window_width / GAME_BOY_WIDTH;
    let scale_y = window_height / GAME_BOY_HEIGHT;

    let scale = scale_x.min(scale_y).max(1);

    let scaled_width = GAME_BOY_WIDTH * scale;
    let scaled_height = GAME_BOY_HEIGHT * scale;

    let x_offset = ((window_width as i32) - (scaled_width as i32)) / 2;
    let y_offset = ((window_height as i32) - (scaled_height as i32)) / 2;

    (scaled_width, scaled_height, x_offset, y_offset)
}
