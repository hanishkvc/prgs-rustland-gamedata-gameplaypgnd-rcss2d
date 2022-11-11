//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

use crate::entities::TeamUpdates;

pub struct Rcg {
    _fname: String,
    _file: File,
    lines: Vec<String>,
    iline: isize,
    pub bdone: bool,
}

impl Rcg {

    pub fn new(fname: &str) -> Rcg {
        let mut file = File::open(fname).unwrap();
        let mut sdata = String::new();
        let _gotr = file.read_to_string(&mut sdata).unwrap();
        let vdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vline = Vec::new();
        for line in vdata {
            vline.push(line.to_string());
        }
        Rcg {
            _fname: fname.to_string(),
            _file: file,
            lines: vline,
            iline: -1,
            bdone: false,
        }
    }

    pub fn next_record(&mut self) -> TeamUpdates {
        let bcontinue = true;
        let mut tu = TeamUpdates::new();
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
            if toks[0].starts_with("show") {
                for tok in toks {
                    if !tok.starts_with("((l") && !tok.starts_with("((r") {
                        continue;
                    }
                    let mut tstr = TStr::from_str(&tok, true);
                    tstr.peel_bracket('(').unwrap();
                    let vdata = tstr.tokens_vec(' ', true, true).unwrap();
                    if vdata[0].starts_with("(l 4)") {
                        let fx: f32 = vdata[3].parse().unwrap();
                        let fy: f32 = vdata[3].parse().unwrap();
                        tu.ateampositions.push((4, fx, fy));
                    }
                }
                break;
            } else {
                print!("DBUG:PGND:Rcg:Skipping:{:?}\n", toks);
            }
        }
        return tu;
    }

}
