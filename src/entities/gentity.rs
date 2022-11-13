//!
//! A graphical entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, rect::Rect};
use sdl2::ttf::Font;
use sdl2::surface::Surface;

use crate::sdlx::{self, SdlX};

use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::ENTITY_WIDTH;
use super::ENTITY_HEIGHT;


pub struct Entity<'a> {
    _id: String,
    /// Position of the entity in 0.0-1.0 space
    fpos: (f32, f32),
    color: Color,
    /// Should the entity be moved back into screen, if it goes out
    onscreen: bool,
    /// A cache of the Id string, as a SDL surface
    ids: Surface<'a>,
    /// Any motion vector that should be used to move entity,
    /// when next frame is called.
    mov: (f32, f32),
}

impl<'a> Entity<'a> {

    pub fn new(id: &str, fpos: (f32, f32), color: Color, font: &'a Font) -> Entity<'a> {
        let ts = sdlx::text_surface(font, id, Color::WHITE);
        Entity {
            _id: id.to_string(),
            fpos: fpos,
            color: color,
            onscreen: true,
            ids: ts,
            mov: (0.0, 0.0)
        }
    }

    fn fpos_fix(&mut self) {
        if self.onscreen {
            if self.fpos.0 < 0.0 {
                self.fpos.0 = 1.0;
            }
            if self.fpos.0 > 1.0 {
                self.fpos.0 = 0.0;
            }
            if self.fpos.1 < 0.0 {
                self.fpos.1 = 1.0;
            }
            if self.fpos.1 > 1.0 {
                self.fpos.1 = 0.0;
            }
        }
    }

    /// Convert the entity's position into screen space from normal space
    pub fn ipos(&self) -> (i32, i32) {
        ((self.fpos.0 * SCREEN_WIDTH as f32).round() as i32, (self.fpos.1 * SCREEN_HEIGHT as f32).round() as i32)
    }

    /// Set absolute position of the entity
    pub fn pos_set_abs(&mut self, fx: f32, fy: f32) {
        self.fpos = (fx, fy);
        self.fpos_fix();
    }

    /// Set relative position of the entity
    pub fn pos_set_rel(&mut self, fx: f32, fy: f32) {
        self.fpos = (self.fpos.0 + fx, self.fpos.1 + fy);
        self.fpos_fix();
    }

    pub fn move_to_in_frames(&mut self, fpos: (f32, f32), frames: f32) {
        let dx = (fpos.0 - self.fpos.0)/frames;
        let dy = (fpos.1 - self.fpos.1)/frames;
        self.mov = (dx, dy);
    }

    pub fn next_frame(&mut self) {
        self.pos_set_rel(self.mov.0, self.mov.1);
    }

    /// Draw the entity on passed canvas
    pub fn draw(&self, sx: &mut SdlX) {
        sx.wc.set_draw_color(self.color);
        let ipos = self.ipos();
        sx.wc.fill_rect(Rect::new(ipos.0, ipos.1, ENTITY_WIDTH, ENTITY_HEIGHT)).unwrap();
        //wc.string(self.pos.0 as i16, self.pos.1 as i16, &self.id, Color::RGB(0, 0, 200)).unwrap();
        let tx = self.ids.as_texture(&sx.wctc).unwrap();
        sx.wc.copy(&tx, None, Some(Rect::new(ipos.0, ipos.1, ENTITY_WIDTH, ENTITY_HEIGHT))).unwrap();
    }

}

impl std::fmt::Debug for Entity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("_id", &self._id)
            .field("pos", &self.fpos)
            .field("color", &self.color)
            .field("onscreen", &self.onscreen)
            .finish()
    }
}
