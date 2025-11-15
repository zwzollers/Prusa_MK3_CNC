use dxf::Point;

use super::polyline::Polyline;
use std::fs::File;
use std::io::prelude::*;

#[derive(Default)]
pub struct Gcode {
    cmds: Vec<Command>,
    pub feedrate: f64,
}

impl Gcode {
    pub fn export(&self, fname: String) {
        let mut str = "".to_string();

        for cmd in &self.cmds {
            str.push_str(&cmd.to_string());
        }
        let mut file = File::create(fname).unwrap();
        file.write_all(str.as_bytes()).unwrap();
    }

    pub fn push(&mut self, cmd: Command) {
        self.cmds.push(cmd);
    }

    pub fn push_shape(&mut self, shape: &Polyline, offset: &Point) {
        self.push(Command::G1 { x: Some(shape.points[0].x + offset.x), y: Some(shape.points[0].y + offset.y), z: None, f: Some(self.feedrate) });
        self.push(Command::G1 { x: None, y: None, z: Some(0.0), f: Some(self.feedrate) });
        for p in &shape.points {
            self.push(Command::G1 { x: Some(p.x + offset.x), y: Some(p.y + offset.y), z: None, f: Some(self.feedrate) });
        }
        self.push(Command::G1 { x: None, y: None, z: Some(2.0), f: Some(self.feedrate) });
    }

    pub fn push_line(&mut self, line: (&Point, &Point), offset: &Point) {
        self.push(Command::G1 { x: Some(line.0.x + offset.x), y: Some(line.0.y + offset.y), z: None, f: Some(self.feedrate) });
        self.push(Command::G1 { x: None, y: None, z: Some(0.0), f: Some(self.feedrate) });
        self.push(Command::G1 { x: Some(line.1.x + offset.x), y: Some(line.1.y + offset.y), z: None, f: Some(self.feedrate) });
        self.push(Command::G1 { x: None, y: None, z: Some(2.0), f: Some(self.feedrate) });
    }
}

pub enum Command {
    G1 {x: Option<f64>, y: Option<f64>, z: Option<f64>, f: Option<f64>},
    G28 {x: bool, y: bool, z: bool},
    G90,
    M84,
}

impl Command {
    fn to_string(&self) -> String {
        match self {
            Command::G1 { x, y, z, f } => {
                format!("G1 {} {} {} {}\n",
                    if let Some(x) = x {format!("X{}", x.to_string())} else {"".into()},
                    if let Some(y) = y {format!("Y{}", y.to_string())} else {"".into()},
                    if let Some(z) = z {format!("Z{}", z.to_string())} else {"".into()},
                    if let Some(f) = f {format!("F{}", f.to_string())} else {"".into()},
                )
            },
            Command::G28 { x, y, z } => {
                format!("G28 {} {} {}\n",
                    if *x {"X"} else {""},
                    if *y {"Y"} else {""},
                    if *z {"Z"} else {""},
                )
            },
            Command::G90 => {
                "G90\n".into()
            }
            Command::M84 => {
                "M84\n".into()
            }
        }
    }
}