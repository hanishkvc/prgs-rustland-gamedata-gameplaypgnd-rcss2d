//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, render::WindowCanvas, pixels::Color};

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
        swc.set_draw_color(Color::RGB(20+dcolor, 200, 20));
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
        swc.set_draw_color(Color::RGB(200, 20, 20));
        swc.fill_rect(sdl2::rect::Rect::new(dcolor as i32, dcolor as i32, 16, 16));
        swc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
