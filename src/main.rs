#![feature(plugin)]
#![feature(reflect_marker)]
#![feature(custom_attribute)]
#![feature(const_fn)]
#![feature(zero_one)]

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
pub mod universe;
pub mod util;
pub mod simulation;

use rand::StdRng;
use na::Cast;
use na::Point3;
use na::Vector3;
use universe::Universe;
use universe::entity::Void;
use universe::entity::shape::*;
use universe::entity::material::*;
use universe::entity::surface::*;
use universe::d3::Universe3D;
use universe::d3::entity::Entity3Impl;
use universe::d3::entity::shape::*;
use universe::d3::entity::surface::*;
use util::CustomFloat;
use util::HasId;
use simulation::Simulation;

fn main() {
    run::<f64>();
}

fn run<F: CustomFloat>() {
    let mut universe: Universe3D<F> = Universe3D::new();

    {
        let mut entities = universe.entities_mut();
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                    Point3::new(Cast::from(2.0),
                                Cast::from(1.0),
                                Cast::from(0.0)),
                    Cast::from(1.0)
                )),
                Box::new(Vacuum::new()),
                // Some(Box::new(PerlinSurface3::rand(&mut StdRng::new().expect("Could not create a random number generator."), 1.0, 1.0)))
                Some(Box::new(ComposableSurface {
                    reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
                    reflection_direction: reflection_direction_specular(),
                    surface_color: Box::new(get_surface_color_test),
                }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                        Point3::new(Cast::from(2.0),
                                    Cast::from(-1.5),
                                    Cast::from(0.0)),
                        Cast::from(1.25)
                        )),
                Box::new(Vacuum::new()),
                Some(Box::new(PerlinSurface3::rand(
                            &mut StdRng::new().expect("Could not create a random number generator."),
                            Cast::from(2.0),
                            Cast::from(1.0))))
                // Some(Box::new(ComposableSurface {
                //     reflection_ratio: get_reflection_ratio_test,
                //     reflection_direction: get_reflection_direction_test,
                //     surface_color: get_surface_color_test,
                // }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                        Point3::new(Cast::from(2.0),
                                    Cast::from(10.5), // FIXME: The Y-coordinate seems to be flipped.
                                    Cast::from(0.0)),
                        Cast::from(2.25)
                        )),
                Box::new(Vacuum::new()),
                Some(Box::new(PerlinSurface3::rand(
                            &mut StdRng::new().expect("Could not create a random number generator."),
                            Cast::from(2.0),
                            Cast::from(1.0))))
                // Some(Box::new(ComposableSurface {
                //     reflection_ratio: get_reflection_ratio_test,
                //     reflection_direction: get_reflection_direction_test,
                //     surface_color: get_surface_color_test,
                // }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(
                    ComposableShape::new(
                        VoidShape::new(),
                        Plane3::cuboid(
                            Point3::new(
                                Cast::from(0.0),
                                Cast::from(0.0),
                                Cast::from(0.0)
                            ),
                            Vector3::new(
                                Cast::from(4.0),
                                Cast::from(6.0),
                                Cast::from(3.0)
                            )
                        ),
                        SetOperation::Complement
                    )
                ),
                Box::new(Vacuum::new()),
                Some(Box::new(ComposableSurface {
                    reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
                    reflection_direction: reflection_direction_specular(),
                    surface_color: Box::new(get_surface_color_test),
                }))
            )));
        entities.push(
            Box::new(
                Entity3Impl::new(
                    Box::new(
                        ComposableShape::new(
                            Sphere3::<F>::new(
                                Point3::new(
                                    Cast::from(5.0),
                                    Cast::from(1.5),
                                    Cast::from(5.0)
                                ),
                                Cast::from(3.0)
                            ),
                            ComposableShape::new(
                                Sphere3::<F>::new(
                                    Point3::new(
                                        Cast::from(5.0),
                                        Cast::from(0.0),
                                        Cast::from(5.0)
                                    ),
                                    Cast::from(3.0)
                                ),
                                Sphere3::<F>::new(
                                    Point3::new(
                                        Cast::from(5.0),
                                        Cast::from(3.0),
                                        Cast::from(5.0)
                                    ),
                                    Cast::from(3.0)
                                ),
                                SetOperation::SymmetricDifference
                            ),
                            SetOperation::Complement
                        ),
                    ),
                    Box::new(
                        Vacuum::new()
                    ),
                    Some(
                        Box::new(
                            ComposableSurface {
                                reflection_ratio: reflection_ratio_uniform(Cast::from(0.5)),
                                reflection_direction: reflection_direction_specular(),
                                surface_color: Box::new(universe::d3::entity::surface::get_surface_color_test),
                            }
                        )
                    )
                )
            )
        );
        // entities.push(Box::new(Entity3Impl::new(
        //             Box::new(ComposableShape::new(
        //                 HalfSpace3::from_point(
        //                     // Plane3::from_equation(Cast::from(1.0),
        //                     //                       Cast::from(0.5),
        //                     //                       Cast::from(0.0),
        //                     //                       Cast::from(-1.0)),
        //                     Plane3::new(
        //                         &Point3::new(Cast::from(0.0),
        //                         Cast::from(3.0),
        //                         Cast::from(3.0)),
        //                         &Vector3::new(Cast::from(0.0),
        //                         Cast::from(3.0),
        //                         Cast::from(1.0)),
        //                         &Vector3::new(Cast::from(1.0),
        //                         Cast::from(0.0),
        //                         Cast::from(0.0)),
        //                     ),
        //                     &Point3::new(Cast::from(0.0),
        //                     Cast::from(0.0),
        //                     Cast::from(-100.0))
        //                 ),
        //                 // HalfSpace3::from_point(
        //                 //     // Plane3::from_equation(Cast::from(1.0),
        //                 //     //                       Cast::from(0.5),
        //                 //     //                       Cast::from(0.0),
        //                 //     //                       Cast::from(-1.0)),
        //                 //     Plane3::new(
        //                 //         &Point3::new(Cast::from(0.0),
        //                 //         Cast::from(3.0),
        //                 //         Cast::from(3.0)),
        //                 //         &Vector3::new(Cast::from(0.0),
        //                 //         Cast::from(3.0),
        //                 //         Cast::from(1.5)),
        //                 //         &Vector3::new(Cast::from(1.0),
        //                 //         Cast::from(0.0),
        //                 //         Cast::from(0.0)),
        //                 //     ),
        //                 //     &Point3::new(Cast::from(0.0),
        //                 //     Cast::from(0.0),
        //                 //     Cast::from(-100.0))
        //                 // ),
        //                 HalfSpace3::from_point(
        //                     // Plane3::from_equation(Cast::from(1.0),
        //                     //                       Cast::from(0.5),
        //                     //                       Cast::from(0.0),
        //                     //                       Cast::from(-1.0)),
        //                     Plane3::new(
        //                         &Point3::new(Cast::from(0.0),
        //                         Cast::from(3.0),
        //                         Cast::from(3.0)),
        //                         &Vector3::new(Cast::from(0.0),
        //                         Cast::from(3.0),
        //                         Cast::from(-1.0)),
        //                         &Vector3::new(Cast::from(1.0),
        //                         Cast::from(0.0),
        //                         Cast::from(0.0)),
        //                     ),
        //                     &Point3::new(Cast::from(0.0),
        //                     Cast::from(0.0),
        //                     Cast::from(100.0))
        //                 ),
        //                 SetOperation::Union
        //                 )),
        //                 Box::new(Vacuum::new()),
        //                 Some(Box::new(ComposableSurface {
        //                     reflection_ratio: get_reflection_ratio_test,
        //                     reflection_direction: get_reflection_direction_test,
        //                     surface_color: get_surface_color_test,
        //                 }))
        //             )));
        entities.push(Box::new(Void::new_with_vacuum()));
    }

    {
        let intersectors = universe.intersectors_mut();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(universe::d3::entity::shape::intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere3::<F>::id_static()),
                            Box::new(Sphere3::<F>::intersect_in_vacuum));
        intersectors.insert((Vacuum::id_static(), Plane3::<F>::id_static()),
                            Box::new(Plane3::<F>::intersect_in_vacuum));
        intersectors.insert((Vacuum::id_static(), HalfSpace3::<F>::id_static()),
                            Box::new(HalfSpace3::<F>::intersect_in_vacuum));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<F, Point3<F>, Vector3<F>>::id_static()),
                    Box::new(ComposableShape::<F, Point3<F>, Vector3<F>>::intersect_in_vacuum));
    }

    {
        let mut transitions = universe.transitions_mut();
        transitions.insert((Vacuum::id_static(), Vacuum::id_static()),
                           Box::new(universe::d3::entity::material::transition_vacuum_vacuum));
    }

    let simulation = Simulation::builder()
        .universe(universe)
        .build();

    simulation.start();
}
