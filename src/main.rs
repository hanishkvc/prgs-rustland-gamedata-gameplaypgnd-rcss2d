//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use loggerk::log_init;

mod entities;
mod sdlx;
mod playdata;
use playdata::rcg::Rcg;
use playdata::random::RandomData;
use playdata::PlayData;
use playdata::rclive::RCLive;
use sdlx::SdlX;

mod testlib;

fn show_help(sx: &mut SdlX) {
    let shelp = "** Help **\n\
    \n\
    larrow: seek back\n\
    rarrow: seek forward\n\
    f/F:    change fps\n\
    p:      pause playback\n\
    s:      hide/unhide stamina\n\
    b:      hide/unhide ball\n\
    h:      hide/unhide help\n\
    1:      kick-off (RCLive)\n\
    \n\
    playbackpgnd live|path/file.rcg\n\
    ...                   Save Nature Save Earth";

    let vhelp: Vec<&str> = shelp.split('\n').collect();
    sx.n_msgbox((0.3, 0.3,0.4,0.4), vhelp, Color::BLUE);

}

#[allow(dead_code)]
fn test_me(font: &Font) {
    testlib::test_ncolor();
    testlib::test_gentity(font);
}

fn identify() {
    println!("Playback Playground");
    if cfg!(feature = "inbetween_frames") {
        println!("INFO:PPGND:Mode: InBetween Frames");
    } else {
        println!("INFO:PPGND:Mode: OnlyProvided Frames");
    }
}

fn pdata_source(vargs: &Vec<String>, fps: f32) -> Box<dyn PlayData> {
    let src;
    if vargs.len() > 1 {
        src = vargs[1].as_str();
    } else {
        src = "";
    }
    if src == "live" {
        let pdrcl = RCLive::new("0.0.0.0:6000");
        return Box::new(pdrcl);
    } else if src.len() > 0 {
        let pdrcg = Rcg::new(src, fps);
        return Box::new(pdrcg);
    } else {
        let pdrandom = RandomData::new(20.0, 11, 11);
        return Box::new(pdrandom);
    }
}

fn main() {
    log_init();
    identify();
    let ttfx = sdl2::ttf::init().unwrap();
    let font = ttfx.load_font("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 16).unwrap();
    let mut sx = sdlx::SdlX::init_plus(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT);

    let mut dcolor = 20;
    let mut pgentities = entities::PGEntities::new(entities::PITCH_RECT, 11, 11, &font);
    pgentities.adjust_teams();

    // Setup the playdata source
    let clargs = env::args().collect::<Vec<String>>();
    let mut pdatasrc = pdata_source(&clargs, pgentities.fps());
    let pdata = pdatasrc.as_mut();

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
        sx.n_msg(0.48, 0.01, &format!("{},{}",&pgentities.fps().round(), actualfps), sdlx::Color::BLUE);

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
                        Keycode::S => {
                            pgentities.toggle_bstamina();
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
                        Keycode::Num1 => {
                            pdata.send_record_coded(1);
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
