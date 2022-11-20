//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use loggerk::{log_init, ldebug, log_d};

mod entities;
mod sdlx;
mod playdata;
use playdata::rcg::Rcg;
use playdata::random::RandomData;
use playdata::{PlayData, rclive};
use playdata::rclive::RCLive;
use sdlx::SdlX;
use entities::PGEntities;

mod testlib;
mod keys;

fn show_help(sx: &mut SdlX) {
    let shelp = "** Help **\n\
    \n\
    larrow: seek back\n\
    rarrow: seek forward\n\
    f/F:    change fps\n\
    p:      pause playback\n\
    ss:     show/hide stamina\n\
    sa:     show/hide actions\n\
    sb:     show/hide ball\n\
    c1:     RCLive kick-off\n\
    c0:     RCLive init hs\n\
    h:      hide/unhide help\n\
    \n\
    playbackpgnd <live [addr]> | <path/file.rcg>\n\
    ...                   Save Nature Save Earth";

    let vhelp: Vec<&str> = shelp.split('\n').collect();
    sx.n_msgbox((0.3,0.2, 0.4,0.6), vhelp, Color::BLUE);

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

///
/// Setup the playdata source based on passed args.
/// * if no args, then start the random playdata source
/// * if live passed as 1st arg to program, then try to
///   connect to a running rcssserver.
///   * if a 2nd argument is passed, use it has the nw
///     address of the server to connect to.
///   * else use the default address specified in rclive.
/// * else use the 1st argument as the rcg file to playback.
///
fn pdata_source(vargs: &Vec<String>, fps: f32) -> Box<dyn PlayData> {
    let src;
    if vargs.len() > 1 {
        src = vargs[1].as_str();
    } else {
        src = "";
    }
    if src == "live" {
        let nwaddr;
        if vargs.len() > 2 {
            nwaddr = vargs[2].as_str();
        } else {
            nwaddr = rclive::NWADDR_DEFAULT;
        }
        let pdrcl = RCLive::new(nwaddr);
        return Box::new(pdrcl);
    } else if src.len() > 0 {
        let pdrcg = Rcg::new(src, fps);
        return Box::new(pdrcg);
    } else {
        let pdrandom = RandomData::new(1.0/24.0, 11, 11);
        return Box::new(pdrandom);
    }
}

/// Sync up fps to the seconds per record of the playdata source
fn sync_up_fps_to_spr(pgentities: &mut PGEntities, pdata: &mut dyn PlayData) {
    let spr = pdata.seconds_per_record();
    let fpsadj = (1.0/spr)/pgentities.fps();
    pgentities.fps_adjust(fpsadj);
    pdata.fps_changed(1.0/spr);
    eprintln!("INFO:PPGND:Main:Fps:{}", pgentities.fps());
}

fn main() {
    log_init();
    identify();
    let ttfx = sdl2::ttf::init().unwrap();
    let font = ttfx.load_font(sdlx::TTF_FONT, 16);
    if font.is_err() {
        let err = font.err().unwrap();
        eprintln!("ERRR:PPGND:Loading font[{}], install it or update font in sdlx.rs:{}", sdlx::TTF_FONT, err);
        std::process::exit(10);
    }
    let font = font.unwrap();
    let mut sx = sdlx::SdlX::init_plus(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT);

    let mut dcolor = 20;
    let mut pgentities = entities::PGEntities::new(entities::PITCH_RECT, 11, 11, &font);
    pgentities.adjust_teams();

    // Setup the playdata source
    let clargs = env::args().collect::<Vec<String>>();
    let mut pdatasrc = pdata_source(&clargs, pgentities.fps());
    let pdata = pdatasrc.as_mut();

    // sync up fps to spr
    sync_up_fps_to_spr(&mut pgentities, pdata);

    // The main loop of the program starts now
    let mut bpause = false;
    let mut frame: usize = 0;
    let mut bhelp = false;
    let mut ptime = std::time::Instant::now();
    let mut pframe = 0;
    let mut actualfps = 0;
    let mut skey = String::new();
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
        sx.n_msg(entities::MSG_FPS_POS.0, entities::MSG_FPS_POS.1, &format!("[{}] [{},{}]", skey, &pgentities.fps().round(), actualfps), sdlx::Color::BLUE);

        // handle any pending program events
        let prgev= keys::get_programevents(&mut sx, &mut skey);
        match prgev {
            keys::ProgramEvent::None => (),
            keys::ProgramEvent::Pause => bpause = !bpause,
            keys::ProgramEvent::BackgroundColorChange => dcolor = dcolor.wrapping_add(20),
            keys::ProgramEvent::ToggleShowHelp => bhelp = !bhelp,
            keys::ProgramEvent::ToggleShowBall => pgentities.showball = !pgentities.showball,
            keys::ProgramEvent::ToggleShowActions => pgentities.toggle_bshowactions(),
            keys::ProgramEvent::ToggleShowStamina => pgentities.toggle_bstamina(),
            keys::ProgramEvent::SeekBackward => pdata.seek(-50),
            keys::ProgramEvent::SeekForward => pdata.seek(50),
            keys::ProgramEvent::AdjustFPS(ratio) => {
                pgentities.fps_adjust(ratio);
                pdata.fps_changed(pgentities.fps());
            },
            keys::ProgramEvent::SendRecordCoded(code) => pdata.send_record_coded(code),
            keys::ProgramEvent::DumpPGEntities => eprintln!("DBUG:PPGND:Main:Entities:{:#?}", pgentities),
            keys::ProgramEvent::Quit => break 'mainloop,
            keys::ProgramEvent::NeedMore => (),
        }

        // Update the entities
        if !bpause {
            if !pdata.bdone() {
                if cfg!(feature = "inbetween_frames") {
                    if pdata.next_frame_is_record_ready() {
                        let pu = pdata.next_record();
                        ldebug!(&format!("DBUG:{:?}", pu));
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
