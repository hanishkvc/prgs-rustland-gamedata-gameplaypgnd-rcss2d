//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, render::WindowCanvas, pixels::Color, ttf::{self, Font}};
use sdl2::render::{TextureCreator};
use sdl2::video::WindowContext;


mod entities;
use entities::gentity::Entity;


fn sdl_init() -> (Sdl, VideoSubsystem, WindowCanvas, EventPump) {
    let sctxt = sdl2::init().unwrap();
    let sv = sctxt.video().unwrap();
    let sw = sv.window("Playback", entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT).build().unwrap();
    let swc = sw.into_canvas().build().unwrap();
    let se = sctxt.event_pump().unwrap();
    //sdl2::gfx::primitives::set_font(fontdata, cw, ch);
    return (sctxt, sv, swc, se);
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
    let stx = ttf::init().unwrap();
    let font = stx.load_font("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 16).unwrap();
    let swctc = swc.texture_creator();

    let mut dcolor = 20;
    let mut pgentities = entities::Entities::new(11, 11); //setup_entities(12*2, &font, &swctc);

    let mut bpause = false;
    'mainloop: loop {
        // Clear the background
        swc.set_draw_color(entities::screen_color_bg_rel(dcolor, 0, 0));
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
                        Keycode::P => {
                            bpause = !bpause;
                        }
                        _ => {

                        }
                    }
                },
                _ => (),
            }
        }

        // Update the entities
        if !bpause {
            update_entities(&mut players, dcolor as i32);
        }

        // Draw entities

        swc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
