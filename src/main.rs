//!
//! 2D Playback of a captured game data
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, video::Window, EventPump, render::WindowCanvas};

fn sdl_init() -> (Sdl, VideoSubsystem, WindowCanvas, EventPump) {
    let sctxt = sdl2::init().unwrap();
    let sv = sctxt.video().unwrap();
    let sw = sv.window("Playback", 800, 600).build().unwrap();
    let swc = sw.into_canvas().build().unwrap();
    let se = sctxt.event_pump().unwrap();
    return (sctxt, sv, swc, se);
}

fn main() {
    println!("Hello, world!");
    let (sctxt, sv, swc, se) = sdl_init();
    
}
