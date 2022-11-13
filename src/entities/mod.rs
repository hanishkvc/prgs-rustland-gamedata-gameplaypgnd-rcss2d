//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::SdlX;
use crate::playdata::PositionsUpdate;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);

pub const FRAMES_PER_SEC: usize = 24;

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
use objects::FixedMessage;


#[derive(Debug)]
pub(crate) struct Entities<'a> {
    scoremsg: FixedMessage,
    stimemsg: FixedMessage,
    gamemsg: FixedMessage,
    pub showball: bool,
    ball: Ball,
    ateam: team::Team<'a>,
    bteam: team::Team<'a>,
}

impl<'a> Entities<'a> {

    pub fn new(anplayers: i32, bnplayers: i32, font: &'a Font) -> Entities<'a> {
        Entities {
            scoremsg: FixedMessage::new("score", MSG_SCORE_POS, false),
            stimemsg: FixedMessage::new("stime", MSG_STIME_POS, false),
            gamemsg: FixedMessage::new("game", MSG_GAME_POS, false),
            ball: Ball::new(),
            showball: true,
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
        }
    }

    pub fn update(&mut self, pu: PositionsUpdate, babsolute: bool) {
        self.scoremsg.update(&pu.msgs);
        self.stimemsg.update(&pu.msgs);
        self.gamemsg.update(&pu.msgs);
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
        sx.nn_thick_line(0.02, 0.04, 0.98, 0.04, 0.002, Color::WHITE);
        sx.nn_thick_line(0.02, 0.04, 0.02, 0.96, 0.004, Color::WHITE);
        sx.nn_thick_line(0.02, 0.96, 0.98, 0.96, 0.002, Color::WHITE);
        sx.nn_thick_line(0.98, 0.04, 0.98, 0.96, 0.004, Color::WHITE);
        sx.nn_line(0.50, 0.02, 0.50, 0.98, inbtwcolor);
        sx.nn_line(0.04, 0.40, 0.04, 0.60, inbtwcolor);
        sx.nn_line(0.96, 0.40, 0.96, 0.60, inbtwcolor);
    }

    pub fn draw(&self, sx: &mut SdlX) {
        self.draw_pitch(sx);
        self.scoremsg.draw(sx);
        self.stimemsg.draw(sx);
        self.gamemsg.draw(sx);
        self.ateam.draw(sx);
        self.bteam.draw(sx);
        if self.showball {
            self.ball.draw(sx);
        }
    }

}
