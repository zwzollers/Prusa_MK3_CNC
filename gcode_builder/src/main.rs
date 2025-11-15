use std::f64::{INFINITY, NEG_INFINITY};
use dxf::{Point, point};
use clap::Parser;

mod shape;
use shape::{DXF};

mod polyline;
use polyline::{Polylines};

mod gcode;
use gcode::{Command, Gcode};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short = 'c', long)]
    copper: String,

    #[arg(short = 'o', long)]
    outline: String,

    #[arg(short = 'g', long)]
    gcode: String,
}

fn main() {
    let args = Args::parse();

    let cu_dxf = DXF::from_file(args.copper);
    //println!("DXF shapes: {dxf:?}");
    let cu_polylines = Polylines::from_dxf(&cu_dxf);
    println!("polylines: {cu_polylines:#?}");

    //let outline_dxf = DXF::from_file(args.outline);
    //let outline_polylines = Polylines::from_dxf(&outline_dxf);

    //println!("outer bound: {:#?}", outline_polylines.aabb);
    

    //println!("count: {}", cu_dxf.num_shapes());

    let mut gcode = Gcode::default();

    gcode.push(Command::G90);
    gcode.push(Command::G1 { x:None, y: None, z: Some(50.0), f: Some(5000.0) });
    gcode.push(Command::G28 { x: true, y: true, z: false });
    gcode.feedrate = 5000.0;

    gcode.push(Command::G1 { x: Some(0.0), y: Some(0.0), z: Some(2.0), f: Some(5000.0) });

    for shape in &cu_polylines.shapes {
        gcode.push_shape(shape, point!(100,100));
    }

    gcode.push(Command::G1 { x: None, y: None, z: Some(50.0), f: Some(10000.0) });

    gcode.push(Command::M84);

    gcode.export(args.gcode);
    
}

fn almost_eq(a: &Point, b: &Point, diff: f64) -> bool {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();

    return dx < diff && dy < diff;
}



