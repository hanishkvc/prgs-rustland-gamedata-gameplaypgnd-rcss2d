//!
//! SDL helpers
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, render::WindowCanvas, pixels::Color, ttf::{self, Font}};
use sdl2::render::{TextureCreator};
use sdl2::video::WindowContext;


pub fn sdl_init(width: u32, height: u32) -> (Sdl, VideoSubsystem, WindowCanvas, EventPump) {
    let sctxt = sdl2::init().unwrap();
    let sv = sctxt.video().unwrap();
    let sw = sv.window("Playback", width, height).build().unwrap();
    let swc = sw.into_canvas().build().unwrap();
    let se = sctxt.event_pump().unwrap();
    //sdl2::gfx::primitives::set_font(fontdata, cw, ch);
    return (sctxt, sv, swc, se);
}

pub fn font_init() {
    let stx = ttf::init().unwrap();
    let font = stx.load_font("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 16).unwrap();
    let swctc = swc.texture_creator();

}
