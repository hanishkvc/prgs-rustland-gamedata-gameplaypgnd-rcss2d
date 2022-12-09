//!
//! Act as a monitor for robocup soccer sim
//! HanishKVC, 2022
//!

use std::net::UdpSocket;
use std::time;

use tokensk::TStrX;
use loggerk::{ldebug,log_d};

use crate::sdlx::XSpaces;

use crate::playdata;
use super::{rcss, GameState};
use super::{PlayData, PlayUpdate, PlayerData};


const MTAG: &str = "GPPGND:PlayDataRCLive";

pub const NWADDR_DEFAULT: &str = "0.0.0.0:6000";
const OWN_ADDRESS: &str = "0.0.0.0:6600";
const READ_TIMEOUT_MS: u64 = 500;


/// Help act as a simple monitor client for RoboCup Sim
pub struct RCLive {
    skt: UdpSocket,
    /// The robocup server address to communicate to.
    srvraddr: String,
    /// Help tokenise recieved data.
    tstrx: TStrX,
    /// Help convert from Robocups pitch space to normal space.
    r2n: XSpaces,
    /// Track whether the server addr has been updated to
    /// the one over which server sent data to the monitor.
    /// ie after the initial handshake.
    bsrvraddr_updated: bool,
    /// Time wrt last message seen from server
    stime: String,
    /// Team name and Score
    ateam: String,
    bteam: String,
}

impl RCLive {

    pub fn new(addr: &str) -> RCLive {
        let skt = UdpSocket::bind(OWN_ADDRESS).unwrap();
        skt.set_read_timeout(Some(time::Duration::from_millis(READ_TIMEOUT_MS))).unwrap();
        let sinit = "(dispinit version 5)\r\n";
        skt.send_to(sinit.as_bytes(), addr).unwrap();
        eprintln!("DBUG:{}:New:{:?}", MTAG, skt);
        let rrect = ((-55.0, -37.0), (55.0, 37.0));
        let nrect = ((0.0,0.0), (1.0,1.0));
        let mut tstrx = TStrX::new();
        tstrx.flags.string_canbe_asubpart = true;
        tstrx.flags.blocktok_dlimuser_endreqd = false;
        tstrx.delims.bracket = ('{','}');
        tstrx.delims.obracket = Some(('[',']'));
        tstrx.delims.string = '"';
        RCLive {
            skt: skt,
            srvraddr: addr.to_string(),
            tstrx: tstrx,
            r2n: XSpaces::new(rrect, nrect),
            bsrvraddr_updated: false,
            stime: String::new(),
            ateam: String::new(),
            bteam: String::new(),
        }
    }

}

impl RCLive {

    fn handle_time(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_,d) = tok.split_once(':').unwrap();
        pu.timecounter = d.parse().unwrap();
        pu.msgs.insert("stime".to_string(), d.to_string());
        self.stime = d.to_string();
    }

    fn handle_mode(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_t,d) = tok.split_once(':').unwrap();
        let d = d.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
        pu.msgs.insert("game".to_string(), format!("{}:{}", self.stime, d));
        if d == "goal_r" {
            pu.state = GameState::Goal('r');
        } else if d == "goal_l" {
            pu.state = GameState::Goal('l');
        } else if d == "play_on" {
            pu.state = GameState::PlayOn;
        } else {
            pu.state = GameState::Other(d.to_string());
        }
    }

    fn handle_teams(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_t,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('[').unwrap();
        let teams = tstr.tokens_vec(',', true, false).unwrap();
        for team in teams {
            if team.trim().len() == 0 {
                continue;
            }
            let mut tstr = self.tstrx.from_str(&team, true);
            tstr.peel_bracket('{').unwrap();
            let toks = tstr.tokens_vec(',', true, false).unwrap();
            let mut side = '?';
            let mut name = String::new();
            let mut score = String::new();
            for tok in toks {
                if tok.starts_with("\"side\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    side = d.chars().nth(1).unwrap();
                }
                if tok.starts_with("\"name\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    let mut tstr = self.tstrx.from_str(d, true);
                    if tstr.char_first().unwrap() == '"' {
                        tstr.peel_string('"').unwrap();
                    }
                    name = tstr.to_string();
                }
                if tok.starts_with("\"score\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    score = d.to_string();
                }
            }
            let ts = format!("{} [{}]", name, score);
            if side == 'l' {
                self.ateam = ts;
            } else if side == 'r' {
                self.bteam = ts;
            }
        }
        pu.msgs.insert("score".to_string(), format!("{} vs {}", self.ateam, self.bteam));
    }

    fn handle_ball(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_b,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('{').unwrap();
        let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
        let mut fx = 0.0;
        let mut fy = 0.0;
        for tokl2 in toksl2 {
            let (k,v) = tokl2.split_once(':').unwrap();
            if k == "\"x\"" {
                fx = v.parse().unwrap();
            }
            if k == "\"y\"" {
                fy = v.parse().unwrap();
            }
        }
        let (fx,fy) = self.r2n.d2o((fx,fy));
        pu.ball = (fx, fy);
    }

    fn handle_players(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_p,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('[').unwrap();
        let toks = tstr.tokens_vec(',', true, false).unwrap();
        ldebug!(&format!("DBUG:{}:Players:Got:Toks:Players:{:#?}", MTAG, toks));
        // handle the individual players
        for tok in toks {
            if tok.trim().len() == 0 {
                continue;
            }
            let mut tstr = self.tstrx.from_str(&tok, true);
            tstr.peel_bracket('{').unwrap();
            let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
            ldebug!(&format!("DBUG:{}:Players:Got:Toks:Player:{:#?}", MTAG, toksl2));
            let mut pid = String::new();
            let mut fx = 0.0;
            let mut fy = 0.0;
            let mut side = String::new();
            let mut fstamina = 1.0f32;
            let mut card = playdata::Card::None;
            let mut action = playdata::Action::None;
            let mut fbody = 0.0;
            let mut fneck = 0.0;
            let mut fvw = 60.0;
            // Extract the player specific datas
            for tokl2 in toksl2 {
                let (k,v) = tokl2.split_once(':').unwrap();
                if k == "\"side\"" {
                    side = v.to_string();
                }
                if k == "\"unum\"" {
                    pid = v.to_string();
                }
                if k == "\"x\"" {
                    fx = v.parse().unwrap();
                }
                if k == "\"y\"" {
                    fy = v.parse().unwrap();
                }
                if k == "\"body\"" {
                    fbody = v.parse().unwrap();
                }
                if k == "\"neck\"" {
                    fneck = v.parse().unwrap();
                }
                if k == "\"vw\"" {
                    fvw = v.parse().unwrap();
                }
                if k == "\"stamina\"" {
                    fstamina = v.parse().unwrap();
                }
                if k == "\"state\"" {
                    let state: u32 = v.parse().unwrap();
                    (action, card) = rcss::handle_state(state);
                    if (action == playdata::Action::None) && (card == playdata::Card::None) {
                        ldebug!(&format!("DBUG:{}:Players:{}-{}:{}", MTAG, side, pid, state));
                    }
                }
            }
            let (fx,fy) = self.r2n.d2o((fx,fy));
            fstamina = (fstamina/rcss::STAMINA_BASE).min(1.0);
            let mut pd = playdata::VPlayerData::new();
            pd.push(PlayerData::Pos(fx, fy));
            pd.push(PlayerData::Stamina(fstamina));
            pd.push(PlayerData::Card(card));
            pd.push(PlayerData::Action(action));
            let (fbody, fneck) = rcss::handle_dir(fbody, fneck);
            pd.push(PlayerData::Dir(fbody, fneck, fvw));
            if side.chars().nth(1).unwrap() == 'l' {
                pu.lteamcoded.push((pid, pd));
            } else {
                pu.rteamcoded.push((pid, pd));
            }
        }

    }

}

impl PlayData for RCLive {

    fn seconds_per_record(&self) -> f32 {
        return rcss::SECONDS_PER_RECORD;
    }

    fn fps_changed(&mut self, _fps: f32) {
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        return true;
    }

    ///
    /// A successful reception of a record/message from the server for
    /// the 1st time, will update the internally maintained server address
    /// to point to the address (including port) from which the record was
    /// recieved.
    ///
    fn next_record(&mut self) -> super::PlayUpdate {
        let fmtag = format!("{}:NextRecord", MTAG);
        let mut pu = PlayUpdate::new();
        let mut buf = [0u8; 8196];
        let gotr = self.skt.recv_from(&mut buf);
        if gotr.is_err() {
            let err = gotr.unwrap_err();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                eprintln!("WARN:{}:No data...", fmtag);
                return pu;
            } else {
                panic!("ERRR:{}:Unexpected error:{}", fmtag, err);
            }
        }
        if !self.bsrvraddr_updated {
            self.srvraddr = gotr.as_ref().unwrap().1.to_string();
            self.bsrvraddr_updated = true;
        }
        let sbuf = String::from_utf8_lossy(&buf);
        ldebug!(&format!("DBUG:{}:Got:{:?}:{}", fmtag, gotr, &sbuf));
        if !sbuf.starts_with("{") {
            eprintln!("WARN:{}:Ignoring unexpected data [{}]...", fmtag, sbuf);
            return pu;
        }
        let mut tstr = self.tstrx.from_str(&sbuf, true);
        tstr.peel_bracket('{').unwrap();
        let toks = tstr.tokens_vec(',', true, true).unwrap();
        ldebug!(&format!("DBUG:{}:Got:Toks:Full:{:#?}", fmtag, toks));
        for tok in toks {
            if tok.starts_with("\"type\"") {
                let (_t,d) = tok.split_once(':').unwrap();
                if d != "\"show\"" {
                    eprintln!("DBUG:{}:UnhandledTypeMsg:{}", fmtag, sbuf);
                    let ilen = sbuf.trim().len().min(32);
                    pu.msgs.insert("unknown".to_string(), sbuf[0..ilen].to_string());
                    return pu;
                }
                continue;
            }
            if tok.starts_with("\"time\"") {
                self.handle_time(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"mode\"") {
                self.handle_mode(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"teams\"") {
                self.handle_teams(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"ball\"") {
                self.handle_ball(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"players\"") {
                self.handle_players(&tok, &mut pu);
                continue;
            }
        }
        ldebug!(&format!("DBUG:{}:Got:Pu:{:?}", fmtag, pu));
        pu
    }

    fn seek(&mut self, _seekdelta: isize) {
        return;
    }

    fn bdone(&self) -> bool {
        return false;
    }

    fn send_record(&mut self, buf: &[u8]) {
        self.skt.send_to(buf, &self.srvraddr).unwrap();
        eprintln!("DBUG:{}:Sent:{:?}:To:{:?}-{:?}", MTAG, buf, self.skt, self.srvraddr);
    }

    fn send_record_coded(&mut self, code: isize) {
        let msg = match code {
            0 => "(dispinit version 5)\r\n".as_bytes(),
            1 => "(dispstart)\x00".as_bytes(),
            _ => todo!(),
        };
        self.send_record(msg);
    }

}
