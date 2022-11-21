//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;
use std::time;

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

struct Gui {
    /// Whether help msgbox should be shown or not in the current frame
    showhelp: bool,
    /// Pause the playback
    pause: bool,
    /// Current frame number
    frame: usize,
    /// Time alloted per frame
    frametime: time::Duration,
    /// fps tracking: frame number wrt prev second
    fpsframe: usize,
    /// fps tracking: time wrt prev second
    fpstime: time::Instant,
    /// fps tracking: actually achieved fps
    actualfps: usize,
    /// the time at begining of processing wrt current frame
    curframetime: time::Instant,
}

impl Gui {

    fn new(fps: f32) -> Gui {
        let ctime = time::Instant::now();
        let mut gui = Gui {
            showhelp: false,
            pause: false,
            frame: 0,
            frametime: time::Duration::from_millis(100),
            fpsframe: 0,
            fpstime: ctime,
            actualfps: 0,
            curframetime: ctime,
        };
        gui.fps_changed(fps);
        return gui;
    }

    /// Update gui internal state, as needed, when fps requested by user/playdata source/... changes
    fn fps_changed(&mut self, fps: f32) {
        self.frametime = time::Duration::from_millis((1000.0/fps).round() as u64);
    }

    /// Update internal state, wrt/related-to begining of a new frame
    fn next_frame(&mut self) {
        self.frame += 1;
        self.curframetime = time::Instant::now();
        let dtime = self.curframetime.duration_since(self.fpstime);
        if dtime > time::Duration::from_millis(1000) {
            self.fpstime = self.curframetime;
            self.actualfps = self.frame - self.fpsframe;
            self.fpsframe = self.frame;
        }
    }

    /// Consume any frame time remaining wrt current frame, by sleeping
    fn consume_frametime(&mut self) {
        let ctime = time::Instant::now();
        let consumedtime = ctime.duration_since(self.curframetime);
        if self.frametime > consumedtime {
            let dtime = self.frametime - consumedtime;
            std::thread::sleep(dtime);
        }
    }

}

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
#[cfg(feature="inbetween_frames")]
fn sync_up_fps_to_spr(pgentities: &mut PGEntities, pdata: &mut dyn PlayData) {
    pdata.fps_changed(pgentities.fps());
    eprintln!("INFO:PPGND:Main:Fps:{}", pgentities.fps());
}

#[cfg(not(feature="inbetween_frames"))]
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
    let mut gui = Gui::new(entities::FRAMES_PER_SEC as f32);
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
    gui.fps_changed(pgentities.fps());

    // The main loop of the program starts now
    let mut skey = String::new();
    'mainloop: loop {
        gui.next_frame();
        // Clear the background
        sx.wc.set_draw_color(entities::screen_color_bg_rel(dcolor, 0, 0));
        sx.wc.clear();
        sx.n_msg(entities::MSG_FPS_POS.0, entities::MSG_FPS_POS.1, &format!("[{}] [{},{}]", skey, &pgentities.fps().round(), gui.actualfps), sdlx::Color::BLUE);

        // handle any pending program events
        let prgev= keys::get_programevents(&mut sx, &mut skey);
        match prgev {
            keys::ProgramEvent::None => (),
            keys::ProgramEvent::Pause => gui.pause = !gui.pause,
            keys::ProgramEvent::BackgroundColorChange => dcolor = dcolor.wrapping_add(20),
            keys::ProgramEvent::ToggleShowHelp => gui.showhelp = !gui.showhelp,
            keys::ProgramEvent::ToggleShowBall => pgentities.showball = !pgentities.showball,
            keys::ProgramEvent::ToggleShowActions => pgentities.toggle_bshowactions(),
            keys::ProgramEvent::ToggleShowStamina => pgentities.toggle_bstamina(),
            keys::ProgramEvent::SeekBackward => pdata.seek(-50),
            keys::ProgramEvent::SeekForward => pdata.seek(50),
            keys::ProgramEvent::AdjustFPS(ratio) => {
                pgentities.fps_adjust(ratio);
                pdata.fps_changed(pgentities.fps());
                gui.fps_changed(pgentities.fps());
            },
            keys::ProgramEvent::SendRecordCoded(code) => pdata.send_record_coded(code),
            keys::ProgramEvent::DumpPGEntities => eprintln!("DBUG:PPGND:Main:Entities:{:#?}", pgentities),
            keys::ProgramEvent::Quit => break 'mainloop,
            keys::ProgramEvent::NeedMore => (),
        }

        // Update the entities
        if !gui.pause {
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
        if gui.showhelp {
            show_help(&mut sx);
        }

        sx.wc.present();
        gui.consume_frametime();
    }

}
