//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, render::WindowCanvas, pixels::Color};


mod entity;
use entity::Entity;


fn sdl_init() -> (Sdl, VideoSubsystem, WindowCanvas, EventPump) {
    let sctxt = sdl2::init().unwrap();
    let sv = sctxt.video().unwrap();
    let sw = sv.window("Playback", 800, 600).build().unwrap();
    let swc = sw.into_canvas().build().unwrap();
    let se = sctxt.event_pump().unwrap();
    return (sctxt, sv, swc, se);
}

fn setup_entities(nplayers: usize) -> Vec<Entity> {
    let mut vplayers = Vec::new();
    for _i in 0..nplayers {
        vplayers.push(Entity::new((0,0), Color::RGB(200, 0, 0)));
    }
    return vplayers;
}


fn main() {
    println!("Hello, world!");
    let (_sctxt, _sv, mut swc, mut se) = sdl_init();
    let mut dcolor = 20;
    let mut players = setup_entities(2);

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
        players[0].pos_set(dcolor as i32, dcolor as i32);
        players[1].pos_update(1, 1);

        for player in players {
            player.draw(&mut swc);
        }

        swc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
