use c8_core::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use clap::Parser;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    rom: String,
}

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let arg = Args::parse();

    let rom = std::fs::read(arg.rom).expect("File not found");

    let mut emu = Emulator::new();
    emu.load_rom(&rom);

    let ctx = sdl2::init().expect("The window to open");
    let video = ctx.video().expect("Video context");
    let win = video
        .window("Chip-8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("The window to open");

    let mut canvas = win.into_canvas().present_vsync().build().expect("Canvas");

    canvas.clear();
    canvas.present();

    let mut ev_pump = ctx.event_pump().expect("Event pump");

    'main_loop: loop {
        for ev in ev_pump.poll_iter() {
            match ev {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        emu.keypress(k, true)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        emu.keypress(k, false)
                    }
                }
                _ => {}
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            emu.step();
        }
        emu.timers_step();
        draw_screen(&emu, &mut canvas);
    }
}

fn key_to_button(key: Keycode) -> Option<usize> {
    return match key {
        Keycode::Left => Some(0x5),
        Keycode::Right => Some(0x6),
        Keycode::Down => Some(0x7),
        Keycode::Up => Some(0x4),
        _ => None,
    };

    let x = match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x7),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    };
}

fn draw_screen(emu: &Emulator, c: &mut Canvas<Window>) {
    c.set_draw_color(Color::RGB(0, 0, 0));
    c.clear();

    let screen_buffer = emu.display();

    c.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            let _ = c.fill_rect(rect);
        }
    }

    c.present();
}
