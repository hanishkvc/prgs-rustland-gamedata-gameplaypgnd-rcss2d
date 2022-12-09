//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use loggerk::{ldebug, log_d};
use tokensk::TStr;

use crate::playdata;
use crate::playdata::GameState;
use crate::playdata::rcss;
use crate::playdata::PlayUpdate;
use crate::playdata::PlayData;
use crate::playdata::PlayerData;
use crate::playdata::VPlayerData;
use crate::sdlx::XSpaces;


const MTAG: &str = "GPPGND:PlayDataRcg";

pub struct Rcg {
    _fname: String,
    _file: File,
    lines: Vec<String>,
    iline: isize,
    pub bdone: bool,
    secondsper_record: f32,
    secondsafter_lastrecord: f32,
    secondsperframe: f32,
    r2d: XSpaces,
}

impl Rcg {

    pub fn new(fname: &str, fps: f32) -> Rcg {
        let mut file = File::open(fname).unwrap();
        let mut sdata = String::new();
        let _gotr = file.read_to_string(&mut sdata).unwrap();
        let vdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vline = Vec::new();
        for line in vdata {
            vline.push(line.to_string());
        }
        let rrect = ((-55.0, -37.0), (55.0, 37.0));
        let drect = ((0.0,0.0), (1.0,1.0));
        Rcg {
            _fname: fname.to_string(),
            _file: file,
            lines: vline,
            iline: -1,
            bdone: false,
            secondsper_record: rcss::SECONDS_PER_RECORD,
            secondsafter_lastrecord: 0.0,
            secondsperframe: 1.0/fps,
            r2d: XSpaces::new(rrect, drect)
        }
    }

}

impl Rcg {

    fn handle_ball(&mut self, vdata: &Vec<String>, pu: &mut PlayUpdate) {
        let fxin: f32 = vdata[1].parse().unwrap();
        let fyin: f32 = vdata[2].parse().unwrap();
        let fx = self.r2d.d2ox(fxin);
        let fy = self.r2d.d2oy(fyin);
        if (fx < 0.0) || (fx > 1.0) || (fy < 0.0) || (fy > 1.0) {
            eprintln!("DBUG:{}:Ball:BeyondBoundry:{},{}:{},{}", MTAG, fxin, fyin, fx, fy);
        }
        pu.ball = (fx, fy);
    }

    fn handle_player(&mut self, vdata: &Vec<String>, pu: &mut PlayUpdate) {
        let mut pd = VPlayerData::new();
        // Handle team and player id
        let mut tstr = TStr::from_str(&vdata[0], true);
        tstr.peel_bracket('(').unwrap();
        let (steam, splayer) = tstr.split_once(' ').unwrap();
        let iplayer: i32 = splayer.parse().unwrap();
        // Handle actions and cards
        let sstate;
        if vdata[2].contains("x") {
            sstate = &vdata[2][2..];
        } else {
            sstate = &vdata[2];
        }
        let state: u32 = u32::from_str_radix(sstate, 16).unwrap();
        let (action, card) = rcss::handle_state(state);
        if (action == playdata::Action::None) && (card == playdata::Card::None) {
            ldebug!(&format!("DBUG:{}:Player:{}-{}:{}", MTAG, steam, iplayer, state));
        }
        pd.push(PlayerData::Card(card));
        pd.push(PlayerData::Action(action));
        // Handle position
        let fxin: f32 = vdata[3].parse().unwrap();
        let fyin: f32 = vdata[4].parse().unwrap();
        let fx = self.r2d.d2ox(fxin);
        let fy = self.r2d.d2oy(fyin);
        if (fx < 0.0) || (fx > 1.0) || (fy < 0.0) || (fy > 1.0) {
            eprintln!("DBUG:{}:Player:BeyondBoundry:{},{}:{},{}", MTAG, fxin, fyin, fx, fy);
        }
        pd.push(PlayerData::Pos(fx, fy));
        // Handle Direction
        let fbody: f32 = vdata[7].parse().unwrap();
        let fneck: f32 = vdata[8].parse().unwrap();
        pd.push(PlayerData::Dir(fbody, fneck));
        // Handle stamina
        for i in 5..vdata.len() {
            if !vdata[i].starts_with("(s ") {
                continue;
            }
            let sstamina = &vdata[i];
            let mut tstr = TStr::from_str(sstamina, true);
            tstr.peel_bracket('(').unwrap();
            let staminatoks = tstr.tokens_vec(' ', true, false).unwrap();
            //ldebug!(&format!("DBUG:PPGND:Rcg:Toks:Stamina:{:?}", staminatoks));
            let mut fstamina: f32 = staminatoks[1].parse().unwrap();
            fstamina = (fstamina/rcss::STAMINA_BASE).min(1.0);
            pd.push(PlayerData::Stamina(fstamina));
        }
        // Fill in the player data
        if steam == "l" {
            pu.lteamcoded.push((iplayer-1, pd));
        } else {
            pu.rteamcoded.push((iplayer-1, pd));
        }
    }

}

impl PlayData for Rcg {

    fn fps_changed(&mut self, fps: f32) {
        self.secondsperframe = 1.0/fps;
    }

    fn seconds_per_record(&self) -> f32 {
        self.secondsper_record
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        self.secondsafter_lastrecord += self.secondsperframe;
        if self.secondsafter_lastrecord >= self.secondsper_record {
            self.secondsafter_lastrecord = 0.0;
            return true;
        }
        return false;
    }

    fn next_record(&mut self) -> PlayUpdate {
        let fmtag: String = format!("{}:NextRecord", MTAG);
        let bcontinue = true;
        let mut pu = PlayUpdate::new();
        while bcontinue {
            self.iline += 1;
            if self.iline >= self.lines.len() as isize {
                print!("WARN:{}:No more data\n", fmtag);
                self.bdone = true;
                break;
            }
            let mut tstr = TStr::from_str(&self.lines[self.iline as usize], true);
            if tstr.len() == 0 {
                continue;
            }
            if tstr.char_first().unwrap() == '#' {
                continue;
            }
            if tstr.the_str().starts_with("ULG") {
                continue;
            }
            tstr.peel_bracket('(').unwrap();
            let toks = tstr.tokens_vec(' ', true, true).unwrap();
            ldebug!(&format!("DBUG:{}:Toks:Top:Full:{:?}", fmtag, toks));
            pu.msgs.insert("stime".to_string(), toks[1].to_string());
            if toks[0].starts_with("show") {
                pu.timecounter = toks[1].parse().unwrap();
                for tok in toks {
                    if !tok.starts_with("((l") && !tok.starts_with("((r") && !tok.starts_with("((b") {
                        continue;
                    }
                    let mut tstr = TStr::from_str(&tok, true);
                    tstr.peel_bracket('(').unwrap();
                    let vdata = tstr.tokens_vec(' ', true, true).unwrap();
                    ldebug!(&format!("DBUG:{}:Toks:Full:{:?}", fmtag, vdata));
                    if vdata[0].starts_with("(b") {
                        self.handle_ball(&vdata, &mut pu);
                    } else {
                        self.handle_player(&vdata, &mut pu);
                    }
                }
                break;
            } else if toks[0].starts_with("playmode") {
                if toks[2] == "goal_r" {
                    pu.state = GameState::Goal('r');
                } else if toks[2] == "goal_l" {
                    pu.state = GameState::Goal('l');
                } else if toks[2] == "play_on" {
                    pu.state = GameState::PlayOn;
                } else {
                    pu.state = GameState::Other(toks[2].clone());
                }
                pu.msgs.insert("game".to_string(), self.lines[self.iline as usize].clone());
            } else if toks[0].starts_with("team") {
                pu.msgs.insert("score".to_string(), self.lines[self.iline as usize].clone());
            } else {
                pu.msgs.insert("unknown".to_string(), self.lines[self.iline as usize].clone());
                print!("DBUG:{}:Skipping:{:?}\n", fmtag, toks);
            }
        }
        return pu;
    }

    fn seek(&mut self, seekdelta: isize) {
        self.iline += seekdelta;
        if self.iline < 0 {
            self.iline = 0;
        } else if self.iline as usize > self.lines.len() {
            self.iline = (self.lines.len() - 1) as isize;
        }
        if self.lines.len() > self.iline as usize {
            self.bdone = false;
        }
    }

    fn bdone(&self) -> bool {
        return self.bdone;
    }

    fn send_record(&mut self, _buf: &[u8]) {
        todo!()
    }

    fn send_record_coded(&mut self, code: isize) {
        eprintln!("WARN:{}:SendRecordCoded:ignoring request for send record coded [{}]", MTAG, code);
    }

}
