//!
//! Testlib
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::playdata::GameState;
use crate::sdlx::{self, SdlX};
use crate::entities;
use crate::entities::gentity::GEntity;

pub fn test_ncolor() {
    for i in 0..100 {
        let f = i as f32/100.0;
        let color = sdlx::ncolor_gyr(f);
        println!("DBUG:TestNColor:{}:{:?}",f, color);
    }
}

pub fn test_gentity(font: &Font) {
    let mut g1 = GEntity::new("test01", (0.5,0.5), (16,16), Color::WHITE, font);
    g1.set_fcolor(0.25, 1.0);
    g1.set_nxarc(1.2, 0.98, Color::RED);
}

#[allow(dead_code)]
pub fn test_dummy() {
    eprintln!("{},{:?}", entities::SIDE_R, GameState::PlayPaused);
}

#[allow(dead_code)]
fn test_sdlx_drawprims(sx: &mut SdlX) {
    let dp1 = sdlx::DrawPrimitive::NArc((0.5,0.5), 0.1, (0.0,1.0), 3, Color::BLACK);
    let dp2 = sdlx::DrawPrimitive::NNThickLine((0.0,0.0), (1.0,1.0), 0.01, Color::WHITE);
    dp1.draw(sx);
    dp2.draw(sx);
}

#[test]
fn test_sdlx_xspaces_dtoq1() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((0.0,0.0),(1.0,1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(0.0,0.0), (0.0,1.0), (1.0,1.0), (1.0,0.0), (0.5,0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:DtoQ1:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:DtoQ1:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
fn test_sdlx_xspaces_dtoq2() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((0.0,0.0),(-1.0,1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(0.0,0.0), (0.0,1.0), (-1.0,1.0), (-1.0,0.0), (-0.5,0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:DtoQ2:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:DtoQ2:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
fn test_sdlx_xspaces_dtoq3() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((0.0,0.0),(-1.0,-1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(0.0,0.0), (0.0,-1.0), (-1.0,-1.0), (-1.0,0.0), (-0.5,-0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:DtoQ3:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:DtoQ3:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
fn test_sdlx_xspaces_dtoq4() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((0.0,0.0),(1.0,-1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(0.0,0.0), (0.0,-1.0), (1.0,-1.0), (1.0,0.0), (0.5,-0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:DtoQ4:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:DtoQ4:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
/// Screen 0,0 left,top in Q3
fn test_sdlx_xspaces_sltinq3() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((-1.0,0.0),(0.0,-1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(-1.0,0.0), (-1.0,-1.0), (0.0,-1.0), (0.0,0.0), (-0.5,-0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:SltInQ3:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:SltInQ3:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
/// Screen 0,0 left,bottom in Q3
fn test_sdlx_xspaces_slbinq3() {
    let s2n = sdlx::XSpaces::new(((0.0,0.0),(640.0,480.0)), ((-1.0,-1.0),(0.0,0.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(-1.0,-1.0), (-1.0,0.0), (0.0,0.0), (0.0,-1.0), (-0.5,-0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:SlbInQ3:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:SlbInQ3:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
fn test_sdlx_xspaces_drtoq1() {
    let s2n = sdlx::XSpaces::new(((640.0,480.0),(0.0,0.0)), ((0.0,0.0),(1.0,1.0)));
    let vd = vec![(0.0,0.0), (0.0,480.0), (640.0,480.0), (640.0,0.0), (320.0,240.0)];
    let vo = vec![(1.0,1.0), (1.0,0.0), (0.0,0.0), (0.0,1.0), (0.5,0.5)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:DrtoQ1:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:DrtoQ1:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
/// Screen 0,0 in mid
fn test_sdlx_xspaces_sinmid() {
    let s2n = sdlx::XSpaces::new(((-320.0,240.0),(320.0,-240.0)), ((-0.5,0.5),(0.5,-0.5)));
    let vd = vec![(-320.0,240.0), (-320.0,-240.0), (320.0,-240.0), (320.0,240.0), (0.0,0.0)];
    let vo = vec![(-0.5,0.5),     (-0.5,-0.5),     (0.5,-0.5),     (0.5,0.5),     (0.0,0.0)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:SInMid:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:SInMid:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

#[test]
/// ScreenR 0,0 in mid
fn test_sdlx_xspaces_srinmid() {
    let s2n = sdlx::XSpaces::new(((-320.0,240.0),(320.0,-240.0)), ((0.5,-0.5),(-0.5,0.5)));
    let vd = vec![(-320.0,240.0), (-320.0,-240.0), (320.0,-240.0), (320.0,240.0), (0.0,0.0)];
    let vo = vec![(0.5,-0.5),     (0.5,0.5),       (-0.5,0.5),     (-0.5,-0.5),   (0.0,0.0)];
    for i in 0..vd.len() {
        eprintln!();
        eprintln!("Test:Sdlx:XSpaces:SrInMid:D2O:{:?}:{:?}", vd[i], s2n.d2o(vd[i]));
        eprintln!("Test:Sdlx:XSpaces:SrInMid:O2D:{:?}:{:?}", vo[i], s2n.o2d(vo[i]));
    }
}

pub fn sdlx_plots() {
    use sdl2::event::Event;
    use sdlx::PlotType;
    let mut sx = sdlx::SdlX::init_plus("Test SdlX", 640, 480, false);
    'mainloop: loop {
        for ev in sx.ep.poll_iter() {
            match ev {
                Event::Quit { timestamp: _ } => {
                    break 'mainloop;
                },
                _ => (),
            }
        }
        sx.n_plot_f(0.1, 0.4, 0.8, 0.3, vec![2.0, 3.0, 4.0, 3.0, 2.0, 1.0], 0.0, 5.0);
        sx.n_plot_uf(0.1, 0.9, 0.8, 0.4, &vec![(2,2.0), (4,4.0), (6,2.0)], 0.0, 8.0, -1.0, 6.0, Some(vec![0.2,0.6,0.2]), "test plotuf", PlotType::Lines);
        sx.wc.present();
    }
}

#[test]
fn test_sdlx_plots() {
    sdlx_plots();
}
