extern crate dxf;
use dxf::Drawing;
use dxf::Point;
use dxf::entities::*;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short = 'f', long)]
    file: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    println!("reading: {}", args.file);

    let dxf = DXF::from_file(args.file);

    println!("DXF shapes: {dxf:?}");

    println!("count: {}", dxf.shapes.len());
}

#[derive(Debug)]
struct DXF {
    shapes: Vec<ClosedPolyLine>,
}

impl DXF {
    fn from_file(fname: String) -> Self {
        let dxf_raw = Drawing::load_file(fname).unwrap();
        let mut dxf = DXF { shapes: Vec::new() };

        'out: for e in dxf_raw.entities() {
            let item = Item::from_entity(e).unwrap();
            let circle = item.p1 == item.p2;

            if circle {
                dxf.shapes.push(ClosedPolyLine {
                    obj: vec![item],
                    closed: true,
                });
            } else {
                for i in 0..dxf.shapes.len() {
                    let push = almost_eq(&item.p1, &dxf.shapes[i].obj.last().unwrap().p2, 0.001)
                        || almost_eq(&item.p1, &dxf.shapes[i].obj.last().unwrap().p1, 0.001)
                        || almost_eq(&item.p2, &dxf.shapes[i].obj.last().unwrap().p2, 0.001);
                    let insert = almost_eq(&item.p1, &dxf.shapes[i].obj.first().unwrap().p2, 0.001)
                        || almost_eq(&item.p1, &dxf.shapes[i].obj.first().unwrap().p1, 0.001)
                        || almost_eq(&item.p2, &dxf.shapes[i].obj.first().unwrap().p2, 0.001);

                    if insert && !push {
                        dxf.shapes[i].obj.insert(0, item);
                        continue 'out;
                    } else if !insert && push {
                        dxf.shapes[i].obj.push(item);
                        continue 'out;
                    } else if insert && push {
                        dxf.shapes[i].obj.push(item);
                        dxf.shapes[i].closed = true;
                        continue 'out;
                    }
                }

                dxf.shapes.push(ClosedPolyLine {
                    obj: vec![item],
                    closed: false,
                });
            }
        }

        //cleanup

        dxf
    }
}

fn almost_eq(a: &Point, b: &Point, diff: f64) -> bool {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();

    return dx < diff && dy < diff;
}

#[derive(Debug)]
struct ClosedPolyLine {
    obj: Vec<Item>,
    closed: bool,
}

#[derive(Debug)]
struct Item {
    p1: Point,
    p2: Point,

    dxf_entity: EntityType,
}

impl Item {
    fn from_entity(entity: &Entity) -> Option<Self> {
        match entity.specific {
            EntityType::Circle(ref circle) => {
                let mut p: Point = circle.center.clone();
                p.x += circle.radius;

                Some(Item {
                    p1: p.clone(),
                    p2: p,
                    dxf_entity: entity.specific.clone(),
                })
            }
            EntityType::Line(ref line) => Some(Item {
                p1: line.p1.clone(),
                p2: line.p2.clone(),
                dxf_entity: entity.specific.clone(),
            }),
            EntityType::Arc(ref arc) => {
                let p1: Point = Point {
                    x: arc.center.x + (arc.start_angle.to_radians().cos() * arc.radius),
                    y: arc.center.y + (arc.start_angle.to_radians().sin() * arc.radius),
                    z: 0.0,
                };
                let p2: Point = Point {
                    x: arc.center.x + (arc.end_angle.to_radians().cos() * arc.radius),
                    y: arc.center.y + (arc.end_angle.to_radians().sin() * arc.radius),
                    z: 0.0,
                };

                Some(Item {
                    p1: p1,
                    p2: p2,
                    dxf_entity: entity.specific.clone(),
                })
            }
            _ => None,
        }
    }
}
