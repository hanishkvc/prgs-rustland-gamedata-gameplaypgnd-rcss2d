//!
//! SDL helpers
//! HanishKVC, 2022
//!

use sdl2::{self, VideoSubsystem, Sdl, EventPump, ttf::{self, Font, Sdl2TtfContext}};
use sdl2::render::{WindowCanvas, TextureCreator, Texture};
use sdl2::video::WindowContext;
use sdl2::pixels::Color;


pub struct SdlX<'a> {
    ctxt: Sdl,
    vs: VideoSubsystem,
    pub wc: WindowCanvas,
    pub ep: EventPump,
    wctc: TextureCreator<WindowContext>,
    pub font: Font<'a,'a>,
}

impl<'a> SdlX<'a> {

    pub fn init_plus(width: u32, height: u32, font: Font<'a,'a>) -> SdlX<'a> {
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
            ctxt: ctxt,
            vs: vs,
            wc: wc,
            ep: ep,
            wctc: wctc,
            font: font,
        }
    }

}

impl<'a> SdlX<'a> {

    pub fn text_texture(&self, text: &str, color: Color) -> Texture {
        let ts = self.font.render(text).blended(color).unwrap();
        let tt = ts.as_texture(&self.wctc).unwrap();
        return tt;
    }

}
