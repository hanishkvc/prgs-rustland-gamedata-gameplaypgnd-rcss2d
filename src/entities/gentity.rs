//!
//! A graphical entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, render::WindowCanvas, rect::Rect};
use sdl2::render::Texture;

use crate::sdlx::SdlX;

use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::ENTITY_WIDTH;
use super::ENTITY_HEIGHT;


pub struct Entity<'a> {
    _id: String,
    pos: (i32, i32),
    color: Color,
    onscreen: bool,
    idtx: Texture<'a>,
}

impl<'a> Entity<'a> {

    pub fn new(id: &str, pos: (i32, i32), color: Color, sx: &SdlX) -> Entity<'a> {
        let tt = sx.text_texture(id, Color::WHITE);
        Entity {
            _id: id.to_string(),
            pos: pos,
            color: color,
            onscreen: true,
            idtx: tt,
        }
    }

    /// Set absolute position of the entity
    pub fn pos_set_abs(&mut self, ix: i32, iy: i32) {
        self.pos = (ix, iy);
    }

    /// Set relative position of the entity
    pub fn pos_set_rel(&mut self, ix: i32, iy: i32) {
        self.pos = (self.pos.0 + ix, self.pos.1 + iy);

        if self.onscreen {
            if self.pos.0 < 0 {
                self.pos.0 = SCREEN_WIDTH as i32;
            }
            if self.pos.0 > (SCREEN_WIDTH as i32) {
                self.pos.0 = 0;
            }
            if self.pos.1 < 0 {
                self.pos.1 = SCREEN_HEIGHT as i32;
            }
            if self.pos.1 > (SCREEN_HEIGHT as i32) {
                self.pos.1 = 0;
            }
        }
    }

    /// Draw the entity on passed canvas
    pub fn draw(&self, wc: &mut WindowCanvas) {
        wc.set_draw_color(self.color);
        wc.fill_rect(Rect::new(self.pos.0, self.pos.1, ENTITY_WIDTH, ENTITY_HEIGHT)).unwrap();
        //wc.string(self.pos.0 as i16, self.pos.1 as i16, &self.id, Color::RGB(0, 0, 200)).unwrap();
        wc.copy(&self.idtx, None, Some(Rect::new(self.pos.0, self.pos.1, 16, 16))).unwrap();
    }

}
