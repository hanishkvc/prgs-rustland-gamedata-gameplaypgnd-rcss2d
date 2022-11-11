//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

struct Rcg {
    fname: String,
    file: File,
    lines: Vec<String>,
    inext: usize,
}

impl Rcg {

    pub fn new(fname: &str) -> Rcg {
        let mut file = File::open(fname).unwrap();
        let mut sdata = String::new();
        let gotr = file.read_to_string(&mut sdata).unwrap();
        let vdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vline = Vec::new();
        for line in vdata {
            vline.push(line.to_string());
        }
        Rcg {
            fname: fname.to_string(),
            file: file,
            lines: vline,
            inext: 0,
        }
    }

}