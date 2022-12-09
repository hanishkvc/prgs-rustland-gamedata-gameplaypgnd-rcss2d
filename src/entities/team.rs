//!
//! A team
//! HanishKVC, 2022
//!

use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use loggerk::{ldebug, log_d};

use crate::entities::{ENTITY_WIDTH, ENTITY_HEIGHT};
use crate::entities::gentity::{GEntity, GEDrawPrimitive};
use crate::proc::actions::{ActionsInfo, ActionData, AIAction};
use crate::sdlx::{SdlX, self, COLOR_INVISIBLE};
use crate::playdata::{PlayerCodedData, self, rcss};



#[derive(Debug)]
pub struct Team<'a> {
    name: String,
    color: Color,
    players: Vec<GEntity<'a>>,
    cards: HashMap<String, Vec<usize>>,
    bshowstamina: bool,
    bshowactions: bool,
    bshowcards: bool,
    bshowotheractions: bool,
}

impl<'a> Team<'a> {

    pub fn new(name: &str, color: Color, nplayers: i32, font: &'a Font) -> Team<'a> {
        let mut team = Team {
            name: name.to_string(),
            color: color,
            players: Vec::new(),
            cards: HashMap::new(),
            bshowstamina: true,
            bshowactions: true,
            bshowcards: true,
            bshowotheractions: false,
        };
        let (prgw, prgh) = sdlx::get_prg_resolution();
        let bx = (rand::random::<u32>() % prgw) as f32;
        for i in 0..nplayers {
            let fx = (rand::random::<u32>() % (prgw/4)) as f32;
            let fy = (rand::random::<u32>() % prgh) as f32;
            team.players.push(GEntity::new(i.to_string().as_str(), (bx+fx, fy), (ENTITY_WIDTH, ENTITY_HEIGHT), team.color, font));
        }
        team.cards.insert(playdata::Card::Red.to_string(), Vec::new());
        team.cards.insert(playdata::Card::Yellow.to_string(), Vec::new());
        ldebug!(&format!("INFO:PGND:Team:Created:{}:{:#?}\n", team.name, team));
        team
    }

    pub fn update(&mut self, timecounter: usize, playersdata: Vec<PlayerCodedData>, babsolute: bool, inframes: f32, actionsinfo: &mut ActionsInfo) {
        let side = self.name.chars().nth(0).unwrap();
        for player in playersdata {
            ldebug!(&format!("DBUG:PPGND:Team:{}:{:?}", self.name, player));
            let pi = player.0 as usize;
            let mut px = 0.0;
            let mut py = 0.0;
            let mut pact = AIAction::None;
            for pd in player.1 {
                match pd {
                    playdata::PlayerData::Pos(fx, fy) => {
                        px = fx;
                        py = fy;
                        // Position
                        if babsolute {
                            self.players[pi].pos_set_abs(fx, fy);
                        } else {
                            self.players[pi].move_to_in_frames((fx, fy), inframes);
                        }
                    },
                    playdata::PlayerData::Dir(body, neck, viewanglewidth) => {
                        // Body direction
                        let bstart = (body.round() as i16) - 10;
                        let bend = (body.round() as i16) + 10;
                        let arcangles = (bstart, bend);
                        self.players[pi].gextras_add(GEDrawPrimitive::NSArc{ remfc: 2, radratio: 1.2, arcangles, color: Color::WHITE});
                        // Body+Neck ie look direction
                        let mid = (body+neck).round() as i16;
                        let halfangle = (viewanglewidth.round() as i16)/2;
                        let start = mid - halfangle;
                        let end = mid + halfangle;
                        let arcangles = (start, end);
                        self.players[pi].gextras_add(GEDrawPrimitive::NSArc{ remfc: 2, radratio: 1.2, arcangles, color: Color::BLACK});
                    },
                    playdata::PlayerData::Stamina(fstamina) => {
                        // Stamina
                        //self.players[ppos.0 as usize].set_fcolor(1.0-fstamina, 1.0);
                        let istamina = (fstamina * 100.0).round() as i32;
                        let mut stamina_color = match istamina {
                            0..=30 => Color::RED,
                            31..=70 => Color::YELLOW,
                            71..=100 => Color::GREEN,
                            _ => todo!(),
                        };
                        if !self.bshowstamina {
                            stamina_color = COLOR_INVISIBLE;
                        }
                        //self.players[ppos.0 as usize].set_nxarc(0.8, fstamina, stamina_color);
                        self.players[pi].set_ll_color(stamina_color);
                        self.players[pi].set_rl_color(stamina_color);
                    },
                    playdata::PlayerData::Card(card) => {
                        // Cards
                        let penalised = self.cards.get_mut(&card.to_string());
                        if penalised.is_some() {
                            let penalised = penalised.unwrap();
                            if !penalised.contains(&pi) {
                                penalised.push(pi);
                                actionsinfo.handle_card(timecounter, side, pi, card.clone())
                            }
                        }
                        let mut card_color = sdlx::COLOR_INVISIBLE;
                        if let playdata::Card::Red = card {
                            card_color = Color::RED;
                        } else if let playdata::Card::Yellow = card {
                            card_color = Color::YELLOW;
                        }
                        if !self.bshowcards {
                            card_color = sdlx::COLOR_INVISIBLE;
                        }
                        self.players[pi].set_tl_color(card_color);
                        self.players[pi].set_bl_color(card_color);
                    },
                    playdata::PlayerData::Action(action) => {
                        let mut action_color = match action {
                            playdata::Action::Kick(good) => {
                                if good {
                                    pact = AIAction::Kick;
                                    Color::BLUE
                                } else {
                                    Color::GRAY
                                }
                            },
                            playdata::Action::Catch(good) => {
                                if good {
                                    pact = AIAction::Catch;
                                    Color::WHITE
                                } else {
                                    Color::GRAY
                                }
                            },
                            playdata::Action::Tackle(good) => {
                                if good {
                                    pact = AIAction::Tackle;
                                    Color::CYAN
                                } else {
                                    Color::GRAY
                                }
                            },
                            playdata::Action::Others(other_action) => {
                                let arcangles = if other_action == rcss::STATE_BALL2PLAYER as usize {
                                    (20,340)
                                } else if other_action == rcss::STATE_PLAYER2BALL as usize {
                                    (200,160)
                                } else {
                                    (340,20)
                                };
                                if self.bshowotheractions {
                                    self.players[pi].gextras_add(GEDrawPrimitive::NSArc{ remfc: 10, radratio: 1.4, arcangles, color: Color::BLACK});
                                }
                                COLOR_INVISIBLE
                            },
                            playdata::Action::None => COLOR_INVISIBLE,
                        };
                        if !self.bshowactions {
                            action_color = COLOR_INVISIBLE;
                        }
                        self.players[pi].set_nxarc(1.0, 0.98, action_color);
                    }
                }
            }
            actionsinfo.handle_action(ActionData::new(timecounter, side, pi, (px,py), pact));
        }
    }

    pub fn next_frame(&mut self) {
        for i in 0..self.players.len() {
            self.players[i].next_frame();
        }
    }

    pub fn draw(&mut self, sx: &mut SdlX) {
        for i in 0..self.players.len() {
            self.players[i].draw(sx);
        }
    }

}

impl<'a> Team<'a> {

    pub fn adjust_players(&mut self, colorsel: u8) {
        for i in 0..self.players.len() {
            self.players[i].colorsel = colorsel;
        }
    }

    pub fn toggle_bshowstamina(&mut self) -> bool {
        self.bshowstamina = !self.bshowstamina;
        self.bshowstamina
    }

    pub fn toggle_bshowactions(&mut self) -> bool {
        self.bshowactions = !self.bshowactions;
        self.bshowactions
    }

    pub fn toggle_bshowcards(&mut self) -> bool {
        self.bshowcards = !self.bshowcards;
        self.bshowcards
    }

}
