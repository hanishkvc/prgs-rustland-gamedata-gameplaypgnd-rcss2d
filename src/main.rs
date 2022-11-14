//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;

use sdl2::pixels::Color;

mod entities;
mod sdlx;
mod playdata;
use playdata::rcg::Rcg;
use playdata::random::RandomData;
use playdata::PlayData;
use sdlx::SdlX;

fn show_help(sx: &mut SdlX) {
    let shelp = "** Help **\n\
    \n\
    larrow: seek back\n\
    rarrow: seek forward\n\
    f/F:    change fps\n\
    p:      pause playback\n\
    b:      hide/unhide ball\n\
    h:      hide/unhide help\n\
    \n\
    playbackpgnd path/file.rcg\n\
    ...                   Save Nature Save Earth";

    let vhelp: Vec<&str> = shelp.split('\n').collect();
    sx.n_msgbox((0.3, 0.3,0.4,0.4), vhelp, Color::BLUE);

}

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
    let mut pgentities = entities::PGEntities::new(entities::PITCH_RECT, 11, 11, &font);

    // Setup the playdata source
    let clargs = env::args().collect::<Vec<String>>();
    let mut pdrandom = RandomData::new(20.0, 11, 11);
    let mut pdrcg;
    let pdata: &mut dyn PlayData;
    if clargs.len() > 1 {
        pdrcg = Rcg::new(&clargs[1], pgentities.fps());
        pdata = &mut pdrcg;
    } else {
        pdata = &mut pdrandom;
    }

    let mut bpause = false;
    let mut frame: usize = 0;
    let mut bhelp = false;
    let mut ptime = std::time::Instant::now();
    let mut pframe = 0;
    let mut actualfps = 0;
    'mainloop: loop {
        frame += 1;
        let dtime = std::time::Instant::now().duration_since(ptime);
        if dtime > std::time::Duration::from_millis(1000) {
            ptime = std::time::Instant::now();
            actualfps = frame - pframe;
            pframe = frame;
        }
        // Clear the background
        sx.wc.set_draw_color(entities::screen_color_bg_rel(dcolor, 0, 0));
        sx.wc.clear();
        sx.n_string(0.48, 0.01, &format!("{},{}",&pgentities.fps().round(), actualfps), sdlx::Color::BLUE);

        // handle any pending events
        for ev in sx.ep.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            use sdl2::keyboard::Mod;
            match ev {
                Event::Quit { timestamp: _ } => break 'mainloop,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod, repeat: _ } => {
                    match keycode.unwrap() {
                        Keycode::C => {
                            dcolor = dcolor.wrapping_add(20);
                        }
                        Keycode::P => {
                            bpause = !bpause;
                        }
                        Keycode::B => {
                            pgentities.showball = !pgentities.showball;
                        }
                        Keycode::Left => {
                            pdata.seek(-50);
                        }
                        Keycode::Right => {
                            pdata.seek(50);
                        }
                        Keycode::F => {
                            if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                                pgentities.fps_adjust(1.20);
                            } else {
                                pgentities.fps_adjust(0.80);
                            }
                            pdata.fps_changed(pgentities.fps());
                        }
                        Keycode::H => {
                            bhelp = !bhelp;
                        }
                        Keycode::D => {
                            eprintln!("DBUG:PPGND:Main:Entities:{:#?}", pgentities);
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
            if !pdata.bdone() {
                if cfg!(feature = "inbetween_frames") {
                    if pdata.next_frame_is_record_ready() {
                        let pu = pdata.next_record();
                        eprintln!("DBUG:{:?}", pu);
                        pgentities.update(pu, false, pdata.seconds_per_record() * pgentities.fps());
                        //eprintln!("DBUG:PPGND:Main:{}:Update called", _frame);
                    }
                    // TODO: Need to let this run for Fps frames ideally, even after bdone is set
                    // Or Rcg needs to be udpated to set bdone after a second of ending or so ...
                    pgentities.next_frame();
                    //eprintln!("DBUG:PPGND:Main:{}:NextFrame called", _frame);
                } else {
                    let pu = pdata.next_record();
                    pgentities.update(pu, true, 0.0);
                }
            }
        }

        // Draw entities
        pgentities.draw(&mut sx);
        if bhelp {
            show_help(&mut sx);
        }

        sx.wc.present();
        std::thread::sleep(std::time::Duration::from_millis((1000.0/pgentities.fps()).round() as u64));
    }

}
