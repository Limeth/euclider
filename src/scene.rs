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

macro_rules! deserializer {
    (
        @try_unwrap
        option: $option:expr,
        type_name: $type_name:expr,
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {
        try!($option
            .ok_or_else(|| {
                ParserError::TypeMismatch {
                    description: format! {
                        "Expected `{}`, could not parse from `{:?}`.",
                        $type_name,
                        $json,
                    },
                    parent_json: $parent_json.clone(),
                }
            }))
    };

    (
        @deserialize [ F ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: <F as JsonFloat>::float_from_json(json),
            type_name: "floating point number",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ &str ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_str(),
            type_name: "string",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ String ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;

        {
            deserializer! {
                @try_unwrap
                option: json.as_str(),
                type_name: "string",
                parent_json: $parent_json,
                parser: $parser,
                json: json
            }
        }.to_string()
    }};

    (
        @deserialize [ Number ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_number(),
            type_name: "number",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ f64 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_f64(),
            type_name: "64-bit floating point number",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ f32 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_f32(),
            type_name: "32-bit floating point number",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ u64 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_u64(),
            type_name: "unsigned 64-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ u32 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_u32(),
            type_name: "unsigned 32-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ u16 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_u16(),
            type_name: "unsigned 16-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ u8 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_u8(),
            type_name: "unsigned 8-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ usize ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_usize(),
            type_name: "unsigned pointer-sized integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ i64 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_i64(),
            type_name: "64-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ i32 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_i32(),
            type_name: "32-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ i16 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_i16(),
            type_name: "16-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ i8 ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_i8(),
            type_name: "8-bit integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ isize ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_isize(),
            type_name: "pointer-sized integer",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ bool ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        deserializer! {
            @try_unwrap
            option: json.as_bool(),
            type_name: "boolean value",
            parent_json: $parent_json,
            parser: $parser,
            json: json
        }
    }};

    (
        @deserialize [ $($item_type:tt)+ ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {
        *try!($parser.deserialize_constructor::<$($item_type)+>($json))
    };

    (
        @iterator_next [ $($item_type:tt)+ ]
        parent_json: $parent_json:expr,
        iterator: $iterator:expr,
    ) => {
        try!(
            $iterator.next()
                     .ok_or_else(|| {
                ParserError::MissingField {
                    description: format! {
                        concat! {
                            "Missing field of type {} in {:?}.",
                            " To fix this, add the field at the end of the array."
                        },
                        stringify!($($item_type)+),
                        $parent_json
                    },
                    parent_json: $parent_json.clone(),
                }
            })
        )
    };

    (
        @object_get [ $($item_type:tt)+ ]
        parent_json: $parent_json:expr,
        object: $object:expr,
        key: $key:expr
    ) => {
        try!(
            $object.get(stringify!($key))
                   .ok_or_else(|| {
                ParserError::MissingField {
                    description: format! {
                        "Missing field of type {} with key {} in {:?}.",
                        stringify!($($item_type)+),
                        stringify!($key),
                        $parent_json
                    },
                    parent_json: $parent_json.clone(),
                }
            })
        )
    };

    (
        @construct [ $return_type:ty ]
        constructor: $constructor:block
    ) => {{
        let result: $return_type = $constructor;
        let result: Box<Any> = Box::new(result);

        Ok(result)
    }};

    (
        $([ $field_name:ident : $($field_type:tt)+ ])* -> $return_type:ty
        $constructor:block
    ) => {
        |parent_json: &JsonValue, json: &JsonValue, parser: &Parser| -> Result<Box<Any>, ParserError> {
            match *json {
                JsonValue::Object(ref json) => {
                    $(
                        let $field_name: $($field_type)+ = deserializer! {
                            @deserialize [$($field_type)+]
                            parent_json: parent_json,
                            parser: parser,
                            json: deserializer! {
                                @object_get [$($field_type)+]
                                parent_json: parent_json,
                                object: json,
                                key: $field_name
                            }
                        };
                    )*

                    deserializer! {
                        @construct [ $return_type ]
                        constructor: $constructor
                    }
                }
                JsonValue::Array(ref json) => {
                    let mut __iterator = json.iter();
                    $(
                        let $field_name: $($field_type)+ = deserializer! {
                            @deserialize [$($field_type)+]
                            parent_json: parent_json,
                            parser: parser,
                            json: deserializer! {
                                @iterator_next [$($field_type)+]
                                parent_json: parent_json,
                                iterator: __iterator,
                            }
                        };
                    )*

                    deserializer! {
                        @construct [ $return_type ]
                        constructor: $constructor
                    }
                }
                _ => Err(ParserError::InvalidConstructor {
                    description: format! {
                        concat! {
                            "The constructor data may only be an array or an object,",
                            " received {:?} instead."
                        },
                        json
                    },
                    parent_json: parent_json.clone(),
                })
            }
        }
    };
}

/// Fields:
/// - Parent json object for printing on failure
/// - The json value to parse
/// - A parser with the deserializers
pub type Deserializer<T> = Fn(&JsonValue, &JsonValue, &Parser) -> Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    NoDeserializer {
        key: String,
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
    InvalidConstructor {
        description: String,
        parent_json: JsonValue,
    },
    MissingField {
        description: String,
        parent_json: JsonValue,
    },
    TypeMismatch {
        description: String,
        parent_json: JsonValue,
    },
    CustomError {
        description: String,
    }
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

            macro_rules! add_deserializer {
                (
                    $constructor_name:expr,
                    $($rest:tt)+
                ) => {
                    deserializers.insert(
                        $constructor_name,
                        Box::new(deserializer! {
                            $($rest)+
                        })
                    );
                }
            }

            // General

            add_deserializer! {
                "Point3::new",
                [x: F] [y: F] [z: F] -> Point3<F> {
                    Point3::new(x, y, z)
                }
            };

            add_deserializer! {
                "Vector3::new",
                [x: F] [y: F] [z: F] -> Vector3<F> {
                    Vector3::new(x, y, z)
                }
            };

            add_deserializer! {
                "Rgba::new",
                [r: F] [g: F] [b: F] [a: F] -> Rgba<F> {
                    Rgba::<F>::new(r, g, b, a)
                }
            };

            add_deserializer! {
                "Rgba::new_u8",
                [r: u8] [g: u8] [b: u8] [a: u8] -> Rgba<F> {
                    Rgba::<F>::new_u8(r, g, b, a)
                }
            };

            // Entities

            add_deserializer! {
                "Void::new",
                [material: Box<Material<F, Point3<F>, Vector3<F>>>]
                -> Box<Entity<F, Point3<F>, Vector3<F>>> {
                    Box::new(Void::<F, Point3<F>, Vector3<F>>::new(material))
                }
            };

            add_deserializer! {
                "Void::new_with_vacuum",
                -> Box<Entity<F, Point3<F>, Vector3<F>>> {
                    Box::new(Void::<F, Point3<F>, Vector3<F>>::new_with_vacuum())
                }
            };

            add_deserializer! {
                "Entity3Impl::new_with_surface",
                [shape: Box<Shape<F, Point3<F>, Vector3<F>>>]
                [material: Box<Material<F, Point3<F>, Vector3<F>>>]
                [surface: Box<Surface<F, Point3<F>, Vector3<F>>>]
                -> Box<Entity<F, Point3<F>, Vector3<F>>> {
                    Box::new(Entity3Impl::new_with_surface(shape, material, surface))
                }
            }

            add_deserializer! {
                "Entity3Impl::new_without_surface",
                [shape: Box<Shape<F, Point3<F>, Vector3<F>>>]
                [material: Box<Material<F, Point3<F>, Vector3<F>>>]
                -> Box<Entity<F, Point3<F>, Vector3<F>>> {
                    Box::new(Entity3Impl::new_without_surface(shape, material))
                }
            }

            // Shapes

            // TODO merge those two together
            add_deserializer! {
                "VoidShape",
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(VoidShape::new())
                }
            }

            add_deserializer! {
                "VoidShape::new",
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(VoidShape::new())
                }
            }

            // TODO add the ability to deserialize `Vec`s
            deserializers.insert("ComposableShape::of",
                                 Box::new(|parent_json: &JsonValue, json: &JsonValue, parser: &Parser| {
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

            add_deserializer! {
                "SetOperation",
                [name: &str] -> SetOperation {
                    match name {
                        "Union" => SetOperation::Union,
                        "Intersection" => SetOperation::Intersection,
                        "Complement" => SetOperation::Complement,
                        "SymmetricDifference" => SetOperation::SymmetricDifference,
                        _ => return Err(ParserError::CustomError {
                            description: format!("Invalid `SetOperation`: \"{}\"", name),
                        }),
                    }
                }
            }

            add_deserializer! {
                "Sphere3::new",
                [center: Point3<F>] [radius: F]
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(Sphere::<F, Point3<F>, Vector3<F>>::new(center, radius))
                }
            }

            add_deserializer! {
                "Plane3::new",
                [normal: Vector3<F>] [constant: F]
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(Plane::new(normal, constant))
                }
            }

            add_deserializer! {
                "Plane3::new_with_point",
                [normal: Vector3<F>] [point: Point3<F>]
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(Plane::new_with_point(normal, &point))
                }
            }

            add_deserializer! {
                "Plane3::new_with_vectors",
                [first: Vector3<F>] [second: Vector3<F>] [point: Point3<F>]
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(Plane::new_with_vectors(&first, &second, &point))
                }
            }

            add_deserializer! {
                "HalfSpace3::new",
                [plane: Box<Shape<F, Point3<F>, Vector3<F>>>]
                [sign: F] -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    let plane: Plane<F, Point3<F>, Vector3<F>>
                        = *try!(<Shape<F, Point3<F>, Vector3<F>>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Plane3`.".to_string(),
                            })));
                    Box::new(HalfSpace::new(plane, sign))
                }
            }

            add_deserializer! {
                "HalfSpace3::new_with_point",
                [plane: Box<Shape<F, Point3<F>, Vector3<F>>>]
                [point: Point3<F>] -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    let plane: Plane<F, Point3<F>, Vector3<F>>
                        = *try!(<Shape<F, Point3<F>, Vector3<F>>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Plane3`.".to_string(),
                            })));
                    Box::new(HalfSpace::new_with_point(plane, &point))
                }
            }

            add_deserializer! {
                "HalfSpace3::cuboid",
                [center: Point3<F>] [dimensions: Vector3<F>]
                -> Box<Shape<F, Point3<F>, Vector3<F>>> {
                    Box::new(cuboid(center, dimensions))
                }
            }

            // Materials

            // TODO merge those two together
            add_deserializer! {
                "Vacuum",
                -> Box<Material<F, Point3<F>, Vector3<F>>> {
                    Box::new(Vacuum::new())
                }
            }

            add_deserializer! {
                "Vacuum::new",
                -> Box<Material<F, Point3<F>, Vector3<F>>> {
                    Box::new(Vacuum::new())
                }
            }

            // TODO
            deserializers.insert("ComponentTransformation",
                                 Box::new(|parent_json: &JsonValue, json: &JsonValue, parser: &Parser| {
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

            // TODO
            deserializers.insert("LinearSpace",
                                 Box::new(|parent_json: &JsonValue, json: &JsonValue, parser: &Parser| {
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

            add_deserializer! {
                "uv_sphere",
                [center: Point3<F>] -> Box<UVFn<F, Point3<F>>> {
                    uv_sphere(center)
                }
            }

            add_deserializer! {
                "texture_image",
                [path: &str] -> Box<Texture<F>> {
                    let data = try!(image::open(path)
                        .map_err(|_| ParserError::CustomError {
                            description: format!("Could not load texture `{}`", path),
                        }));

                    texture_image(data)
                }
            }

            add_deserializer! {
                "MappedTextureImpl3::new",
                [uvfn: Box<UVFn<F, Point3<F>>>]
                [texture: Box<Texture<F>>]
                -> Box<MappedTexture<F, Point3<F>, Vector3<F>>> {
                    Box::new(MappedTextureImpl::new(uvfn, texture))
                }
            }

            add_deserializer! {
                "ComposableSurface",
                [reflection_ratio: Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>>]
                [reflection_direction: Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>>]
                [threshold_direction: Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>>]
                [surface_color: Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>]
                -> Box<Surface<F, Point3<F>, Vector3<F>>> {
                    Box::new(ComposableSurface {
                        reflection_ratio: reflection_ratio,
                        reflection_direction: reflection_direction,
                        threshold_direction: threshold_direction,
                        surface_color: surface_color
                    })
                }
            }

            add_deserializer! {
                "blend_function_ratio",
                [ratio: F] -> Box<BlendFunction<F>> {
                    blend_function_ratio(ratio)
                }
            }

            macro_rules! deserialize_blending_functions {
                (
                    $($name:ident),+
                ) => {
                    $(
                        add_deserializer! {
                            concat!(
                                "blend_function_",
                                stringify!($name)
                            ),
                            -> Box<BlendFunction<F>> {
                                concat_idents!(blend_function_, $name)()
                            }
                        }
                    )+
                }
            }

            deserialize_blending_functions!(over, inside, outside, atop, xor, plus, multiply,
                                            screen, overlay, darken, lighten, dodge, burn,
                                            hard_light, soft_light, difference, exclusion);

            add_deserializer! {
                "surface_color_blend",
                [source: Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>]
                [destination: Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>>]
                [blend_function: Box<BlendFunction<F>>]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_blend(source, destination, blend_function)
                }
            }

            add_deserializer! {
                "surface_color_illumination_global",
                [light_color: Rgba<F>]
                [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_illumination_global(light_color, dark_color)
                }
            }

            add_deserializer! {
                "surface_color_perlin_hue_seed",
                [seed: u32] [size: F] [speed: F]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_perlin_hue_seed(seed, size, speed)
                }
            }

            add_deserializer! {
                "surface_color_perlin_hue_random",
                [size: F] [speed: F]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_perlin_hue_random(size, speed)
                }
            }

            add_deserializer! {
                "surface_color_illumination_directional",
                [direction: Vector3<F>] [light_color: Rgba<F>] [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_illumination_directional(direction, light_color, dark_color)
                }
            }

            add_deserializer! {
                "reflection_ratio_uniform",
                [ratio: F] -> Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>> {
                    reflection_ratio_uniform(ratio)
                }
            }

            add_deserializer! {
                "reflection_direction_specular",
                -> Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>> {
                    reflection_direction_specular()
                }
            }

            add_deserializer! {
                "threshold_direction_snell",
                [refractive_index: F]
                -> Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>> {
                    threshold_direction_snell(refractive_index)
                }
            }

            add_deserializer! {
                "threshold_direction_identity",
                -> Box<ThresholdDirectionProvider<F, Point3<F>, Vector3<F>>> {
                    threshold_direction_identity()
                }
            }

            add_deserializer! {
                "surface_color_uniform",
                [color: Rgba<F>] -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_uniform(color)
                }
            }

            add_deserializer! {
                "reflection_ratio_fresnel",
                [refractive_index_inside: F] [refractive_index_outside: F]
                -> Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>> {
                    reflection_ratio_fresnel(refractive_index_inside,
                                             refractive_index_outside)
                }
            }

            add_deserializer! {
                "surface_color_texture",
                [mapped_texture: Box<MappedTexture<F, Point3<F>, Vector3<F>>>]
                -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
                    surface_color_texture(mapped_texture)
                }
            }

            // Environments

            deserializers.insert("Environment",
                                 Box::new(|parent_json: &JsonValue, json: &JsonValue, parser: &Parser| {
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
                                 Box::new(|parent_json: &JsonValue, json: &JsonValue, parser: &Parser| {
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

    pub fn deserialize<T: Any>(&self, key: &str, json: &JsonValue, parent_json: &JsonValue) -> Result<Box<T>, ParserError> {
        let deserializer = try!(self.deserializer(key));
        let result = try!(deserializer(parent_json, json, &self)).downcast::<T>();

        match result {
            Ok(value) => Ok(value),
            Err(_) => {
                Err(ParserError::TypeMismatch {
                    description: format! {
                        "The constructor used (`{}`) has an incorrect type for this field. {:?}",
                        key,
                        parent_json
                    },
                    parent_json: parent_json.clone(),
                })
            }
        }
    }

    pub fn deserialize_constructor<T: Any>(&self, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let entries: Vec<(&str, &JsonValue)> = json.entries().collect();

        if entries.len() == 1 {
            let (constructor_key, constructor_value) = entries[0];
            self.deserialize::<T>(constructor_key, constructor_value, json)
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
            Ok(value) => self.deserialize::<T>(key, &value, &value),
            Err(err) => {
                Err(ParserError::SyntaxError {
                    key: key.to_owned(),
                    error: err,
                })
            }
        }
    }
}
