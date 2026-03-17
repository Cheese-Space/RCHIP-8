mod chip8;
mod database;
use std::io;
use std::{env, fs, fmt};
use crate::chip8::Chip8;
use database::Compatability;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Scancode};
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use std::time::{Instant, Duration};
use sdl3::messagebox;
use std::path::PathBuf;
pub enum EmulatorError {
    SdlInit(sdl3::Error),
    VSubsystem(sdl3::Error),
    Window(sdl3::video::WindowBuildError),
    Io{filename: PathBuf, err: io::Error},
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
            Self::Window(err) => write!(f, "failed to create window: {}", err),
            Self::Io { filename, err } => write!(f, "failed to open {}: {}", filename.display(), err),
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
fn set_key(machine: &mut Chip8, code: Scancode, pressed: bool) {
    // uses the layout of the cosmac vip and hp 48
    match code {
        Scancode::_1 => machine.keys[0x1] = pressed,
        Scancode::_2 => machine.keys[0x2] = pressed,
        Scancode::_3 => machine.keys[0x3] = pressed,
        Scancode::_4 => machine.keys[0xC] = pressed,
        Scancode::Q => machine.keys[0x4] = pressed,
        Scancode::W => machine.keys[0x5] = pressed,
        Scancode::E => machine.keys[0x6] = pressed,
        Scancode::R => machine.keys[0xD] = pressed,
        Scancode::A => machine.keys[0x7] = pressed,
        Scancode::S => machine.keys[0x8] = pressed,
        Scancode::D => machine.keys[0x9] = pressed,
        Scancode::F => machine.keys[0xE] = pressed,
        Scancode::Z => machine.keys[0xA] = pressed,
        Scancode::X => machine.keys[0x0] = pressed,
        Scancode::C => machine.keys[0xB] = pressed,
        Scancode::V => machine.keys[0xF] = pressed,
        _ => ()
    }
}
fn main() -> Result<(), EmulatorError> {
    let path = match env::args().nth(1) {
        Some(f) => PathBuf::from(f),
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
    let title = format!("RCHIP-8 - {}", path.file_name().unwrap().display());
    let window = match video_subsystem.window(&title, 640, 320)
        .position_centered()
        .build() {
            Ok(w) => w,
            Err(err) => return Err(EmulatorError::Window(err))
    };
    let res = database::check_compatability(&buff);
    if let Compatability::NotCompatible | Compatability::NotInList = res {
        let _ = messagebox::show_simple_message_box(messagebox::MessageBoxFlag::WARNING, "warning", &res.to_string(), &window);
        eprintln!("{res}");
    }
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
                    set_key(&mut machine, code, true);
                }
                Event::KeyUp {scancode: Some(code), ..} => {
                    set_key(&mut machine, code, false);
                }
                _ => ()
            }
        }
        // set timers
        if reset_timers.elapsed().as_secs_f64() >= 1 as f64 / 60 as f64 {
            machine.delay_timer = machine.delay_timer.wrapping_sub(1);
            machine.sound_timer = machine.sound_timer.wrapping_sub(1);
            reset_timers = Instant::now();
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