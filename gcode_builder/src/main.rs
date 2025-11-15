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
    //println!("polylines: {cu_polylines:#?}");

    let offset_cu = cu_polylines.shapes[1].offset(-5.0);
    println!("{offset_cu:?} before: {} after:{}", cu_polylines.shapes[1].length(), offset_cu.length());

    let outline_dxf = DXF::from_file(args.outline);
    let outline_polylines = Polylines::from_dxf(&outline_dxf);

    //println!("outer bound: {:#?}", outline_polylines.aabb);
    

    //println!("count: {}", cu_dxf.num_shapes());

    let mut gcode = Gcode::default();

    let offset = point!(100,100);

    gcode.push(Command::G90);
    gcode.push(Command::G1 { x:None, y: None, z: Some(50.0), f: Some(5000.0) });
    gcode.push(Command::G28 { x: true, y: true, z: false });
    gcode.feedrate = 5000.0;

    gcode.push(Command::G1 { x: Some(0.0), y: Some(0.0), z: Some(2.0), f: Some(5000.0) });

    gcode.push_shape(&offset_cu, &offset);
    gcode.push_shape(&cu_polylines.shapes[1], &offset);

    // for shape in &cu_polylines.shapes {
    //     gcode.push_shape(shape, &offset);
    // }

    // let mut line = (outline_polylines.aabb.0.clone(), point!(outline_polylines.aabb.0.x, outline_polylines.aabb.1.y));

    // let tool_diameter = 1.5875;

    // let lines = ((outline_polylines.aabb.1.x - outline_polylines.aabb.0.x) / tool_diameter).ceil() as usize;


    // for _ in 0..lines {

    //     let mut intersections = cu_polylines.find_intersections((&line.0, &line.1));
    //     intersections.append(&mut outline_polylines.find_intersections((&line.0, &line.1)));

    //     intersections.sort_by(|a,b| a.y.partial_cmp(&b.y).unwrap());

    //     for line in intersections.chunks_exact(2) {
    //         gcode.push_line((&line[0], &line[1]), &offset);
    //     }

    //     line.0.x += tool_diameter;
    //     line.1.x += tool_diameter;
    // }

    gcode.push(Command::G1 { x: None, y: None, z: Some(50.0), f: Some(10000.0) });

    gcode.push(Command::M84);

    gcode.export(args.gcode);
    
}

fn almost_eq(a: &Point, b: &Point, diff: f64) -> bool {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();

    return dx < diff && dy < diff;
}



