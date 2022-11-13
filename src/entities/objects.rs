//!
//! The ball
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use crate::sdlx::SdlX;

pub const BALL_SIZE: u32 = 6;
pub const BALL_COLOR: Color = Color::WHITE;


#[derive(Debug)]
pub struct Ball {
    npos: (f32,f32),
    ssize: u32,
    color: Color,
}

impl Ball {

    pub fn new() -> Ball {
        Ball {
            npos: (0.0,0.0),
            ssize: BALL_SIZE,
            color: BALL_COLOR,
        }
    }

    pub fn update(&mut self, pos: (f32,f32)) {
        self.npos = pos;
    }

    pub fn draw(&self, sx: &mut SdlX) {
        sx.wc.set_draw_color(self.color);
        sx.ns_fill_rect(self.npos.0, self.npos.1, self.ssize, self.ssize);
    }

}