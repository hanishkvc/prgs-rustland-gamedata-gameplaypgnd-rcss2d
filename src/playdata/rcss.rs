//!
//! Robocup related common stuff
//! HanishKVC, 2022
//!

use super::{Card, Action};

/// This time is infered from live record reception,
/// Later need to check docs/src of rcss to check, if it can change
pub const SECONDS_PER_RECORD: f32 = 0.1;
pub const STAMINA_BASE: f32 = 8000.0;

/// Player states
pub const STATE_KICK: u32           = 0x00002;
pub const STATE_KICK_FAULT: u32     = 0x00004;
pub const STATE_CATCH: u32          = 0x00010;
pub const STATE_CATCH_FAULT: u32    = 0x00020;
pub const STATE_TACKLE: u32         = 0x01000;
pub const STATE_TACKLE_FAULT: u32   = 0x02000;
pub const STATE_REDCARD: u32        = 0x80000;
pub const STATE_YELLOWCARD: u32     = 0x40000;


pub fn handle_state(state: u32) -> (Action, Card) {
    let mut action = Action::None;
    let mut card = Card::None;
    if state & STATE_REDCARD == STATE_REDCARD {
        card = Card::Red;
    } else if state & STATE_YELLOWCARD == STATE_YELLOWCARD {
        card = Card::Yellow;
    } else if state & STATE_KICK == STATE_KICK {
        action = Action::Kick(true);
    } else if state & STATE_KICK_FAULT == STATE_KICK_FAULT {
        action = Action::Kick(false);
    } else if state & STATE_CATCH == STATE_CATCH {
        action = Action::Catch(true);
    } else if state & STATE_CATCH_FAULT == STATE_CATCH_FAULT {
        action = Action::Catch(false);
    } else if state & STATE_TACKLE == STATE_TACKLE {
        action = Action::Tackle(true);
    } else if state & STATE_TACKLE_FAULT == STATE_TACKLE_FAULT {
        action = Action::Tackle(false);
    }
    return (action, card);
}
