//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!


mod entities;
mod sdlx;


fn main() {
    println!("Hello, world!");
    let ttfx = sdl2::ttf::init().unwrap();
    let font = ttfx.load_font("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 16).unwrap();
    let mut sx = sdlx::SdlX::init_plus(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT);

    let mut dcolor = 20;
    let mut pgentities = entities::Entities::new(11, 11, &font);

    let mut bpause = false;
    let mut step = -1;
    'mainloop: loop {
        step += 1;
        // Clear the background
        sx.wc.set_draw_color(entities::screen_color_bg_rel(dcolor, 0, 0));
        sx.wc.clear();

        // handle any pending events
        for ev in sx.ep.poll_iter() {
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
            pgentities.update(step as usize);
        }

        // Draw entities
        pgentities.draw(&mut sx);

        sx.wc.present();
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}
