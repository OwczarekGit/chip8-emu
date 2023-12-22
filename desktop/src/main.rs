use c8_core::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use clap::Parser;
use sdl2::{
    controller::Button, event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas,
    video::Window,
};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    rom: String,
    #[arg(short, long)]
    scale: Option<u32>,
    #[arg(short, long)]
    gap: Option<u32>,
}

const SCALE: u32 = 15;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let arg = Args::parse();
    let scale = arg.scale.unwrap_or(SCALE);

    let mut gap: u32 = arg.gap.unwrap_or(0);

    let rom = std::fs::read(arg.rom).expect("File not found");

    let mut emu = Emulator::new();
    emu.load_rom(&rom);

    let ctx = sdl2::init().expect("The window to open");
    let video = ctx.video().expect("Video context");
    let win = video
        .window(
            "Chip-8",
            SCREEN_WIDTH as u32 * scale,
            SCREEN_HEIGHT as u32 * scale,
        )
        .position_centered()
        .build()
        .expect("The window to open");

    let mut canvas = win.into_canvas().present_vsync().build().expect("Canvas");
    let joy = ctx.game_controller().expect("GAME");

    let _ = joy.num_joysticks();
    let _ = joy.open(0);

    canvas.clear();
    canvas.present();

    let mut ev_pump = ctx.event_pump().unwrap();

    let mut save_states: [Option<Emulator>; 4] = [None; 4];

    let mut pause = false;

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
                    match key {
                        Keycode::Minus => gap = gap.saturating_sub(1),
                        Keycode::Equals => gap = gap.saturating_add(1),
                        _ => {}
                    }

                    match key {
                        Keycode::F1 => save_states[0] = Some(emu.snapshot()),
                        Keycode::F2 => save_states[1] = Some(emu.snapshot()),
                        Keycode::F3 => save_states[2] = Some(emu.snapshot()),
                        Keycode::F4 => save_states[3] = Some(emu.snapshot()),
                        _ => {}
                    }

                    let index = match key {
                        Keycode::F5 => Some(0),
                        Keycode::F6 => Some(1),
                        Keycode::F7 => Some(2),
                        Keycode::F8 => Some(4),
                        _ => None,
                    };

                    if let Some(i) = index {
                        if let Some(state) = save_states[i] {
                            emu = state;
                        }
                    }

                    if key == Keycode::Space {
                        pause = !pause;
                    }

                    if let Some(k) = key_to_button(key) {
                        emu.keypress(k, false)
                    }
                }
                Event::ControllerButtonDown { button, .. } => {
                    if let Some(k) = controller_to_button(button) {
                        emu.keypress(k, true);
                    }
                }
                Event::ControllerButtonUp { button, .. } => {
                    if let Some(k) = controller_to_button(button) {
                        emu.keypress(k, false);
                    }
                }
                _ => {}
            }
        }

        if !pause {
            for _ in 0..TICKS_PER_FRAME {
                emu.step();
            }
            emu.timers_step();
        }

        draw_screen(&emu, &mut canvas, scale, gap as i32);
    }
}

fn controller_to_button(btn: Button) -> Option<usize> {
    match btn {
        Button::DPadLeft => Some(0x5),
        Button::DPadRight => Some(0x6),
        Button::A => Some(0x7),
        Button::X => Some(0x4),
        _ => None,
    }
}

fn key_to_button(key: Keycode) -> Option<usize> {
    // return match key {
    //     Keycode::Left => Some(0x5),
    //     Keycode::Right => Some(0x6),
    //     Keycode::Down => Some(0x7),
    //     Keycode::Up => Some(0x4),
    //     _ => None,
    // };

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
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    };
    x
}

fn draw_screen(emu: &Emulator, c: &mut Canvas<Window>, scale: u32, gap: i32) {
    c.set_draw_color(Color::RGB(0, 0, 0));
    c.clear();

    let screen_buffer = emu.display();

    c.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new(
                (x * scale) as i32 + gap,
                (y * scale) as i32 + gap,
                scale - (gap * 2) as u32,
                scale - (gap * 2) as u32,
            );
            let _ = c.fill_rect(rect);
        }
    }

    c.present();
}
