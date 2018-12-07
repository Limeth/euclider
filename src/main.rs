#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(too_many_arguments))]

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
extern crate boolinator;
extern crate float_cmp;
extern crate smallvec;
#[macro_use]
extern crate mashup;
#[macro_use]
extern crate mopa;
#[macro_use]
extern crate parse_generics_shim;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate det;

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

#[cfg(feature = "low_precision")]
pub type F = f32;
#[cfg(not(feature = "low_precision"))]
pub type F = f64;

fn main() {
    if cfg!(feature = "low_precision") {
        println!("Running in low floating-point number precision mode.");
    }

    const ARG_SCENE: &str = "SCENE";
    const ARG_DEBUG: &str = "DEBUG";

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
                      .arg(Arg::with_name(ARG_DEBUG)
                               .short("d")
                               .long("debug")
                               .help("Displays debug info"))
                      .get_matches();

    let scene = matches.value_of(ARG_SCENE).unwrap();
    let debug = matches.is_present(ARG_DEBUG);
    let mut reader = BufReader::new(File::open(scene)
        .expect("Unable to find the scene file."));
    let mut json = String::new();
    reader.read_to_string(&mut json).expect("Unable to read the scene file.");
    let environment: Box<Box<Environment>> = scene::Parser::default()
        .parse::<Box<Environment>>(&json)
        .expect("Unable to parse the Environment.");

    let simulation = Simulation::builder()
        .environment(*environment)
        .threads(num_cpus::get() as u32)
        .debug(debug)
        .build();

    if debug {
        println!("Running in debug mode.");
    }

    simulation.start();
}
