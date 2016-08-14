#![feature(plugin)]
#![feature(reflect_marker)]
#![feature(custom_attribute)]
#![feature(const_fn)]
#![feature(concat_idents)]

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
    run::<f64>();
}

fn run<F: CustomFloat>() {
    // let mut universe: Universe3D<F> = Universe3D::new();

    // universe.set_background(
    //     Box::new(
    //         MappedTextureImpl::new(
    //             uv_sphere(
    //                 na::origin()
    //             ),
    //             texture_image(
    //                 image::open("./resources/universe_dim.jpg").unwrap()
    //             )
    //         )
    //     )
    // );

    // {
    //     let mut entities = universe.entities_mut();
    //     entities.push(Box::new(Entity3Impl::new(
    //             Box::new(Sphere::new(
    //                 Point3::new(Cast::from(2.0),
    //                             Cast::from(1.0),
    //                             Cast::from(0.0)),
    //                 Cast::from(1.0)
    //             )),
    //             Box::new(Vacuum::new()),
    //             // Some(Box::new(PerlinSurface3::rand(&mut StdRng::new().expect("Could not create a random number generator."), 1.0, 1.0)))
    //             Some(Box::new(ComposableSurface {
    //                 reflection_ratio: reflection_ratio_uniform(Cast::from(1.0)),
    //                 reflection_direction: reflection_direction_specular(),
    //                 surface_color: surface_color_illumination_directional(-AXIS_Z()),
    //             }))
    //         )));
    //     entities.push(Box::new(Entity3Impl::new(
    //             Box::new(Sphere::new(
    //                     Point3::new(Cast::from(2.0),
    //                                 Cast::from(-1.5),
    //                                 Cast::from(0.0)),
    //                     Cast::from(1.25)
    //                     )),
    //             Box::new(Vacuum::new()),
    //             Some(Box::new(PerlinSurface3::rand(
    //                         &mut StdRng::new().expect("Could not create a random number generator."),
    //                         Cast::from(2.0),
    //                         Cast::from(1.0))))
    //             // Some(Box::new(ComposableSurface {
    //             //     reflection_ratio: get_reflection_ratio_test,
    //             //     reflection_direction: get_reflection_direction_test,
    //             //     surface_color: get_surface_color_test,
    //             // }))
    //         )));
    //     entities.push(Box::new(Entity3Impl::new(
    //             Box::new(Sphere::new(
    //                     Point3::new(Cast::from(2.0),
    //                                 Cast::from(10.5),
    //                                 Cast::from(0.0)),
    //                     Cast::from(2.25)
    //                     )),
    //             Box::new(Vacuum::new()),
    //             Some(
    //                 Box::new(
    //                     ComposableSurface {
    //                         reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
    //                         reflection_direction: reflection_direction_specular(),
    //                         surface_color: surface_color_texture(
    //                             Box::new(
    //                                 MappedTextureImpl::new(
    //                                     uv_sphere(
    //                                         Point3::new(Cast::from(2.0),
    //                                                     Cast::from(10.5),
    //                                                     Cast::from(0.0))
    //                                     ),
    //                                     texture_image(
    //                                         image::open("./resources/universe_bright.jpg").unwrap()
    //                                     )
    //                                 )
    //                             )
    //                         ),
    //                     }
    //                 )
    //             )
    //         )));
    //     entities.push(Box::new(Entity3Impl::new(
    //             Box::new(
    //                 ComposableShape::new(
    //                     VoidShape::new(),
    //                     Plane3::cuboid(
    //                         Point3::new(
    //                             Cast::from(0.0),
    //                             Cast::from(0.0),
    //                             Cast::from(0.0)
    //                         ),
    //                         Vector3::new(
    //                             Cast::from(4.0),
    //                             Cast::from(6.0),
    //                             Cast::from(3.0)
    //                         )
    //                     ),
    //                     SetOperation::Complement
    //                 )
    //             ),
    //             Box::new(Vacuum::new()),
    //             Some(Box::new(ComposableSurface {
    //                 reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
    //                 reflection_direction: reflection_direction_specular(),
    //                 surface_color: surface_color_illumination_directional(-AXIS_Z()),
    //             }))
    //         )));
    //     entities.push(
    //         Box::new(
    //             Entity3Impl::new(
    //                 Box::new(
    //                     ComposableShape::new(
    //                         Sphere::new(
    //                             Point3::new(
    //                                 Cast::from(5.0),
    //                                 Cast::from(1.5),
    //                                 Cast::from(5.0)
    //                             ),
    //                             Cast::from(3.0)
    //                         ),
    //                         ComposableShape::new(
    //                             Sphere::new(
    //                                 Point3::new(
    //                                     Cast::from(5.0),
    //                                     Cast::from(0.0),
    //                                     Cast::from(5.0)
    //                                 ),
    //                                 Cast::from(3.0)
    //                             ),
    //                             Sphere::new(
    //                                 Point3::new(
    //                                     Cast::from(5.0),
    //                                     Cast::from(3.0),
    //                                     Cast::from(5.0)
    //                                 ),
    //                                 Cast::from(3.0)
    //                             ),
    //                             SetOperation::SymmetricDifference
    //                         ),
    //                         SetOperation::Complement
    //                     ),
    //                 ),
    //                 Box::new(
    //                     Vacuum::new()
    //                 ),
    //                 Some(
    //                     Box::new(
    //                         ComposableSurface {
    //                             reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
    //                             reflection_direction: reflection_direction_specular(),
    //                             surface_color: surface_color_illumination_directional(-AXIS_Z()),
    //                         }
    //                     )
    //                 )
    //             )
    //         )
    //     );
    //     entities.push(Box::new(Void::new_with_vacuum()));
    // }

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
        .parse::<Box<Environment<F>>>("Environment", &json)
        .expect("Unable to parse the Environment.");

    let simulation = Simulation::builder()
        .environment(*environment)
        .build();

    simulation.start();
}
