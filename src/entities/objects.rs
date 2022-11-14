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
/// Show a ball on the screen.
///
/// It uses GEntity internally, so it allows update to either specify
/// direct immidiate updating of the position of the ball on the screen OR
/// interpolated multi frame based updating of the ball to the provided position.
pub struct Ball<'a> {
    bge: Entity<'a>,
}

impl<'a> Ball<'a> {

    /// Create a new instance of the ball.
    /// NOTE: The position is set using update call.
    pub fn new(font: &'a Font) -> Ball<'a> {
        Ball {
            bge: Entity::new(" ", (0.0,0.0), (BALL_SIZE, BALL_SIZE, (BALL_SIZE/2) as i16), BALL_COLOR, font)
        }
    }

    /// Update the position of the ball on the screen.
    ///
    /// babsolute
    /// * if true, sets the ball to the given position immidiately,
    /// * if false, it uses multi-frame interpolated updating of the ball
    ///   to the given position.
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
/// Allow a Text message to be placed on the screen.
/// The position of the message is fixed when a new instance is create.
///
/// The message will remain on the screen till either a new message is
/// provided using update, or the optional autoclear kicks in.
pub struct FixedPosMessage {
    /// Key used to identify any new message in the hashmap of messages
    /// provided during update call.
    key: String,
    /// The position is specified in the normalised space of 0.0-1.0
    npos: (f32, f32),
    msg: String,
    color: Color,
    /// Control whether empty string is allowed to be set using update call.
    allowemptyupdate: bool,
    /// Control whether the message auto clears after a given count of frames
    /// If -ve, autoclear is disabled.
    autoclearchk: i32,
    /// Track the remaining frames wrt autoclear logic
    autoclearcnt: i32,
}

impl FixedPosMessage {

    /// Create a new instance of FixedPosMessage
    pub fn new(key: &str, npos: (f32,f32), ballowemptyupdate: bool, autoclearchk: i32) -> FixedPosMessage {
        FixedPosMessage {
            key: key.to_string(),
            npos: npos,
            msg: String::new(),
            color: MSG_COLOR,
            allowemptyupdate: ballowemptyupdate,
            autoclearchk: autoclearchk,
            autoclearcnt: autoclearchk,
        }
    }

    /// Directly set the message to be shown
    pub fn update_direct(&mut self, msg: &str) {
        if (msg.trim().len() == 0) && !self.allowemptyupdate {
            return;
        }
        self.msg = msg.to_string();
        self.autoclearcnt = self.autoclearchk;
    }

    /// Pass a hashmap of messages from which the message, if any,
    /// is picked up using the key setup during new.
    pub fn update(&mut self, msgs: &Messages) {
        let msg = msgs.get(&self.key);
        if msg.is_none() {
            return;
        }
        let msg = msg.unwrap();
        self.update_direct(msg);
    }

    /// Draw the message on the screen.
    ///
    /// If autoclear is set, then it keeps track of autoclear count
    /// and clears the message once the count is over.
    pub fn draw(&mut self, sx: &mut SdlX) {
        if self.autoclearchk > 0 {
            self.autoclearcnt -= 1;
            if self.autoclearcnt < 0 {
                self.msg = String::new();
            }
        }
        sx.n_string(self.npos.0, self.npos.1, &self.msg, self.color);
    }

}
