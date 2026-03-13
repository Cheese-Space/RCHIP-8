mod chip8;
use std::{env, fs, process::ExitCode};
use crate::chip8::Chip8;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Scancode};
use sdl3::pixels::Color;
use sdl3::rect::Rect;
use std::time::Duration;
fn set_key(machine: &mut Chip8, code: Scancode, pressed: bool) {
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
fn main() -> ExitCode {
    let filename = match env::args().nth(1) {
        Some(f) => f,
        None => {
            eprintln!("no filename provided!");
            return ExitCode::FAILURE;
        }
    };
    let buff = match fs::read(&filename) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to open {filename}: {}", e);
            return ExitCode::FAILURE;
        }
    };
    let mut machine = Chip8::init(&buff); // load program and font into memory, sets registers and sets timers
    let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("RCHIP-8", 640, 320)
            .position_centered()
            .build()
            .unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();  
        'running: loop {
            // poll for events
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
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
            if machine.delay_timer == 0 {
                machine.delay_timer = 255;
            }
            else {
                machine.delay_timer -= 1;
            }
            if machine.sound_timer == 0 {
                machine.sound_timer = 255;
            }
            else {
                machine.sound_timer -= 1;
            }
            machine.exec();
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
            // ^todo: other logic
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 700));
        }
    ExitCode::SUCCESS
}