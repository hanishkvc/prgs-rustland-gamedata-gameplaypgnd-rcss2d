//!
//! Data related to play
//! HanishKVC, 2022
//!

use std::collections::HashMap;


pub type Messages = HashMap<String, String>;

#[derive(Debug)]
pub struct PositionsUpdate {
    pub msgs: Messages,
    pub ball: (f32, f32),
    pub ateampositions: Vec<(i32, f32, f32)>,
    pub bteampositions: Vec<(i32, f32, f32)>,
}

impl PositionsUpdate {

    pub fn new() -> PositionsUpdate {
        PositionsUpdate {
            msgs: Messages::new(),
            ball: (0.0,0.0),
            ateampositions: Vec::new(),
            bteampositions: Vec::new(),
        }
    }

}

pub trait PlayData {

    fn setup(&mut self, fps: f32);

    fn next_frame_is_record_ready(&mut self) -> bool;

    fn next_record(&mut self) -> PositionsUpdate;

    fn seek(&mut self, seekdelta: isize);

    fn bdone(&self) -> bool;

}

pub mod random;
pub mod rcg;
