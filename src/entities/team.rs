//!
//! A team
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

use crate::entities;
use crate::entities::gentity::Entity;
use crate::sdlx::SdlX;



pub struct Team<'a> {
    name: String,
    color: Color,
    players: Vec<Entity<'a>>
}

impl<'a> Team<'a> {

    pub fn new(name: &str, color: Color, nplayers: i32, sx: &SdlX) -> Team<'a> {
        let team = Team {
            name: name.to_string(),
            color: color,
            players: Vec::new(),
        };
        for i in 0..nplayers {
            let iy: i32 = (rand::random::<u32>() % entities::SCREEN_HEIGHT) as i32;
            team.players.push(Entity::new(i.to_string().as_str(), (i*20i32, iy), team.color, sx));
        }
        team
    }

    pub fn draw(&self, swc: &mut WindowCanvas) {
        for i in 0..self.players.len() {
            self.players[i].draw(swc);
        }
    }

}