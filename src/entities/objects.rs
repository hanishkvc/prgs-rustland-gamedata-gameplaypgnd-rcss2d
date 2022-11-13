//!
//! The ball
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::SdlX;
use crate::playdata::Messages;
use crate::entities::FRAMES_PER_SEC;
use crate::entities::gentity::Entity;

pub const BALL_SIZE: u32 = 6;
pub const BALL_COLOR: Color = Color::WHITE;
pub const MSG_COLOR: Color = Color::RED;


#[derive(Debug)]
pub struct Ball<'a> {
    bge: Entity<'a>,
}

impl<'a> Ball<'a> {

    pub fn new(font: &'a Font) -> Ball<'a> {
        Ball {
            bge: Entity::new(" ", (0.0,0.0), (BALL_SIZE, BALL_SIZE, (BALL_SIZE/2) as i16), BALL_COLOR, font)
        }
    }

    pub fn update(&mut self, pos: (f32,f32), babsolute: bool) {
        let fx = pos.0;
        let fy = pos.1;
        if babsolute {
            self.bge.pos_set_abs(fx, fy);
        } else {
            self.bge.move_to_in_frames((fx, fy), FRAMES_PER_SEC as f32);
        }
    }

    pub fn next_frame(&mut self) {
        self.bge.next_frame();
    }

    pub fn draw(&self, sx: &mut SdlX) {
        self.bge.draw(sx);
    }

}


#[derive(Debug)]
pub struct FixedPosMessage {
    key: String,
    npos: (f32, f32),
    msg: String,
    color: Color,
    allowemptyupdate: bool,
}

impl FixedPosMessage {

    pub fn new(key: &str, npos: (f32,f32), ballowemptyupdate: bool) -> FixedPosMessage {
        FixedPosMessage {
            key: key.to_string(),
            npos: npos,
            msg: String::new(),
            color: MSG_COLOR,
            allowemptyupdate: ballowemptyupdate,
        }
    }

    pub fn update_direct(&mut self, msg: &str) {
        if (msg.trim().len() == 0) && !self.allowemptyupdate {
            return;
        }
        self.msg = msg.to_string();
    }

    pub fn update(&mut self, msgs: &Messages) {
        let msg = msgs.get(&self.key);
        if msg.is_none() {
            return;
        }
        let msg = msg.unwrap();
        self.update_direct(msg);
    }

    pub fn draw(&self, sx: &mut SdlX) {
        sx.n_string(self.npos.0, self.npos.1, &self.msg, self.color);
    }

}
