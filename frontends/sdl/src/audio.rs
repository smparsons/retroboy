use retroboy::emulator::Emulator;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::collections::VecDeque;

const QUEUE_LOW_THRESHOLD: usize = 1024;

pub struct AudioState {
    left_samples: VecDeque<f32>,
    right_samples: VecDeque<f32>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            left_samples: VecDeque::new(),
            right_samples: VecDeque::new(),
        }
    }

    pub fn queue_samples(&mut self, left: &[f32], right: &[f32]) {
        self.left_samples.extend(left.iter());
        self.right_samples.extend(right.iter());
    }

    pub fn samples_queued(&self) -> usize {
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

pub fn create_device(
    audio_subsystem: &sdl2::AudioSubsystem,
) -> Result<AudioDevice<AudioState>, Box<dyn std::error::Error>> {
    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: Some(1024),
    };

    let device = audio_subsystem.open_playback(None, &audio_spec, |spec| {
        println!("Audio device opened with spec: {:?}", spec);
        AudioState::new()
    })?;

    Ok(device)
}

pub fn pump(device: &mut AudioDevice<AudioState>, emulator: &mut Emulator) {
    let samples_queued = device.lock().samples_queued();

    if samples_queued < QUEUE_LOW_THRESHOLD {
        let (left, right) = emulator.step_until_next_audio_buffer();
        device.lock().queue_samples(left, right);
    }
}
