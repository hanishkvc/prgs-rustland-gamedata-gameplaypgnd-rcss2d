//!
//! SDL helpers
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, ttf::Font, surface::Surface};
use sdl2::render::{WindowCanvas, TextureCreator, Texture};
use sdl2::video::WindowContext;
use sdl2::pixels::Color;


pub struct SdlX {
    _ctxt: Sdl,
    _vs: VideoSubsystem,
    pub wc: WindowCanvas,
    pub ep: EventPump,
    pub wctc: TextureCreator<WindowContext>,
}

impl SdlX {

    pub fn init_plus(width: u32, height: u32) -> SdlX {
        let ctxt = sdl2::init().unwrap();
        // Setup window
        let vs = ctxt.video().unwrap();
        let win = vs.window("Playback", width, height).build().unwrap();
        let wc = win.into_canvas().build().unwrap();
        let wctc = wc.texture_creator();
        // Setup events
        let ep = ctxt.event_pump().unwrap();
        // Font related
        //sdl2::gfx::primitives::set_font(fontdata, cw, ch);
        SdlX {
            _ctxt: ctxt,
            _vs: vs,
            wc: wc,
            ep: ep,
            wctc: wctc,
        }
    }

}

impl SdlX {

    #[allow(dead_code)]
    pub fn text_texture(&self, text: &str, color: Color, font: &Font) -> Texture {
        let ts = font.render(text).blended(color).unwrap();
        let tt = ts.as_texture(&self.wctc).unwrap();
        return tt;
    }

}

pub fn text_surface<'a>(font: &'a Font, text: &str, color: Color) -> Surface<'a> {
    return font.render(text).blended(color).unwrap();
}
