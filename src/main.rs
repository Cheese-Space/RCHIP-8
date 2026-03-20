mod chip8;
mod database;
use std::io;
use std::{env, fs, fmt};
use crate::chip8::Chip8;
use database::Compatibility;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use std::time::{Instant, Duration};
use sdl3::messagebox;
use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream};
pub enum EmulatorError {
    SdlInit(sdl3::Error),
    VSubsystem(sdl3::Error),
    Audio(sdl3::Error),
    Window(sdl3::video::WindowBuildError),
    Io{filename: String, err: io::Error},
    Nofilename,
    ProgramTooLarge,
    EmptyReturnStack,
    InvalidInstruction(u16)
}
// displayed when returned from main
impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SdlInit(err) => write!(f, "failed to init sdl: {}", err),
            Self::VSubsystem(err) => write!(f, "failed to init sdl's video subsystem: {}", err),
            Self::Audio(err) => write!(f, "failed to init audio: {}", err),
            Self::Window(err) => write!(f, "failed to create window: {}", err),
            Self::Io { filename, err } => write!(f, "failed to open {}: {}", filename, err),
            Self::Nofilename => write!(f, "no filename provided"),
            Self::ProgramTooLarge => write!(f, "program is too large (limit is 3584 bytes)"),
            Self::EmptyReturnStack => write!(f, "tried to get return adress from empty return stack"),
            Self::InvalidInstruction(instruction) => write!(f, "invalid or unimplemented instruction: 0x{:x}", instruction)
        }
    }
}
impl fmt::Debug for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}
impl AudioCallback<f32> for SquareWave {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let mut out = Vec::<f32>::with_capacity(requested as usize);
        // Generate a square wave
        for _ in 0..requested {
            out.push(if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            });
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
        let _ = stream.put_data_f32(&out);
    }
}
fn main() -> Result<(), EmulatorError> {
    let path = match env::args().nth(1) {
        Some(f) => f,
        None => return Err(EmulatorError::Nofilename)
    };
    let buff = match fs::read(&path) {
        Ok(b) => b,
        Err(err) => return Err(EmulatorError::Io { filename: path, err })
    };
    let mut machine = Chip8::init(&buff)?;
    let sdl_context = match sdl3::init() {
        Ok(c) => c,
        Err(err) => return Err(EmulatorError::SdlInit(err))
    };
    let video_subsystem = match sdl_context.video() {
        Ok(v) => v,
        Err(err) => return Err(EmulatorError::VSubsystem(err))
    };
    let audio_subsystem = match sdl_context.audio() {
        Ok(a) => a,
        Err(e) => return Err(EmulatorError::Audio(e))
    };
    let source_freq = 44100;
    let source_spec = AudioSpec {
        freq: Some(source_freq),
        channels: Some(1),                      // mono
        format: Some(AudioFormat::f32_sys())    // floating 32 bit samples
    };
    let device = match audio_subsystem.open_playback_stream(&source_spec, SquareWave {
        phase_inc: 440.0 / source_freq as f32,
        phase: 0.0,
        volume: 0.05
    }) {
        Ok(d) => d,
        Err(e) => return Err(EmulatorError::Audio(e))
    };
    let mut window = match video_subsystem.window("RCHIP-8", 640, 320)
        .position_centered()
        .build() {
            Ok(w) => w,
            Err(err) => return Err(EmulatorError::Window(err))
    };
    machine.get_info(&buff);
    if let Compatibility::NotCompatible | Compatibility::NotInList = machine.info.compatibility {
        let _ = messagebox::show_simple_message_box(messagebox::MessageBoxFlag::WARNING, "compatibility warning", &machine.info.compatibility.to_string(), &window);
        eprintln!("{}", machine.info.compatibility);
    }
    window.set_keyboard_grab(true);
    window.set_title(&format!("RCHIP-8 - {}", machine.info.title)).expect("title shouldn't contain null character");
    let mut event_pump = sdl_context.event_pump().expect("no other event_pump instance should be alive");
    let mut canvas = window.into_canvas();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut reset_timers = Instant::now();
    'running: loop {
        // poll for events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                Event::KeyDown {scancode: Some(code), ..} => {
                    machine.set_key(code, true);
                }
                Event::KeyUp {scancode: Some(code), ..} => {
                    machine.set_key( code, false);
                }
                _ => ()
            }
        }
        // set timers
        if reset_timers.elapsed().as_secs_f64() >= 1 as f64 / 60 as f64 {
            machine.delay_timer = machine.delay_timer.saturating_sub(1);
            machine.sound_timer = machine.sound_timer.saturating_sub(1);
            reset_timers = Instant::now();
        }
        // check audio
        if machine.sound_timer == 0 {
            let _ = device.pause();
        }
        else {
            let _ = device.resume();
        }
        if let Err(error) = machine.exec() {
            let _ = messagebox::show_simple_message_box(messagebox::MessageBoxFlag::ERROR, "error", &format!("error: {error}"), canvas.window());
            return Err(error);
        }
        if machine.refresh_display {
            canvas.clear();
            for y in 0usize..32 {
                for x in 0usize..64 {
                    let rect = Rect::new(x as i32 *10, y  as i32 *10, 10, 10);
                    if machine.display[y][x] {
                            canvas.set_draw_color(Color::RGB(255, 255, 255));
                    }
                    else {
                            canvas.set_draw_color(Color::RGB(0, 0, 0));
                    }
                    canvas.fill_rect(rect).unwrap();
                }
            }
            canvas.present();
            machine.refresh_display = false;
        }
        std::thread::sleep(Duration::from_secs_f64(1 as f64 / 700 as f64));
    }
    Ok(())
}