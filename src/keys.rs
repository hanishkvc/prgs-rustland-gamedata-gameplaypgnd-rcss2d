//!
//! Key shortcuts
//! HanishKVC, 2022
//!

use sdl2::keyboard::{Keycode, Mod};

use loggerk::{ldebug, log_d};

use crate::{sdlx::SdlX, proc::actions};

pub enum ProgramEvent {
    None,
    Pause,
    BackgroundColorChange,
    ToggleShowHelp,
    ToggleShowActions,
    ToggleShowBall,
    ToggleShowStamina,
    ToggleShowCards,
    SeekBackward,
    SeekForward,
    AdjustFPS(f32),
    SendRecordCoded(isize),
    DumpPGEntities,
    DumpAIScoresSummary(char),
    DumpAIDistancesSummary(char),
    DumpIncCardScore,
    DumpAITimeVsScoreSummary(char),
    Quit,
    NeedMore,
}

fn handle_s_cmds(keycode: Keycode) -> ProgramEvent {
    match keycode {
        Keycode::A => {
            return ProgramEvent::ToggleShowActions;
        },
        Keycode::B => {
            return ProgramEvent::ToggleShowBall;
        },
        Keycode::S => {
            return ProgramEvent::ToggleShowStamina;
        },
        Keycode::C => {
            return ProgramEvent::ToggleShowCards;
        },
        Keycode::H => {
            return ProgramEvent::ToggleShowHelp;
        },
        _ => (),
    }
    return ProgramEvent::None;
}

fn handle_c_cmds(keycode: Keycode) -> ProgramEvent {
    match keycode {
        Keycode::Num0 => {
            return ProgramEvent::SendRecordCoded(0);
        },
        Keycode::Num1 => {
            return ProgramEvent::SendRecordCoded(1);
        },
        _ => (),
    }
    return ProgramEvent::None;
}

fn handle_d_cmds(keycode: Keycode, keymod: Mod) -> ProgramEvent {
    match keycode {
        Keycode::E => {
            return ProgramEvent::DumpPGEntities;
        },
        Keycode::A => {
            if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                return ProgramEvent::DumpAIScoresSummary(actions::SUMMARY_RELATIVE_ALL);
            } else {
                return ProgramEvent::DumpAIScoresSummary(actions::SUMMARY_RELATIVE_TEAM);
            }
        },
        Keycode::D => {
            if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                return ProgramEvent::DumpAIDistancesSummary(actions::SUMMARY_RELATIVE_ALL);
            } else {
                return ProgramEvent::DumpAIDistancesSummary(actions::SUMMARY_RELATIVE_TEAM);
            }
        },
        Keycode::C => {
            return ProgramEvent::DumpIncCardScore;
        }
        Keycode::T => {
            if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                return ProgramEvent::DumpAITimeVsScoreSummary(actions::SUMMARY_RELATIVE_ALL);
            } else {
                return ProgramEvent::DumpAITimeVsScoreSummary(actions::SUMMARY_RELATIVE_TEAM);
            }
        },
        Keycode::LShift | Keycode::RShift => return ProgramEvent::NeedMore,
        _ => ldebug!(&format!("DBUG:GPPGND:Keys:DCmds:{}:{}", keycode, keymod)),
    }
    return ProgramEvent::None;
}

pub fn get_programevents(sx: &mut SdlX, skey: &mut String) -> ProgramEvent {
    for ev in sx.ep.poll_iter() {
        use sdl2::event::Event;
        if skey.len() > 0 {
            match ev {
                Event::Quit { timestamp: _} => return ProgramEvent::Quit,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod, repeat: _ } => {
                    let mut pev = ProgramEvent::None;
                    if skey == "s" {
                        pev = handle_s_cmds(keycode.unwrap());
                    }
                    if skey == "c" {
                        pev = handle_c_cmds(keycode.unwrap());
                    }
                    if skey == "d" {
                        pev = handle_d_cmds(keycode.unwrap(), keymod);
                    }
                    if let ProgramEvent::None = pev {
                        skey.clear();
                    }
                    return pev;
                },
                _ => return ProgramEvent::None,
            }
        }
        skey.clear();
        match ev {
            Event::Quit { timestamp: _ } => return ProgramEvent::Quit,
            Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod, repeat: _ } => {
                match keycode.unwrap() {
                    Keycode::B => {
                        return ProgramEvent::BackgroundColorChange;
                    }
                    Keycode::C => {
                        skey.push('c');
                        return ProgramEvent::NeedMore;
                    }
                    Keycode::P => {
                        return ProgramEvent::Pause;
                    }
                    Keycode::S => {
                        skey.push('s');
                        return ProgramEvent::NeedMore;
                    }
                    Keycode::Left => {
                        return ProgramEvent::SeekBackward;
                    }
                    Keycode::Right => {
                        return ProgramEvent::SeekForward;
                    }
                    Keycode::F => {
                        if keymod.contains(Mod::RSHIFTMOD) || keymod.contains(Mod::LSHIFTMOD) {
                            return ProgramEvent::AdjustFPS(1.20);
                        } else {
                            return ProgramEvent::AdjustFPS(0.80);
                        }
                    }
                    Keycode::H => {
                        return ProgramEvent::ToggleShowHelp;
                    }
                    Keycode::D => {
                        skey.push('d');
                        return ProgramEvent::NeedMore;
                    }
                    _ => {

                    }
                }
            },
            _ => (),
        }
    }
    ProgramEvent::None
}
