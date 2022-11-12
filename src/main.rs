//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;

mod entities;
mod sdlx;
mod playdata;
use playdata::rcg::Rcg;
use playdata::PlayData;

fn identify() {
    println!("Playback Playground");
    if cfg!(feature = "inbetween_frames") {
        println!("INFO:PPGND:Mode: InBetween Frames");
    } else {
        println!("INFO:PPGND:Mode: OnlyProvided Frames");
    }
}

fn main() {
    identify();
    let ttfx = sdl2::ttf::init().unwrap();
    let font = ttfx.load_font("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 16).unwrap();
    let mut sx = sdlx::SdlX::init_plus(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT);

    let mut dcolor = 20;
    let mut pgentities = entities::Entities::new(11, 11, &font);

    let clargs = env::args().collect::<Vec<String>>();
    let mut rcg: Option<Rcg> = None;
    if clargs.len() > 1 {
        rcg = Some(Rcg::new(&clargs[1]));
    }

    let mut bpause = false;
    let mut frame: usize = 0;
    'mainloop: loop {
        frame += 1;
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
                        Keycode::D => {
                            print!("DBUG:PGND:Main:Entities:{:#?}\n", pgentities);
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
            let rcg = rcg.as_mut().unwrap();
            if !rcg.bdone {
                if cfg!(feature = "inbetween_frames") {
                    if rcg.next_frame_is_record_ready() {
                        let tu = rcg.next_record();
                        print!("DBUG:{:?}\n", tu);
                        pgentities.update(tu, false);
                    }
                    // TODO: Need to let this run for Fps frames ideally, even after bdone is set
                    // Or Rcg needs to be udpated to set bdone after a second of ending or so ...
                    pgentities.next_frame();
                } else {
                    let pu = rcg.next_record();
                    pgentities.update(pu, true);
                }
            }
        }

        // Draw entities
        pgentities.draw(&mut sx);

        sx.wc.present();
        std::thread::sleep(std::time::Duration::from_millis((1000/entities::FRAMES_PER_SEC) as u64));
    }

}
