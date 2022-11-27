//!
//! A graphical entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, rect::Rect};
use sdl2::ttf::Font;
use sdl2::surface::Surface;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::BlendMode;

use crate::sdlx::{self, SdlX};

use sdlx::COLOR_INVISIBLE;



/// Represents a Graphical entity, with support
/// * for drawing itself, with any associated cached id text
/// * for being moved around explicitly or doing interpolated movement
///
/// Can be configured at compile time to either be a filled rect/circle.
pub struct GEntity<'a> {
    /// A textual id of the entity, the same is cached in a image form
    /// in the ids member.
    _id: String,
    /// Position of the entity in normal 0.0-1.0 space
    npos: (f32, f32),
    /// width, height in screen space dimensions
    width_height: (u32, u32),
    /// Radius in screen space dimensions
    radius: i16,
    /// Color of the object
    color: Color,
    /// Color adjust fraction for the adjustable part
    fcolor: f32,
    /// Color adjust selector
    pub colorsel: u8,
    /// Should the entity be moved back into screen, if it goes out
    onscreen: bool,
    /// A cache of the Id string, as a SDL surface
    ids: Surface<'a>,
    /// Any motion vector that should be used to move entity,
    /// when next frame is called.
    mov: (f32, f32),
    /// Internal member - half width
    hw: i32,
    /// Internal member - half height
    hh: i32,
    /// Extras - XArc
    /// XArc radius relative to GEntity size
    arc_nradius: f32,
    /// XArc angle in normalised space of 0.0-1.0 (ie wrt 0-360)
    arc_nangle: f32,
    arc_color: Color,
    tl_color: Color,
    bl_color: Color,
    ll_color: Color,
    rl_color: Color,
}

impl<'a> GEntity<'a> {

    /// Create a new instance of the Graphical Entity
    pub fn new(id: &str, npos: (f32, f32), width_height: (u32, u32), color: Color, font: &'a Font) -> GEntity<'a> {
        let ts = sdlx::text_surface(font, id, Color::WHITE);
        GEntity {
            _id: id.to_string(),
            npos: npos,
            width_height,
            radius: ((width_height.0 + width_height.1)/2) as i16,
            color: color,
            fcolor: -1.0,
            colorsel: 0x01,
            onscreen: true,
            ids: ts,
            mov: (0.0, 0.0),
            hw: (width_height.0/2) as i32,
            hh: (width_height.1/2) as i32,
            arc_nradius: -1.0,
            arc_nangle: -1.0,
            arc_color: Color::WHITE,
            tl_color: COLOR_INVISIBLE,
            bl_color: COLOR_INVISIBLE,
            ll_color: COLOR_INVISIBLE,
            rl_color: COLOR_INVISIBLE,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Ensure that the gentity remains within the 0.0-1.0 normal space,
    /// by wrapping it around to the other end, if required.
    ///
    /// NOTE: It only wraps around to the other end, any movement required
    /// within the other end, is not done.
    fn npos_fix(&mut self) {
        if self.onscreen {
            if self.npos.0 < 0.0 {
                self.npos.0 = 1.0;
            }
            if self.npos.0 > 1.0 {
                self.npos.0 = 0.0;
            }
            if self.npos.1 < 0.0 {
                self.npos.1 = 1.0;
            }
            if self.npos.1 > 1.0 {
                self.npos.1 = 0.0;
            }
        }
    }

    /// Convert the gentity's position into screen space from normal space
    pub fn ipos(&self) -> (i32, i32) {
        let (prgw, prgh) = sdlx::get_prg_resolution();
        ((self.npos.0 * prgw as f32).round() as i32, (self.npos.1 * prgh as f32).round() as i32)
    }

    /// Set absolute position of the gentity in normal 0.0-1.0 space
    pub fn pos_set_abs(&mut self, fx: f32, fy: f32) {
        self.npos = (fx, fy);
        self.npos_fix();
    }

    /// Set relative position of the gentity in normal 0.0-1.0 space
    pub fn pos_set_rel(&mut self, fx: f32, fy: f32) {
        self.npos = (self.npos.0 + fx, self.npos.1 + fy);
        self.npos_fix();
    }

    /// Set position of the gentity in normal 0.0-1.0 space, but to be
    /// applied/reached over a specified number of frames, as and when
    /// next_frame will be called, as required.
    ///
    /// NOTE: THis is for use in the interpolated movements mode.
    pub fn move_to_in_frames(&mut self, fpos: (f32, f32), frames: f32) {
        let dx = (fpos.0 - self.npos.0)/frames;
        let dy = (fpos.1 - self.npos.1)/frames;
        self.mov = (dx, dy);
    }

    /// Update the position of the gentity, wrt interpolated movement.
    /// It uses the move vector setup using move_to_in_frames call,
    /// to update the position.
    pub fn next_frame(&mut self) {
        self.pos_set_rel(self.mov.0, self.mov.1);
    }

    /// Draw the outer lines provided their colors is not invisible
    fn draw_outerlines(&self, sx: &mut SdlX) {
        let nw = sx.n2s.o2dx(self.width_height.0 as f32);
        let nh = sx.n2s.o2dy(self.width_height.1 as f32);
        let nhw = nw/2.0;
        let nhh = nh/2.0;
        let hlw = nh*0.2;
        let vlw = hlw*(nh/nw); // nw*0.2;
        //eprintln!("DBUG:PPGND:GEntity:DrawOuterLines:{}=>{},{}=>{},{}-{}",self.width_height.0, nw, self.width_height.1, nh, vlw, hlw);
        // Top line
        if self.tl_color != COLOR_INVISIBLE {
            let tx1 = self.npos.0 - nhw;
            let ty1 = self.npos.1 - nhh - nh*0.2;
            sx.nn_thick_line(tx1, ty1, tx1+nw, ty1, hlw, self.tl_color);
        }
        // Bottom line
        if self.bl_color != COLOR_INVISIBLE {
            let tx1 = self.npos.0 - nhw;
            let ty1 = self.npos.1 + nhh + nh*0.2;
            sx.nn_thick_line(tx1, ty1, tx1+nw, ty1, hlw, self.bl_color);
        }
        // left line
        if self.ll_color != COLOR_INVISIBLE {
            let lx1 = self.npos.0 - nhw - nw*0.2;
            let ly1 = self.npos.1 - nhh;
            sx.nn_thick_line(lx1, ly1, lx1, ly1+nh, vlw, self.ll_color);
        }
        // Right line
        if self.rl_color != COLOR_INVISIBLE {
            let lx1 = self.npos.0 + nhw + nw*0.2;
            let ly1 = self.npos.1 - nhh;
            sx.nn_thick_line(lx1, ly1, lx1, ly1+nh, vlw, self.rl_color);
        }
    }

    /// Draw the gentity on passed canvas
    /// At the core it consists of a
    /// * filled rectangle or a filled circle
    /// Further one can augument it with additional data using
    /// * a textual id
    /// * the fill color (which can be partly modified using fcolor)
    /// * a arc (wrt/including its radius, angle and color)
    /// * a set of outer lines and their colors
    pub fn draw(&self, sx: &mut SdlX) {
        let color;
        if self.fcolor < 0.0 {
            color = self.color;
        } else {
            color = self.color_adjust();
        }
        sx.wc.set_draw_color(color);
        sx.wc.set_blend_mode(BlendMode::Blend);
        let ipos = self.ipos();
        if cfg!(feature="gentity_circle") {
            sx.wc.filled_circle(ipos.0 as i16, ipos.1 as i16, self.radius, self.color).unwrap();
        } else {
            sx.ns_fill_rect_mid(self.npos.0, self.npos.1, self.width_height.0, self.width_height.1);
        }
        let tx = self.ids.as_texture(&sx.wctc).unwrap();
        sx.wc.copy(&tx, None, Some(Rect::new(ipos.0-self.hw, ipos.1-self.hh, self.width_height.0, self.width_height.1))).unwrap();
        if self.arc_nradius > 0.0 {
            let rad = (self.radius as f32 * self.arc_nradius).round() as i16;
            let edeg = (self.arc_nangle * 359.0).round() as i16;
            sx.ns_arc(self.npos.0, self.npos.1, rad, 0, edeg, 3, self.arc_color);
        }
        drop(tx);
        self.draw_outerlines(sx);
    }

}

impl std::fmt::Debug for GEntity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GEntity")
            .field("_id", &self._id)
            .field("pos", &self.npos)
            .field("whr", &self.width_height)
            .field("color", &self.color)
            .field("onscreen", &self.onscreen)
            .field("move", &self.mov)
            .finish()
    }
}

/// Helpers to manipulate the base color set
impl<'a> GEntity<'a> {

    /// color adjust fraction is set to fval1 * fval2
    pub fn set_fcolor(&mut self, fval1: f32, fval2: f32) {
        self.fcolor = fval1 * fval2;
    }

    /// Adjust the gentity's color, rather the adjustable part
    fn color_adjust(&self) -> Color {
        let (mut r, mut g, mut b, mut a) = self.color.rgba();
        if (self.colorsel & 0x08) == 0x08 {
            r = (((r as f32)*0.5) + (127.0*self.fcolor)).min(255.0) as u8;
        }
        if (self.colorsel & 0x04) == 0x04 {
            g = (((g as f32)*0.5) + (127.0*self.fcolor)).min(255.0) as u8;
        }
        if (self.colorsel & 0x02) == 0x02 {
            b = (((b as f32)*0.5) + (127.0*self.fcolor)).min(255.0) as u8;
        }
        if (self.colorsel & 0x01) == 0x01 {
            a = (((a as f32)*0.5) + (127.0*self.fcolor)).min(255.0) as u8;
        }
        return Color::RGBA(r, g, b, a);
    }

}

/// Helpers wrt Extra geometry
impl<'a> GEntity<'a> {

    /// Set the arc
    /// * nradius is relative to gentity size/radius
    /// * nangle is in 0.0-1.0 normal space (ie wrt 0-360 degrees).
    pub fn set_nxarc(&mut self, nradius: f32, nangle: f32, color: Color) {
        self.arc_nradius = nradius;
        self.arc_nangle = nangle;
        self.arc_color = color;
    }

    pub fn set_tl_color(&mut self, color: Color) {
        self.tl_color = color;
    }

    pub fn set_bl_color(&mut self, color: Color) {
        self.bl_color = color;
    }

    pub fn set_ll_color(&mut self, color: Color) {
        self.ll_color = color;
    }

    pub fn set_rl_color(&mut self, color: Color) {
        self.rl_color = color;
    }

}
