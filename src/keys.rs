//!
//! Key shortcuts
//! HanishKVC, 2022
//!

use sdl2::keyboard::Keycode;

use crate::sdlx::SdlX;

pub enum ProgramEvent {
    None,
    Pause,
    BackgroundColorChange,
    ToggleShowHelp,
    ToggleShowActions,
    ToggleShowBall,
    ToggleShowStamina,
    SeekBackward,
    SeekForward,
    AdjustFPS(f32),
    SendRecordCoded(isize),
    DumpPGEntities,
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

pub fn get_programevents(sx: &mut SdlX, skey: &mut String) -> ProgramEvent {
    for ev in sx.ep.poll_iter() {
        use sdl2::event::Event;
        use sdl2::keyboard::Mod;
        if skey.len() > 0 {
            match ev {
                Event::Quit { timestamp: _} => return ProgramEvent::Quit,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    let mut pev = ProgramEvent::None;
                    if skey == "s" {
                        pev = handle_s_cmds(keycode.unwrap());
                    }
                    if skey == "c" {
                        pev = handle_c_cmds(keycode.unwrap());
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
                        return ProgramEvent::DumpPGEntities;
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