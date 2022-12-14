//!
//! Data related to play
//! HanishKVC, 2022
//!

use std::{collections::HashMap, fmt::Display};


type TPlayerId = String;

#[derive(Debug, PartialEq, Clone)]
pub enum Card {
    None,
    Red,
    Yellow,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let svariant = match self {
            Card::None => "None",
            Card::Red => "Red",
            Card::Yellow => "Yellow",
        };
        f.write_str(svariant)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    None,
    Kick(bool),
    Catch(bool),
    Tackle(bool),
    Others(usize),
}

#[derive(Debug)]
/// Player related data
pub enum PlayerData {
    /// Player position x,y
    Pos(f32,f32),
    Stamina(f32),
    Card(Card),
    Action(Action),
    /// Player Body and Neck direction, view angle width
    Dir(f32,f32,f32),
}

/// Maintain a vector of player related data
/// This allows only a subset of data to be available at any given time
pub type VPlayerData = Vec<PlayerData>;

pub type PlayerCodedData = (TPlayerId, VPlayerData);

#[derive(Debug)]
pub enum GameState {
    None,
    PlayPaused,
    PlayOn,
    Goal(char),
    Other(String),
}

pub type Messages = HashMap<String, String>;

#[derive(Debug)]
/// Maintain possible updates wrt a playdata update.
/// It can contain
/// * states like timecounter, game play state, ...
/// * messages if any, like score, time info, game actions, ...
/// * position of the ball
/// * positiono f the players.
/// * stamina of the players
pub struct PlayUpdate {
    pub timecounter: usize,
    pub state: GameState,
    pub msgs: Messages,
    pub ball: (f32, f32),
    pub lteamcoded: Vec<PlayerCodedData>,
    pub rteamcoded: Vec<PlayerCodedData>,
}

impl PlayUpdate {

    pub fn new() -> PlayUpdate {
        PlayUpdate {
            timecounter: 0,
            state: GameState::None,
            msgs: Messages::new(),
            ball: (0.0,0.0),
            lteamcoded: Vec::new(),
            rteamcoded: Vec::new(),
        }
    }

}

pub trait PlayData {

    /// Allows the playdata source to inform the main logic,
    /// has to how many seconds transpire between each record
    /// in the source.
    ///
    /// NOTE: The seconds between records can also be a
    /// fraction of a second.
    ///
    /// NOTE: This is global and cant change between each
    /// record currently.
    fn seconds_per_record(&self) -> f32;

    /// Informs the data playdata source about the current
    /// fps of the main playback gui logic.
    ///
    /// This allows the playdata source to respond properly
    /// to nextframe_isrecordready calls.
    fn fps_changed(&mut self, fps: f32);

    /// Each time the main gui logic is about to show a new
    /// frame, it asks the playdata source, if there is any
    /// new playdata record available to show wrt the new frame.
    ///
    /// NOTE: This is currently used, only if the main gui/prg
    /// is in interpolated movements mode. In which case, the
    /// main prg will request next_record, only if this returns
    /// true.
    fn next_frame_is_record_ready(&mut self) -> bool;

    /// Request the playdata source to send the next record,
    /// available with it (immidiately).
    fn next_record(&mut self) -> PlayUpdate;

    /// Request the playdata source to seek either forward or
    /// backward through its list of records.
    fn seek(&mut self, seekdelta: isize);

    /// Playdata source informs the main program, that there is
    /// no more data available with it.
    fn bdone(&self) -> bool;

    /// A generic send record, where the caller can pass a byte buffer.
    fn send_record(&mut self, buf: &[u8]);

    /// A generic send record, where the caller specifies a integer code.
    fn send_record_coded(&mut self, code: isize);

}

pub mod random;
pub mod rcss;
pub mod rcg;
pub mod rclive;