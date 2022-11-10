//!
//! A entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, render::WindowCanvas, rect::Rect};
use sdl2::gfx::primitives::DrawRenderer;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;


pub struct Entity {
    id: String,
    pos: (i32, i32),
    color: Color,
    onscreen: bool,
}

impl Entity {

    pub fn new(id: &str, pos: (i32, i32), color: Color) -> Entity {
        Entity {
            id: id.to_string(),
            pos: pos,
            color: color,
            onscreen: true,
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
        wc.string(self.pos.0 as i16, self.pos.1 as i16, &self.id, Color::RGB(0, 0, 200)).unwrap();
    }

}