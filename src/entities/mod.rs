//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::{SdlX, XRect};
use crate::playdata::PositionsUpdate;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;
const ENTITY_RADIUS: i16 = 8;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);

pub const FRAMES_PER_SEC: usize = 24;

pub const PITCH_RECT: XRect = ((0.02,0.04), (0.98,0.96));

pub const MSG_SCORE_POS: (f32,f32) = (0.01,0.01);
pub const MSG_STIME_POS: (f32,f32) = (0.90,0.01);
pub const MSG_GAME_POS: (f32,f32) = (0.01,0.98);

pub fn screen_color_bg_rel(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: SCREEN_COLOR_BG.r+r,
        g: SCREEN_COLOR_BG.g+g,
        b: SCREEN_COLOR_BG.b+b,
        a: SCREEN_COLOR_BG.a,
    }
}

type _PosInt = i32;

pub mod gentity;
pub mod team;
pub mod objects;
use objects::Ball;
use objects::FixedPosMessage;


#[derive(Debug)]
pub(crate) struct Entities<'a> {
    vfpmsgs: Vec<FixedPosMessage>,
    pub showball: bool,
    ball: Ball,
    ateam: team::Team<'a>,
    bteam: team::Team<'a>,
    pitch: XRect,
    pub showxtrapitchmarkers: bool,
}

impl<'a> Entities<'a> {

    pub fn new(pitch: XRect, anplayers: i32, bnplayers: i32, font: &'a Font) -> Entities<'a> {
        let mut vfpmsgs = Vec::new();
        let scoremsg = FixedPosMessage::new("score", MSG_SCORE_POS, false);
        vfpmsgs.push(scoremsg);
        let stimemsg = FixedPosMessage::new("stime", MSG_STIME_POS, false);
        vfpmsgs.push(stimemsg);
        let gamemsg = FixedPosMessage::new("game", MSG_GAME_POS, false);
        vfpmsgs.push(gamemsg);
        Entities {
            vfpmsgs: vfpmsgs,
            ball: Ball::new(),
            showball: true,
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
            pitch: pitch,
            showxtrapitchmarkers: true,
        }
    }

    pub fn update(&mut self, pu: PositionsUpdate, babsolute: bool) {
        for fpmsg in &mut self.vfpmsgs {
            fpmsg.update(&pu.msgs);
        }
        self.ball.update(pu.ball);
        self.ateam.update(pu.ateampositions, babsolute);
        self.bteam.update(pu.bteampositions, babsolute);
    }

    pub fn next_frame(&mut self) {
        self.ateam.next_frame();
        self.bteam.next_frame();
    }

    fn draw_pitch(&self, sx: &mut SdlX) {
        //let inbtwcolor = Color::RGB(230, 230, 230);
        let inbtwcolor = Color::WHITE;
        let ((nx1,ny1),(nx2,ny2)) = self.pitch;
        sx.nn_thick_line(nx1, ny1, nx2, ny1, 0.002, Color::WHITE);
        sx.nn_thick_line(nx1, ny1, nx1, ny2, 0.004, Color::WHITE);
        sx.nn_thick_line(nx1, ny2, nx2, ny2, 0.002, Color::WHITE);
        sx.nn_thick_line(nx2, ny1, nx2, ny2, 0.004, Color::WHITE);
        sx.nn_line(0.50, 0.02, 0.50, 0.98, inbtwcolor); // Center vertical
        let leftmidx = nx1+0.02;
        sx.nn_line(leftmidx, 0.40, leftmidx, 0.60, inbtwcolor); // Left mid
        let rightmidx = nx2-0.02;
        sx.nn_line(rightmidx, 0.40, rightmidx, 0.60, inbtwcolor); // Right mid
        // Additional markers
        if self.showxtrapitchmarkers {
            sx.nn_line(0.25, 0.48, 0.25, 0.52, inbtwcolor); // Horizontal left quarter
            sx.nn_line(0.75, 0.48, 0.75, 0.52, inbtwcolor); // Horizontal right quarter
            sx.nn_line(0.48, 0.25, 0.52, 0.25, inbtwcolor); // Vertical top quarter
            sx.nn_line(0.48, 0.75, 0.52, 0.75, inbtwcolor); // Vertical bottom quarter
            sx.nn_line(0.49, 0.50, 0.51, 0.50, inbtwcolor); // Center horizontal
        }
    }

    pub fn draw(&self, sx: &mut SdlX) {
        self.draw_pitch(sx);
        for fpmsg in &self.vfpmsgs {
            fpmsg.draw(sx);
        }
        self.ateam.draw(sx);
        self.bteam.draw(sx);
        if self.showball {
            self.ball.draw(sx);
        }
    }

}
