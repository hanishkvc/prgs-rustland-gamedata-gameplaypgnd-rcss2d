//!
//! Key shortcuts
//! HanishKVC, 2022
//!

use crate::sdlx::SdlX;

pub enum ProgramEvent {
    None,
    Pause,
    BackgroundColorChange,
    ToggleShowHelp,
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

pub fn get_programevents(sx: &mut SdlX, skey: &mut String) -> ProgramEvent {
    for ev in sx.ep.poll_iter() {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        use sdl2::keyboard::Mod;
        if skey == "s" {
            match ev {
                Event::Quit { timestamp: _} => return ProgramEvent::Quit,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    match keycode.unwrap() {
                        Keycode::B => {
                            return ProgramEvent::ToggleShowBall;
                        },
                        Keycode::S => {
                            return ProgramEvent::ToggleShowStamina;
                        },
                        Keycode::H => {
                            return ProgramEvent::ToggleShowHelp;
                        },
                        Keycode::Escape | _ => {
                            skey.clear();
                            return ProgramEvent::None;
                        },
                    }
                },
                _ => return ProgramEvent::None,
            }
        }
        skey.clear();
        match ev {
            Event::Quit { timestamp: _ } => return ProgramEvent::Quit,
            Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod, repeat: _ } => {
                match keycode.unwrap() {
                    Keycode::C => {
                        return ProgramEvent::BackgroundColorChange;
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
                    Keycode::Num1 => {
                        return ProgramEvent::SendRecordCoded(1);
                    }
                    Keycode::Num0 => {
                        return ProgramEvent::SendRecordCoded(0);
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
