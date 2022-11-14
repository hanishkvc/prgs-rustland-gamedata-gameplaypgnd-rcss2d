//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

use crate::playdata::PlayUpdate;
use crate::playdata::PlayData;
use crate::sdlx::XSpaces;

/// Currently the time in terms of seconds (could be a fraction),
/// between the records maintained in the rcg file, is hard coded, here.
const SECONDS_PER_RECORD: f32 = 0.2;

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
            secondsper_record: SECONDS_PER_RECORD,
            secondsafter_lastrecord: 0.0,
            secondsperframe: 1.0/fps,
            r2d: XSpaces::new(rrect, drect)
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
        let bcontinue = true;
        let mut pu = PlayUpdate::new();
        while bcontinue {
            self.iline += 1;
            if self.iline >= self.lines.len() as isize {
                print!("WARN:PGND:Rcg:No more data\n");
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
            pu.msgs.insert("stime".to_string(), toks[1].to_string());
            if toks[0].starts_with("show") {
                for tok in toks {
                    if !tok.starts_with("((l") && !tok.starts_with("((r") && !tok.starts_with("((b") {
                        continue;
                    }
                    let mut tstr = TStr::from_str(&tok, true);
                    tstr.peel_bracket('(').unwrap();
                    let vdata = tstr.tokens_vec(' ', true, true).unwrap();
                    if vdata[0].starts_with("(b") {
                        let fxin: f32 = vdata[1].parse().unwrap();
                        let fyin: f32 = vdata[2].parse().unwrap();
                        let fx = self.r2d.d2ox(fxin);
                        let fy = self.r2d.d2oy(fyin);
                        if (fx < 0.0) || (fx > 1.0) || (fy < 0.0) || (fy > 1.0) {
                            eprintln!("DBUG:Rcg:Ball:BeyondBoundry:{},{}:{},{}", fxin, fyin, fx, fy);
                        }
                        pu.ball = (fx, fy);
                        continue;
                    }
                    let mut tstr = TStr::from_str(&vdata[0], true);
                    tstr.peel_bracket('(').unwrap();
                    let (steam, splayer) = tstr.split_once(' ').unwrap();
                    let iplayer: i32 = splayer.parse().unwrap();
                    let fxin: f32 = vdata[3].parse().unwrap();
                    let fyin: f32 = vdata[4].parse().unwrap();
                    let fx = self.r2d.d2ox(fxin);
                    let fy = self.r2d.d2oy(fyin);
                    if (fx < 0.0) || (fx > 1.0) || (fy < 0.0) || (fy > 1.0) {
                        eprintln!("DBUG:Rcg:Player:BeyondBoundry:{},{}:{},{}", fxin, fyin, fx, fy);
                    }
                    if steam == "l" {
                        pu.ateampositions.push((iplayer-1, fx, fy));
                    } else {
                        pu.bteampositions.push((iplayer-1, fx, fy));
                    }
                }
                break;
            } else if toks[0].starts_with("playmode") {
                pu.msgs.insert("game".to_string(), self.lines[self.iline as usize].clone());
            } else if toks[0].starts_with("team") {
                pu.msgs.insert("score".to_string(), self.lines[self.iline as usize].clone());
            } else {
                pu.msgs.insert("unknown".to_string(), self.lines[self.iline as usize].clone());
                print!("DBUG:PGND:Rcg:Skipping:{:?}\n", toks);
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

}
