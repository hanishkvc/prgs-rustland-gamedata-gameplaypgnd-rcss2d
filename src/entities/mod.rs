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

pub const BALL_SIZE: u32 = 6;
pub const BALL_COLOR: Color = Color::WHITE;


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


#[derive(Debug)]
pub(crate) struct Entities<'a> {
    sstep: String,
    ball: (f32,f32),
    pub showball: bool,
    ateam: team::Team<'a>,
    bteam: team::Team<'a>,
}

impl<'a> Entities<'a> {

    pub fn new(anplayers: i32, bnplayers: i32, font: &'a Font) -> Entities<'a> {
        Entities {
            sstep: String::new(),
            ball: (0.0,0.0),
            showball: true,
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
        }
    }

    pub fn update(&mut self, pu: PositionsUpdate, babsolute: bool) {
        self.sstep = pu.sstep;
        self.ball = pu.ball;
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

    fn draw_ball(&self, sx: &mut SdlX) {
        if !self.showball {
            return;
        }
        sx.wc.set_draw_color(BALL_COLOR);
        sx.ns_fill_rect(self.ball.0, self.ball.1, BALL_SIZE, BALL_SIZE);
    }

    pub fn draw(&self, sx: &mut SdlX) {
        sx.n_string(0.01, 0.01, &self.sstep, Color::RED);
        self.draw_pitch(sx);
        self.ateam.draw(sx);
        self.bteam.draw(sx);
        self.draw_ball(sx);
    }

}
