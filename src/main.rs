#![feature(plugin)]
#![feature(reflect_marker)]
#![feature(custom_attribute)]
#![feature(const_fn)]
#![feature(concat_idents)]
#![feature(trace_macros)]

#![plugin(clippy)]

extern crate core;
extern crate nalgebra as na;
extern crate scoped_threadpool;
extern crate image;
extern crate noise;
extern crate rand;
extern crate palette;
extern crate glium;
extern crate num;
extern crate json;
extern crate num_cpus;
extern crate meval;
#[macro_use]
extern crate mopa;
#[macro_use]
extern crate parse_generics_shim;
#[macro_use]
extern crate clap;

#[macro_use]
pub mod util;
pub mod universe;
pub mod simulation;
pub mod scene;

use universe::Environment;
use util::CustomFloat;
use simulation::Simulation;
use std::io::BufReader;
use std::fs::File;
use std::io::Read;
use clap::App;
use clap::Arg;

fn main() {
    if cfg!(feature = "low_precision") {
        println!("Running in low floating-point number precision mode.");
        run::<f32>();
    } else {
        run::<f64>();
    }
}

fn run<F: CustomFloat>() {
    const ARG_SCENE: &'static str = "SCENE";

    let matches = App::new("euclider")
                      .version(crate_version!())
                      .author(crate_authors!())
                      .about("A non-euclidean raytracer")
                      .arg(Arg::with_name(ARG_SCENE)
                               .short("s")
                               .long("scene")
                               .help("Loads a `.json` scene file")
                               .takes_value(true)
                               .required(true))
                      .get_matches();

    let scene = matches.value_of(ARG_SCENE).unwrap();
    let mut reader = BufReader::new(File::open(scene)
        .expect("Unable to find the scene file."));
    let mut json = String::new();
    reader.read_to_string(&mut json).expect("Unable to read the scene file.");
    let environment: Box<Box<Environment<F>>> = scene::Parser::default::<F>()
        .parse::<Box<Environment<F>>>(&json)
        .expect("Unable to parse the Environment.");

    let simulation = Simulation::builder()
        .environment(*environment)
        .threads(num_cpus::get() as u32)
        .build();

    simulation.start();
}
