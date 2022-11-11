//!
//! A team
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::entities;
use crate::entities::gentity::Entity;
use crate::sdlx::SdlX;



#[derive(Debug)]
pub struct Team<'a> {
    name: String,
    color: Color,
    players: Vec<Entity<'a>>
}

impl<'a> Team<'a> {

    pub fn new(name: &str, color: Color, nplayers: i32, font: &'a Font) -> Team<'a> {
        let mut team = Team {
            name: name.to_string(),
            color: color,
            players: Vec::new(),
        };
        for i in 0..nplayers {
            let iy: i32 = (rand::random::<u32>() % entities::SCREEN_HEIGHT) as i32;
            team.players.push(Entity::new(i.to_string().as_str(), (i*20i32, iy), team.color, font));
        }
        print!("INFO:PGND:Team:Created:{}:{:?}", team.name, team);
        team
    }

    pub fn update(&mut self) {
        for i in 0..self.players.len() {
            let dy: i32 = (rand::random::<i32>() % 4) as i32;
            self.players[i].pos_set_rel(1, dy);
        }
    }

    pub fn draw(&self, sx: &mut SdlX) {
        for i in 0..self.players.len() {
            self.players[i].draw(sx);
        }
    }

}