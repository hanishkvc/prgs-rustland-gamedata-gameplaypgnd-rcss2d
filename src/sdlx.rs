//!
//! SDL helpers
//! HanishKVC, 2022
//!

use sdl2::gfx::primitives::DrawRenderer;
use sdl2::rect::Rect;
use sdl2::{self, VideoSubsystem, Sdl, EventPump, ttf::Font, surface::Surface};
use sdl2::render::{WindowCanvas, TextureCreator, Texture, BlendMode};
use sdl2::video::WindowContext;
pub use sdl2::pixels::Color;


const STRING_CHAR_PIXEL_WIDTH: f32 = 8.0;


/// Initialises and maintains the SDL contexts wrt Video and Events.
pub struct SdlX {
    _ctxt: Sdl,
    _vs: VideoSubsystem,
    pub wc: WindowCanvas,
    pub ep: EventPump,
    pub wctc: TextureCreator<WindowContext>,
    n2s: XSpaces,
}

impl SdlX {

    /// Initialise SDL
    ///
    /// Initialise its Video subsystem and create a Window
    /// * Get a canvas and inturn the renderer wrt this window
    ///   * texture creator for creating textures for this window
    ///
    /// Get a event pump, to access input events
    ///
    /// Create a data space conversion mapper between
    /// * a normalised space (0.0-1.0) and
    /// * the screen space
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

/// Create a surface with a image of the passed text
pub fn text_surface<'a>(font: &'a Font, text: &str, color: Color) -> Surface<'a> {
    return font.render(text).blended(color).unwrap();
}


pub type XPoint = (f32,f32);
pub type XRect = (XPoint,XPoint);


/// Allow conversion between two different 2d spaces
pub struct XSpaces {
    /// The 1st/Data 2d space ((dx1,dy1), (dx2,dy2))
    drect: XRect,
    /// The 2nd/Other 2d space ((ox1,oy1), (ox2,oy2))
    orect: XRect,
    /// The conversion ratio from Data to Other
    d2o: XPoint,
    /// The conversion ratio from Other to Data
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

    /// Convert from Data to Other space along/wrt x-axis
    pub fn d2ox(&self, dx: f32) -> f32 {
        let ddx = dx - self.drect.0.0;
        let odx = ddx * self.d2o.0;
        return self.orect.0.0 + odx;
    }

    /// Convert from Data to Other space along/wrt y-axis
    pub fn d2oy(&self, dy: f32) -> f32 {
        let ddy = dy - self.drect.0.1;
        let ody = ddy * self.d2o.1;
        return self.orect.0.1 + ody;
    }

    /// Convert from Data to Other space wrt both x and y axis
    pub fn d2o(&self, d: XPoint) -> XPoint {
        return (self.d2ox(d.0), self.d2oy(d.1));
    }

}

#[allow(dead_code)]
impl XSpaces {

    /// Convert from Other to Data space along/wrt x-axis
    pub fn o2dx(&self, ox: f32) -> f32 {
        let odx = ox - self.orect.0.0;
        let ddx = odx * self.o2d.0;
        return self.drect.0.0 + ddx;
    }

    /// Convert from Other to Data space along/wrt y-axis
    pub fn o2dy(&self, oy: f32) -> f32 {
        let ody = oy - self.orect.0.1;
        let ddy = ody * self.o2d.1;
        return self.drect.0.1 + ddy;
    }

    /// Convert from Other to Data space wrt both x and y axis
    pub fn o2d(&self, o: XPoint) -> XPoint {
        return (self.o2dx(o.0), self.o2dy(o.1));
    }

}


impl SdlX {

    /// Draw a filled rect, which takes
    /// * x,y in normal space, and it represents the top-left of the rect
    /// * w,h in screen space
    pub fn ns_fill_rect(&mut self, nx: f32, ny: f32, sw: u32, sh: u32) {
        let sorigin = self.n2s.d2o((nx,ny));
        self.wc.fill_rect(Some(Rect::new(sorigin.0 as i32, sorigin.1 as i32, sw, sh))).unwrap();
    }

    /// Draw a filled rect, which takes
    /// * x,y in normal space, and it represents the top-left of the rect
    /// * w,h in normal space
    pub fn nn_fill_rect(&mut self, nx: f32, ny: f32, nw: f32, nh: f32) {
        let (sw, sh) = self.n2s.d2o((nw,nh));
        let sw = sw.round() as u32;
        let sh = sh.round() as u32;
        self.ns_fill_rect(nx, ny, sw, sh);
    }

    /// Draw a filled rect, which takes
    /// * x,y in normal space, and it represents the mid point of the rect
    /// * w,h in screen space
    pub fn ns_fill_rect_mid(&mut self, nx: f32, ny: f32, sw: u32, sh: u32) {
        let sorigin = self.n2s.d2o((nx,ny));
        let midw = (sw as f32)/2.0;
        let midh = (sh as f32)/2.0;
        let x = (sorigin.0 - midw).round() as i32;
        let y = (sorigin.1 - midh).round() as i32;
        self.wc.fill_rect(Some(Rect::new(x, y, sw, sh))).unwrap();
    }

    /// Draw a line, it takes the end points of the line in normal space
    pub fn nn_line(&mut self, nx1: f32, ny1: f32, nx2: f32, ny2: f32, color: Color) {
        let x1 = self.n2s.d2ox(nx1).round() as i16;
        let y1 = self.n2s.d2oy(ny1).round() as i16;
        let x2 = self.n2s.d2ox(nx2).round() as i16;
        let y2 = self.n2s.d2oy(ny2).round() as i16;
        self.wc.line(x1, y1, x2, y2, color).unwrap();
    }

    /// Draw a thick line. It takes the end points as well as the width wrt normal space
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

    /// Draw a string.
    /// Takes the starting point for drawing in normal space.
    pub fn n_string(&self, nx: f32, ny: f32, s: &str, color: Color) {
        let sx = self.n2s.d2ox(nx).round() as i16;
        let sy = self.n2s.d2oy(ny).round() as i16;
        self.wc.string(sx, sy, s, color).unwrap();
    }

    /// Draw/Show multiple lines on the screen.
    /// The starting point as well as the gap between lines is given in normal space.
    /// nlh: gives the height to be used wrt each line
    pub fn n_strings(&self, nx: f32, ny: f32, nlh: f32, ss: Vec<&str>, color: Color) {
        for i in 0..ss.len() {
            let y = ny + (i as f32 * nlh);
            self.n_string(nx, y, ss[i], color);
        }
    }

    /// Show a message box.
    /// * nr: (x,y,w,h) of the message box in normal space.
    /// * ss: a vector of strings
    ///   * 0th string will be treated has the heading and centered at the top.
    ///     The heading will be colored blackish grey, with white text.
    ///   * remaining strings will be treated has the message to show,
    ///     offset by 4 char space.
    /// * color: color of the message text shown. Background will be light grey.
    pub fn n_msgbox(&mut self, nr: (f32, f32, f32, f32), mut ss: Vec<&str>, color: Color) {
        let nlh = nr.3/((ss.len()+2) as f32);
        self.wc.set_blend_mode(BlendMode::Blend);
        self.wc.set_draw_color(Color::RGBA(200, 200, 200, 180));
        self.nn_fill_rect(nr.0, nr.1, nr.2, nr.3);
        // Heading rectangle
        self.wc.set_draw_color(Color::RGBA(80, 80, 80, 180));
        self.nn_fill_rect(nr.0, nr.1, nr.2, nlh*(2 as f32));
        // Heading text
        let ncw = self.n2s.o2dx(STRING_CHAR_PIXEL_WIDTH);
        let hlen = ss[0].len() as f32*ncw;
        let hbefore = (nr.3 - hlen)/2.0;
        self.n_string(nr.0+hbefore, nr.1+nlh, ss[0], Color::WHITE);
        // The message
        ss.remove(0);
        self.n_strings(nr.0+4.0*ncw, nr.1+3.0*nlh, nlh, ss, color);
    }

}

impl SdlX {

    /// NOTE: Remember if start and end angle are same (inc 0 to 360), then no arc
    pub fn ns_arc(&self, nx: f32, ny: f32, sradius: i16, sstartangle: i16, sendangle: i16, swidth: isize, color: Color) {
        let (sx,sy) = self.n2s.d2o((nx,ny));
        let sx = sx.round() as i16;
        let sy = sy.round() as i16;
        let wdiv = swidth/2;
        let wmod = swidth%2;
        let sw = sradius as isize - wdiv;
        let ew = sradius as isize + wdiv + wmod;
        for ir in sw..ew {
            self.wc.arc(sx, sy, ir as i16, sstartangle, sendangle, color).unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn n_arc(&self, nx: f32, ny: f32, nrad: f32, nstartangle: f32, nendangle: f32, width: isize, color: Color) {
        let radius = self.n2s.d2ox(nrad).round() as i16;
        let ssdeg = (nstartangle*360.0).round() as i16;
        let sedeg = (nendangle*360.0).round() as i16;
        self.ns_arc(nx, ny, radius, ssdeg, sedeg, width, color);
    }

}
