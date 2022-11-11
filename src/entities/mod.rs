//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::SdlX;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);


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
pub struct TeamUpdates {
    pub ateampositions: Vec<(i32, f32, f32)>,
    pub bteampositions: Vec<(i32, f32, f32)>,
}

impl TeamUpdates {

    pub fn new() -> TeamUpdates {
        TeamUpdates { ateampositions: Vec::new(), bteampositions: Vec::new() }
    }

}


#[derive(Debug)]
pub(crate) struct Entities<'a> {
    ateam: team::Team<'a>,
    bteam: team::Team<'a>,
}

impl<'a> Entities<'a> {

    pub fn new(anplayers: i32, bnplayers: i32, font: &'a Font) -> Entities<'a> {
        Entities {
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
        }
    }

    pub fn update_dummy(&mut self, step: usize) {
        self.ateam.update_dummy(step);
        self.bteam.update_dummy(step);
    }

    pub fn update(&mut self, tu: TeamUpdates) {
        self.ateam.update(tu.ateampositions);
        self.bteam.update(tu.bteampositions);
    }

    pub fn draw(&self, sx: &mut SdlX) {
        self.ateam.draw(sx);
        self.bteam.draw(sx);
    }

}

