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

use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;


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
    /// width, height and radius in screen space dimensions
    whr: (u32, u32, i16),
    color: Color,
    /// Color adjust
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
}

impl<'a> GEntity<'a> {

    /// Create a new instance of the Graphical Entity
    pub fn new(id: &str, npos: (f32, f32), whr: (u32, u32, i16), color: Color, font: &'a Font) -> GEntity<'a> {
        let ts = sdlx::text_surface(font, id, Color::WHITE);
        GEntity {
            _id: id.to_string(),
            npos,
            whr: whr,
            color: color,
            fcolor: 1.0,
            colorsel: 0x01,
            onscreen: true,
            ids: ts,
            mov: (0.0, 0.0),
            hw: (whr.0/2) as i32,
            hh: (whr.1/2) as i32,
        }
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
        ((self.npos.0 * SCREEN_WIDTH as f32).round() as i32, (self.npos.1 * SCREEN_HEIGHT as f32).round() as i32)
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

    pub fn set_fcolor(&mut self, fval1: f32, fval2: f32) {
        self.fcolor = fval1 * fval2;
    }

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

    /// Draw the gentity on passed canvas
    pub fn draw(&self, sx: &mut SdlX) {
        sx.wc.set_draw_color(self.color_adjust());
        sx.wc.set_blend_mode(BlendMode::Blend);
        let ipos = self.ipos();
        if cfg!(feature="gentity_circle") {
            sx.wc.filled_circle(ipos.0 as i16, ipos.1 as i16, self.whr.2, self.color).unwrap();
        } else {
            sx.ns_fill_rect_mid(self.npos.0, self.npos.1, self.whr.0, self.whr.1);
        }
        let tx = self.ids.as_texture(&sx.wctc).unwrap();
        sx.wc.copy(&tx, None, Some(Rect::new(ipos.0-self.hw, ipos.1-self.hh, self.whr.0, self.whr.1))).unwrap();
    }

}

impl std::fmt::Debug for GEntity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GEntity")
            .field("_id", &self._id)
            .field("pos", &self.npos)
            .field("whr", &self.whr)
            .field("color", &self.color)
            .field("onscreen", &self.onscreen)
            .field("move", &self.mov)
            .finish()
    }
}
