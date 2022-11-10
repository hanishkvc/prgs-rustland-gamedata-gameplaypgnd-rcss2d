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

fn setup_entities(nplayers: i32) -> Vec<Entity> {
    let mut vplayers = Vec::new();
    for i in 0..nplayers {
        let iy: i32 = (rand::random::<u32>() % entity::SCREEN_HEIGHT) as i32;
        vplayers.push(Entity::new((i*20i32,iy), Color::RGB(200, 0, 0)));
    }
    return vplayers;
}

fn update_entities(vplayers: &mut Vec<Entity>, bpos: i32) {
    vplayers[0].pos_set_abs(bpos, bpos);
    for i in 1..vplayers.len() {
        let dy: i32 = (rand::random::<i32>() % 4) as i32;
        vplayers[i].pos_set_rel(1, dy);
    }
}


fn main() {
    println!("Hello, world!");
    let (_sctxt, _sv, mut swc, mut se) = sdl_init();
    let mut dcolor = 20;
    let mut players = setup_entities(12*2);

    'mainloop: loop {
        // Clear the background
        swc.set_draw_color(Color::RGB(20+dcolor, 200, 20));
        swc.clear();

        // handle any pending events
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

        // Update the entities
        update_entities(&mut players, dcolor as i32);

        // Draw entities
        for i in 0..players.len() {
            players[i].draw(&mut swc);
        }

        swc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
