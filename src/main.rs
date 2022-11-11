//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!


mod entities;
use entities::gentity::Entity;

mod sdlx;


fn update_entities(vplayers: &mut Vec<Entity>, bpos: i32) {
    vplayers[0].pos_set_abs(bpos, bpos);
    for i in 1..vplayers.len() {
        let dy: i32 = (rand::random::<i32>() % 4) as i32;
        vplayers[i].pos_set_rel(1, dy);
    }
}


fn main() {
    println!("Hello, world!");
    let (_sctxt, _sv, mut swc, mut se) = sdlx::sdl_init(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT);

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
