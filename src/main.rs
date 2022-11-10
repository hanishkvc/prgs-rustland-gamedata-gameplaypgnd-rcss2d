//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, render::WindowCanvas, pixels};

fn sdl_init() -> (Sdl, VideoSubsystem, WindowCanvas, EventPump) {
    let sctxt = sdl2::init().unwrap();
    let sv = sctxt.video().unwrap();
    let sw = sv.window("Playback", 800, 600).build().unwrap();
    let swc = sw.into_canvas().build().unwrap();
    let se = sctxt.event_pump().unwrap();
    return (sctxt, sv, swc, se);
}


fn main() {
    println!("Hello, world!");
    let (_sctxt, _sv, mut swc, mut se) = sdl_init();
    let mut dcolor = 20;

    'mainloop: loop {
        swc.set_draw_color(pixels::Color::RGB(20+dcolor, 200, 20));
        swc.clear();
        for ev in se.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            match ev {
                Event::Quit { timestamp: _ } => break 'mainloop,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    match keycode.unwrap() {
                        Keycode::W => {
                            dcolor += 20;
                        }
                        _ => {

                        }
                    }
                },
                _ => (),
            }
        }
        swc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
