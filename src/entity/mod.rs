//!
//! A entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, render::WindowCanvas, rect::Rect};


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;


pub struct Entity {
    pos: (i32, i32),
    color: Color,
}

impl Entity {

    pub fn new(pos: (i32, i32), color: Color) -> Entity {
        Entity {
            pos: pos,
            color: color,
        }
    }

    pub fn pos_set_abs(&mut self, ix: i32, iy: i32) {
        self.pos = (ix, iy);
    }

    pub fn pos_set_rel(&mut self, ix: i32, iy: i32) {
        self.pos = (self.pos.0 + ix, self.pos.1 + iy);
        if self.pos.0 > (SCREEN_WIDTH as i32) {
            self.pos.0 = 0;
        }
        if self.pos.1 > (SCREEN_HEIGHT as i32) {
            self.pos.1 = 0;
        }
    }

    pub fn draw(&self, wc: &mut WindowCanvas) {
        wc.set_draw_color(self.color);
        wc.fill_rect(Rect::new(self.pos.0, self.pos.1, ENTITY_WIDTH, ENTITY_HEIGHT)).unwrap();
    }

}