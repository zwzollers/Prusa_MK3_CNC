extern crate dxf;
use dxf::Drawing;
use dxf::entities::*;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short = 'f', long,)]
    file: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    println!("reading: {}", args.file);

    let drawing = Drawing::load_file(args.file).unwrap();
    for e in drawing.entities() {
        match e.specific {
            EntityType::Circle(ref circle) => {
                println!("{circle:?}");
            },
            EntityType::Line(ref line) => {
                println!("{line:?}");
            },
            _ => {
                println!("WAWA: {:?}", e.specific)
            },
        }
    }
}

fn offset(entity: EntityType) {
    
}