//!
//! SDL helpers
//! HanishKVC, 2022
//!

use sdl2::gfx::primitives::DrawRenderer;
use sdl2::rect::Rect;
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
    n2s: XSpaces,
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
        // Normal to Screen space
        let drect = ((0.0,0.0), (1.0,1.0));
        let orect = ((0.0,0.0), (width as f32, height as f32));
        SdlX {
            _ctxt: ctxt,
            _vs: vs,
            wc: wc,
            ep: ep,
            wctc: wctc,
            n2s: XSpaces::new(drect, orect),
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


pub type XPoint = (f32,f32);
pub type XRect = (XPoint,XPoint);

pub struct XSpaces {
    drect: XRect,
    orect: XRect,
    d2o: XPoint,
    o2d: XPoint,
}

impl XSpaces {

    pub fn new(drect: XRect, orect: XRect) -> XSpaces {
        let ddx = drect.1.0 - drect.0.0;
        let odx = orect.1.0 - orect.0.0;
        let d2ox = odx/ddx;
        let o2dx = ddx/odx;
        let ddy = drect.1.1 - drect.0.1;
        let ody = orect.1.1 - orect.0.1;
        let d2oy = ody/ddy;
        let o2dy = ddy/ody;
        XSpaces {
            drect: drect,
            orect: orect,
            d2o: (d2ox, d2oy),
            o2d: (o2dx, o2dy),
        }
    }

}

impl XSpaces {

    pub fn d2ox(&self, dx: f32) -> f32 {
        let ddx = dx - self.drect.0.0;
        let odx = ddx * self.d2o.0;
        return self.orect.0.0 + odx;
    }

    pub fn d2oy(&self, dy: f32) -> f32 {
        let ddy = dy - self.drect.0.1;
        let ody = ddy * self.d2o.1;
        return self.orect.0.1 + ody;
    }

    pub fn d2o(&self, d: XPoint) -> XPoint {
        return (self.d2ox(d.0), self.d2oy(d.1));
    }

}

#[allow(dead_code)]
impl XSpaces {

    pub fn o2dx(&self, ox: f32) -> f32 {
        let odx = ox - self.orect.0.0;
        let ddx = odx * self.o2d.0;
        return self.drect.0.0 + ddx;
    }

    pub fn o2dy(&self, oy: f32) -> f32 {
        let ody = oy - self.orect.0.1;
        let ddy = ody * self.o2d.1;
        return self.drect.0.1 + ddy;
    }

    pub fn o2d(&self, o: XPoint) -> XPoint {
        return (self.o2dx(o.0), self.o2dy(o.1));
    }

}


impl SdlX {

    pub fn ns_fill_rect(&mut self, nx: f32, ny: f32, sw: u32, sh: u32) {
        let sorigin = self.n2s.d2o((nx,ny));
        self.wc.fill_rect(Some(Rect::new(sorigin.0 as i32, sorigin.1 as i32, sw, sh))).unwrap();
    }

    pub fn nn_line(&mut self, nx1: f32, ny1: f32, nx2: f32, ny2: f32, color: Color) {
        let x1 = self.n2s.d2ox(nx1).round() as i16;
        let y1 = self.n2s.d2oy(ny1).round() as i16;
        let x2 = self.n2s.d2ox(nx2).round() as i16;
        let y2 = self.n2s.d2oy(ny2).round() as i16;
        self.wc.line(x1, y1, x2, y2, color).unwrap();
    }

    pub fn nn_thick_line(&mut self, nx1: f32, ny1: f32, nx2: f32, ny2: f32, nw: f32, color: Color) {
        let x1 = self.n2s.d2ox(nx1).round() as i16;
        let y1 = self.n2s.d2oy(ny1).round() as i16;
        let x2 = self.n2s.d2ox(nx2).round() as i16;
        let y2 = self.n2s.d2oy(ny2).round() as i16;
        let sw;
        if (x2-x1).abs() > (y2-y1).abs() {
            sw = self.n2s.d2ox(nw).round() as u8;
        } else {
            sw = self.n2s.d2oy(nw).round() as u8;
        }
        self.wc.thick_line(x1, y1, x2, y2, sw, color).unwrap();
    }

    pub fn n_string(&self, nx: f32, ny: f32, s: &str, color: Color) {
        let sx = self.n2s.d2ox(nx).round() as i16;
        let sy = self.n2s.d2oy(ny).round() as i16;
        self.wc.string(sx, sy, s, color).unwrap();
    }

}
