use palette::Rgba;
use std::any::Any;
use std::collections::HashMap;
use util::CustomFloat;
use universe::Environment;
use universe::Universe;
use universe::d3::Universe3;
use json;
use json::JsonValue;
use json::iterators::Members;
use json::object::Object;
use universe::entity::*;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::entity::surface::*;
use universe::d3::entity::Entity3Impl;
use universe::d3::entity::surface::*;
use universe::d3::entity::shape::*;
use na::Point3;
use na::Vector3;
use util::JsonFloat;
use image;
use meval::Expr;

pub type Deserializer<T> = Fn(&JsonValue, &Parser) -> Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    NoDeserializer {
        key: String,
    },
    TypeMismatch {
        key: String,
        value: Box<Any + 'static>,
        json: JsonValue,
    },
    SyntaxError {
        key: String,
        error: json::Error,
    },
    InvalidStructure {
        description: String,
        json: JsonValue,
    },
    MissingType {
        type_str: String,
    },
}

pub struct Parser {
    pub deserializers: HashMap<&'static str, Box<Deserializer<Box<Any>>>>,
}

impl Parser {
    pub fn empty() -> Self {
        Parser { deserializers: HashMap::new() }
    }

    #[allow(unused_variables)]
    pub fn default<F: CustomFloat>() -> Self {
        let mut parser = Self::empty();

        {
            let deserializers = &mut parser.deserializers;

            // General

            deserializers.insert("Point3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let mut members: Members = json.members();

                                     Ok(
                                         Box::new(
                                             Point3::<F>::new(
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap(),
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap(),
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap()
                                             )
                                         )
                                     )
                                 }));

            deserializers.insert("Vector3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let mut members: Members = json.members();

                                     Ok(
                                         Box::new(
                                             Vector3::<F>::new(
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap(),
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap(),
                                                 <F as JsonFloat>::float_from_json(members.next().unwrap()).unwrap()
                                             )
                                         )
                                     )
                                 }));

            deserializers.insert("Rgba::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                Ok(Box::new(Rgba::new(try!(<F as JsonFloat>::float_from_json(try!(members.next()
                                              .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the red component of an `Rgba` color."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                                          .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the red component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                      try!(<F as JsonFloat>::float_from_json(try!(members.next()
                                              .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the green component of an `Rgba` color."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                                          .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the green component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                      try!(<F as JsonFloat>::float_from_json(try!(members.next()
                                              .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the blue component of an `Rgba` color."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                                          .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the blue component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                      try!(<F as JsonFloat>::float_from_json(try!(members.next()
                                              .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the alpha component of an `Rgba` color."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                                          .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the alpha component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })))))
            }));

            deserializers.insert("Rgba::new_u8",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                Ok(Box::new(Rgba::<F>::new_u8(try!(try!(members.next().ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Missing the red component of an `Rgba` color.".to_owned(),
                            json: json.clone(),
                        }
                    }))
                                                  .as_u8()
                                                  .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the red component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                              try!(try!(members.next().ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Missing the green component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    }))
                                                  .as_u8()
                                                  .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the green component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                              try!(try!(members.next().ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Missing the blue component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    }))
                                                  .as_u8()
                                                  .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the blue component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })),
                                              try!(try!(members.next().ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Missing the alpha component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    }))
                                                  .as_u8()
                                                  .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the alpha component of an `Rgba` color."
                                .to_owned(),
                            json: json.clone(),
                        }
                    })))))
            }));

            // Entities

            deserializers.insert("Void::new_with_vacuum",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let result: Box<Box<Entity<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(Void::new_with_vacuum()));

                                     Ok(result)
                                 }));

            deserializers.insert("Entity3Impl::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let shape: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the `Shape` parameter.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let material: Box<Box<Material<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the `Material` parameter.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let surface: Option<Box<Surface<F, Point3<F>, Vector3<F>>>> =
                    if let Some(surface_json) = members.next() {
                        Some(*try!(parser.deserialize_constructor(surface_json)))
                    } else {
                        None
                    };

                let result: Box<Box<Entity<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(Entity3Impl::new(*shape, *material, surface)));

                Ok(result)
            }));

            // Shapes

            deserializers.insert("VoidShape",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     Ok(Box::new(VoidShape::new()))
                                 }));

            deserializers.insert("ComposableShape::of",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let shapes_json = try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing an array of `Shapes` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        })).members();

                let operation: Box<SetOperation> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `SetOperation` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let mut shapes: Vec<Box<Shape<F, Point3<F>, Vector3<F>>>> = Vec::new();

                for shape_json in shapes_json {
                    let shape: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                        try!(parser.deserialize_constructor(shape_json)
                                   .or_else(|err| Err(ParserError::InvalidStructure {
                                       description: "Could not parse a `Shape`.".to_owned(),
                                       json: shape_json.clone(),
                                   })));
                    shapes.push(*shape);
                }

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(ComposableShape::of(shapes, *operation)));

                Ok(result)
            }));

            deserializers.insert("SetOperation",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let name = try!(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `String` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        })).as_str().ok_or_else(|| ParserError::InvalidStructure {
                            description: "The first argument must be a `String`.".to_owned(),
                            json: json.clone(),
                        }));

                let result = match name {
                    "Union" => SetOperation::Union,
                    "Intersection" => SetOperation::Intersection,
                    "Complement" => SetOperation::Complement,
                    "SymmetricDifference" => SetOperation::SymmetricDifference,
                    _ => return Err(ParserError::InvalidStructure {
                        description: format!("Invalid `SetOperation`: \"{}\"", name),
                        json: json.clone(),
                    }),
                };

                Ok(Box::new(result))
            }));

            deserializers.insert("Sphere3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let center: Box<Point3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the `radius` argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let radius: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the `radius` argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the `radius`.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(Sphere::<F, Point3<F>, Vector3<F>>::new(*center, radius)));

                Ok(result)
            }));

            deserializers.insert("Plane3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let normal: Box<Vector3<F>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Vector3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let constant: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a floating point number as the second \
                                              argument."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the constant (second argument)."
                                .to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(Plane::new(*normal, constant)));

                Ok(result)
            }));

            deserializers.insert("Plane3::new_with_point",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let normal: Box<Vector3<F>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Vector3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let point: Box<Point3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Point3` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(Plane::new_with_point(*normal, &*point)));

                Ok(result)
            }));

            deserializers.insert("Plane3::new_with_vectors",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let vector_a: Box<Vector3<F>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Vector3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let vector_b: Box<Vector3<F>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Vector3` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let point: Box<Point3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Point3` as the third argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(Plane::new_with_vectors(&*vector_a, &*vector_b, &*point)));

                Ok(result)
            }));

            deserializers.insert("HalfSpace3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let plane: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Plane3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let plane: Plane<F, Point3<F>, Vector3<F>> = *try!(<Shape<F, Point3<F>, Vector3<F>>>::downcast(*plane)
                    .or_else(|err| Err(ParserError::InvalidStructure {
                        description: "Invalid type, expected a `Plane3`.".to_owned(),
                        json: json.clone(),
                    })));
                let signum: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a floating point number as the second \
                                              argument."
                                    .to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the sign (second argument).".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(HalfSpace::new(plane, signum)));

                Ok(result)
            }));

            deserializers.insert("HalfSpace3::new_with_point",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let plane: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Plane3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let plane: Plane<F, Point3<F>, Vector3<F>> = *try!(<Shape<F, Point3<F>, Vector3<F>>>::downcast(*plane)
                    .or_else(|err| Err(ParserError::InvalidStructure {
                        description: "Invalid type, expected a `Plane3`.".to_owned(),
                        json: json.clone(),
                    })));
                let point: Box<Point3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Point3` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(HalfSpace::new_with_point(plane, &point)));

                Ok(result)
            }));

            deserializers.insert("HalfSpace3::cuboid",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();

                let center: Box<Point3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Point3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let abc: Box<Vector3<F>> = try!(parser.deserialize_constructor(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing `Vector3` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<Shape<F, Point3<F>, Vector3<F>>>> =
                    Box::new(Box::new(cuboid(*center, *abc)));

                Ok(result)
            }));

            // Materials

            deserializers.insert("Vacuum",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let result: Box<Box<Material<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(Vacuum::new()));

                                     Ok(result)
                                 }));

            deserializers.insert("Vacuum::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let result: Box<Box<Material<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(Vacuum::new()));

                                     Ok(result)
                                 }));
            
            deserializers.insert("ComponentTransformation",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The JSON value must be an object.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let array = try!(object.get("expressions").ok_or_else(|| ParserError::InvalidStructure {
                                         description: "Could not find `expressions`.".to_owned(),
                                         json: json.clone(),
                                     }));
                                     let array: &Vec<JsonValue> = if let JsonValue::Array(ref object) = *array {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The `expressions` field must be an array.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let mut expressions: Vec<ComponentTransformationExpr> = Vec::new();

                                     for json in array {
                                         let object: &Object = if let JsonValue::Object(ref object) = *json {
                                             object
                                         } else {
                                             return Err(ParserError::InvalidStructure {
                                                 description: "The JSON value must be an object.".to_owned(),
                                                 json: json.clone(),
                                             });
                                         };

                                         let expression = try!(try!(object.get("expression").ok_or_else(|| ParserError::InvalidStructure {
                                                          description: "The `expression` field is missing.".to_owned(),
                                                          json: json.clone(),
                                                      })).as_str().ok_or_else(|| ParserError::InvalidStructure {
                                                          description: "Expected a string as the function.".to_owned(),
                                                          json: json.clone(),
                                                      }));
                                         let expression = try!(Expr::from_str(expression).or_else(|err| Err(ParserError::InvalidStructure {
                                             description: "Invalid component transformation expression.".to_owned(),
                                             json: json.clone(),
                                         })));

                                         let inverse_expression = try!(try!(object.get("inverse_expression").ok_or_else(|| ParserError::InvalidStructure {
                                                          description: "The `inverse_expression` field is missing.".to_owned(),
                                                          json: json.clone(),
                                                      })).as_str().ok_or_else(|| ParserError::InvalidStructure {
                                                          description: "Expected a string as the inverse function.".to_owned(),
                                                          json: json.clone(),
                                                      }));
                                         let inverse_expression = try!(Expr::from_str(inverse_expression).or_else(|err| Err(ParserError::InvalidStructure {
                                             description: "Invalid component transformation inverse expression.".to_owned(),
                                             json: json.clone(),
                                         })));

                                         expressions.push(ComponentTransformationExpr {
                                             expression: expression,
                                             inverse_expression: inverse_expression,
                                         });
                                     }

                                     let result: Box<Box<LinearTransformation<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(ComponentTransformation {
                                             expressions: expressions,
                                         }));

                                     Ok(result)
                                 }));
            
            deserializers.insert("LinearSpace",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The JSON value must be an object.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let legend = try!(try!(object.get("legend").ok_or_else(|| ParserError::InvalidStructure {
                                                      description: "The `legend` field is missing.".to_owned(),
                                                      json: json.clone(),
                                                  })).as_str().ok_or_else(|| ParserError::InvalidStructure {
                                                      description: "Expected a string as the legend.".to_owned(),
                                                      json: json.clone(),
                                                  }));

                                     let array = try!(object.get("transformations").ok_or_else(|| ParserError::InvalidStructure {
                                         description: "Could not find `transformations`.".to_owned(),
                                         json: json.clone(),
                                     }));
                                     let array: &Vec<JsonValue> = if let JsonValue::Array(ref object) = *array {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The `transformations` field must be an array.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let mut transformations: Vec<Box<LinearTransformation<F, Point3<F>, Vector3<F>>>> = Vec::new();

                                     for json in array {
                                         let transformation = try!(parser.deserialize_constructor::<Box<LinearTransformation<F, Point3<F>, Vector3<F>>>>(json));
                                         transformations.push(*transformation);
                                     }

                                     let result: Box<Box<Material<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(LinearSpace {
                                             legend: legend.to_owned(),
                                             transformations: transformations,
                                         }));

                                     Ok(result)
                                 }));

            // Surfaces

            deserializers.insert("uv_sphere",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let center = try!(parser.deserialize_constructor::<Point3<F>>(try!(members.next()
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Center location not specified.".to_owned(),
                            json: json.clone(),
                        }
                    }))));

                Ok(Box::new(uv_sphere(*center)))
            }));

            deserializers.insert("texture_image",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let path = try!(try!(members.next().ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Missing a path to the image as the first argument."
                                .to_owned(),
                            json: json.clone(),
                        }
                    }))
                    .as_str()
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "The `texture_image` must be a string.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<Texture<F>>> = Box::new(texture_image(image::open(path)
                    .expect(&format!("Could not find texture `{}`.", path))));

                Ok(result)
            }));

            deserializers.insert("MappedTextureImpl3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let mut members: Members = json.members();
                                     let uvfn = try!(parser.deserialize_constructor::<Box<UVFn<F, Point3<F>>>>(
                                                    try!(members.next().ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Missing a `UVFn` as the first argument.".to_owned(),
                                                        json: json.clone(),
                                                    })
                                                )));
                                     let texture = try!(parser.deserialize_constructor::<Box<Texture<F>>>(
                                                    try!(members.next().ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Missing a `Texture` as the second argument.".to_owned(),
                                                        json: json.clone(),
                                                    })
                                                )));

                                     let result: Box<Box<MappedTexture<F, Point3<F>, Vector3<F>>>> = Box::new(Box::new(MappedTextureImpl::new(
                                                 *uvfn,
                                                 *texture
                                             )));

                                     Ok(result)
                                 }));

            deserializers.insert("ComposableSurface",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The JSON value must be an object.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let reflection_ratio: Box<Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>>> = 
                                         try!(parser.deserialize_constructor::<Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>>>(
                                                 try!(object.get("reflection_ratio").ok_or_else(|| ParserError::InvalidStructure {
                                                     description: "The `reflection_ratio` field is missing.".to_owned(),
                                                     json: json.clone(),
                                                 }))
                                             ));

                                     let reflection_direction: Box<Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>>> = 
                                         try!(parser.deserialize_constructor::<Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>>>(
                                                 try!(object.get("reflection_direction").ok_or_else(|| ParserError::InvalidStructure {
                                                     description: "The `reflection_direction` field is missing.".to_owned(),
                                                     json: json.clone(),
                                                 }))
                                             ));

                                     let threshold_direction: Box<Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>>> = 
                                         try!(parser.deserialize_constructor::<Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>>>(
                                                 try!(object.get("threshold_direction").ok_or_else(|| ParserError::InvalidStructure {
                                                     description: "The `threshold_direction` field is missing.".to_owned(),
                                                     json: json.clone(),
                                                 }))
                                             ));

                                     let surface_color: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> = 
                                         try!(parser.deserialize_constructor::<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>>(
                                                 try!(object.get("surface_color").ok_or_else(|| ParserError::InvalidStructure {
                                                     description: "The `surface_color` field is missing.".to_owned(),
                                                     json: json.clone(),
                                                 }))
                                             ));

                                     let result: Box<Box<Surface<F, Point3<F>, Vector3<F>>>> =
                                         Box::new(Box::new(ComposableSurface {
                                             reflection_ratio: *reflection_ratio,
                                             reflection_direction: *reflection_direction,
                                             threshold_direction: *threshold_direction,
                                             surface_color: *surface_color
                                         }));

                                     Ok(result)
                                 }));

            deserializers.insert("blend_function_ratio",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let ratio: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `ratio` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the ratio.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<BlendFunction<F>>> =
                    Box::new(blend_function_ratio(ratio));

                Ok(result)
            }));

            macro_rules! deserialize_blending_functions {
                (
                    $($name:ident),+
                ) => {
                    $(
                        deserializers.insert(
                            concat!(
                                "blend_function_",
                                stringify!($name)
                            ), Box::new(|json: &JsonValue, parser: &Parser| {
                                let result: Box<Box<BlendFunction<F>>> =
                                    Box::new(concat_idents!(blend_function_, $name)());
                                Ok(result)
                            })
                        );
                    )+
                }
            }

            deserialize_blending_functions!(over, inside, outside, atop, xor, plus, multiply,
                                            screen, overlay, darken, lighten, dodge, burn,
                                            hard_light, soft_light, difference, exclusion);

            deserializers.insert("surface_color_blend",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let source: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor::<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>>(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `SurfaceColorProvider` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let destination: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    try!(parser.deserialize_constructor::<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>>(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `SurfaceColorProvider` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));
                let blend_function: Box<Box<BlendFunction<F>>> =
                    try!(parser.deserialize_constructor::<Box<BlendFunction<F>>>(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `BlendFunction` as the third argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(surface_color_blend(*source, *destination, *blend_function));

                Ok(result)
            }));

            deserializers.insert("surface_color_perlin_hue_seed",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let seed: u32 = try!(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `seed` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))
                    .as_u32()
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the seed.".to_owned(),
                            json: json.clone(),
                        }
                    }));
                let size: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `size` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the size.".to_owned(),
                            json: json.clone(),
                        }
                    }));
                let speed: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `speed` as the third argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the speed.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(surface_color_perlin_hue_seed(seed, size, speed));

                Ok(result)
            }));

            deserializers.insert("surface_color_illumination_directional",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let direction: Box<Vector3<F>> =
                    try!(parser.deserialize_constructor::<Vector3<F>>(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `Vector3` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(surface_color_illumination_directional(*direction));

                Ok(result)
            }));

            deserializers.insert("surface_color_perlin_hue_random",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let size: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `size` as the second argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the size.".to_owned(),
                            json: json.clone(),
                        }
                    }));
                let speed: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `speed` as the third argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the speed.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(surface_color_perlin_hue_random(size, speed));

                Ok(result)
            }));

            deserializers.insert("reflection_ratio_uniform",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let ratio: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `ratio` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the ratio.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(reflection_ratio_uniform(ratio));

                Ok(result)
            }));

            deserializers.insert("reflection_direction_specular",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let result: Box<Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>>>
                                         = Box::new(reflection_direction_specular());

                                     Ok(result)
                                 }));

            deserializers.insert("threshold_direction_snell",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                    let mut members: Members = json.members();
                                    let refractive_index: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                                            .ok_or_else(|| {
                                                ParserError::InvalidStructure {
                                                    description: "Missing a refractive index as the first argument.".to_owned(),
                                                    json: json.clone(),
                                                }
                                            })))
                                        .ok_or_else(|| {
                                            ParserError::InvalidStructure {
                                                description: "Could not parse the refractive index.".to_owned(),
                                                json: json.clone(),
                                            }
                                        }));
                                     let result: Box<Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>>>
                                         = Box::new(threshold_direction_snell(refractive_index));

                                     Ok(result)
                                 }));

            deserializers.insert("threshold_direction_identity",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let result: Box<Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>>>
                                         = Box::new(threshold_direction_identity());

                                     Ok(result)
                                 }));

            deserializers.insert("surface_color_uniform",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let color: Box<Rgba<F>> =
                    try!(parser.deserialize_constructor::<Rgba<F>>(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing a `Rgba` as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        }))));

                let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(surface_color_uniform(*color));

                Ok(result)
            }));

            deserializers.insert("reflection_ratio_fresnel",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let mut members: Members = json.members();
                let refractive_index_inside: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the inside refractive index as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the inside refractive index.".to_owned(),
                            json: json.clone(),
                        }
                    }));
                let refractive_index_outside: F = try!(<F as JsonFloat>::float_from_json(try!(members.next()
                        .ok_or_else(|| {
                            ParserError::InvalidStructure {
                                description: "Missing the outside refractive index as the first argument.".to_owned(),
                                json: json.clone(),
                            }
                        })))
                    .ok_or_else(|| {
                        ParserError::InvalidStructure {
                            description: "Could not parse the outside refractive index.".to_owned(),
                            json: json.clone(),
                        }
                    }));

                let result: Box<Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>>> =
                    Box::new(reflection_ratio_fresnel(refractive_index_inside,
                                                      refractive_index_outside));

                Ok(result)
            }));

            deserializers.insert("surface_color_texture",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let mut members: Members = json.members();
                                     let mapped_texture: Box<Box<MappedTexture<F, Point3<F>, Vector3<F>>>>
                                         = try!(parser.deserialize_constructor::<Box<MappedTexture<F, Point3<F>, Vector3<F>>>>(
                                                try!(members.next().ok_or_else(|| ParserError::InvalidStructure {
                                                    description: "Missing a `MappedTexture` as the first argument.".to_owned(),
                                                    json: json.clone(),
                                                })
                                            )));

                                     let result: Box<Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>>
                                         = Box::new(surface_color_texture(*mapped_texture));

                                     Ok(result)
                                 }));

            // Environments

            deserializers.insert("Environment",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                let object: &Object = if let JsonValue::Object(ref object) = *json {
                    object
                } else {
                    return Err(ParserError::InvalidStructure {
                        description: "The JSON value must be an object.".to_owned(),
                        json: json.clone(),
                    });
                };

                let result: Box<Box<Environment<F>>> =
                    try!(parser.deserialize_constructor::<Box<Environment<F>>>(json));

                Ok(result)
            }));

            deserializers.insert("Universe3",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The JSON value must be an object.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let mut universe = Universe3::<F>::default();

                                     {
                                         let mut entities = universe.entities_mut();

                                         let json_entities = if let Some(&JsonValue::Array(ref json_entities))
                                             = object.get("entities") {
                                             json_entities
                                         } else {
                                             return Err(ParserError::InvalidStructure {
                                                 description: "The `entities` must be an array.".to_owned(),
                                                 json: json.clone(),
                                             });
                                         };

                                         for json_entity in json_entities {
                                             let entity = try!(parser.deserialize_constructor::<Box<Entity<F, Point3<F>, Vector3<F>>>>(
                                                     json_entity
                                                 ));

                                             entities.push(*entity);
                                         }
                                     }

                                     {
                                         let background = try!(parser.deserialize_constructor::<Box<MappedTexture<F, Point3<F>, Vector3<F>>>>(
                                                 try!(object.get("background")
                                                      .ok_or_else(|| ParserError::InvalidStructure {
                                                          description: "The `background` field is missing.".to_owned(),
                                                          json: json.clone(),
                                                      }))
                                             ));

                                         universe.set_background(*background);
                                     }

                                     let result: Box<Box<Environment<F>>> =
                                         Box::new(Box::new(universe));

                                     Ok(result)
                                 }));
        }

        parser
    }

    pub fn deserializer(&self, key: &str) -> Result<&Deserializer<Box<Any>>, ParserError> {
        let option = self.deserializers.get(key);

        if option.is_some() {
            Ok(option.unwrap().as_ref())
        } else {
            Err(ParserError::NoDeserializer { key: key.to_owned() })
        }
    }

    pub fn deserialize<T: Any>(&self, key: &str, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let deserializer = try!(self.deserializer(key));
        let result = try!(deserializer(json, &self)).downcast::<T>();

        match result {
            Ok(value) => Ok(value),
            Err(original_value) => {
                Err(ParserError::TypeMismatch {
                    key: key.to_owned(),
                    value: original_value,
                    json: json.clone(),
                })
            }
        }
    }

    pub fn deserialize_constructor<T: Any>(&self, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let entries: Vec<(&str, &JsonValue)> = json.entries().collect();

        if entries.len() == 1 {
            let (constructor_key, constructor_value) = entries[0];
            self.deserialize::<T>(constructor_key, constructor_value)
        } else {
            Err(ParserError::InvalidStructure {
                description: "A constructor must be an object containing a single key pointing to \
                              either an object or an array."
                    .to_owned(),
                json: json.clone(),
            })
        }
    }

    pub fn parse<T: Any>(&self, key: &str, json: &str) -> Result<Box<T>, ParserError> {
        let value = json::parse(json);

        match value {
            Ok(value) => self.deserialize::<T>(key, &value),
            Err(err) => {
                Err(ParserError::SyntaxError {
                    key: key.to_owned(),
                    error: err,
                })
            }
        }
    }
}
