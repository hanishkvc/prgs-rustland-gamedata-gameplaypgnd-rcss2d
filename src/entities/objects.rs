//!
//! The ball
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use crate::sdlx::SdlX;
use crate::playdata::Messages;

pub const BALL_SIZE: u32 = 6;
pub const BALL_COLOR: Color = Color::WHITE;
pub const MSG_COLOR: Color = Color::RED;


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


#[derive(Debug)]
pub struct FixedMessage {
    key: String,
    npos: (f32, f32),
    msg: String,
    color: Color,
    allowemptyupdate: bool,
}

impl FixedMessage {

    pub fn new(key: &str, npos: (f32,f32), ballowemptyupdate: bool) -> FixedMessage {
        FixedMessage {
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
