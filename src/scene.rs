use palette::Rgba;
use palette::Hsva;
use std::any::Any;
use std::collections::HashMap;
use util::CustomFloat;
use universe::Environment;
use universe::Universe;
use universe::d3::Universe3;
use universe::d4::Universe4;
use json;
use json::JsonValue;
use universe::entity::*;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::entity::surface::*;
use universe::d3::entity::Entity3Impl;
use universe::d3::entity::Camera3;
use universe::d3::entity::surface::*;
use universe::d3::entity::shape::*;
use universe::d3::entity::camera::*;
use universe::d4::entity::Entity4Impl;
use universe::d4::entity::Camera4;
use universe::d4::entity::surface::*;
use universe::d4::entity::shape::*;
use universe::d4::entity::camera::*;
use universe::d3::Point3;
use universe::d3::Vector3;
use universe::d4::Point4;
use universe::d4::Vector4;
use util::JsonFloat;
use image;
use meval::Expr;
use core::str::FromStr;
use ::F;

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
        @deserialize Vec [ $($item_type:tt)+ ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {{
        let json = $json;
        let mut result: Vec< $($item_type)+ > = Vec::new();

        for member in json.members() {
            result.push(
                deserializer! {
                    @deserialize [ $($item_type)+ ]
                    parent_json: $parent_json,
                    parser: $parser,
                    json: member
                }
            );
        }

        result
    }};

    (
        @deserialize [ Vec $($item_type:tt)+ ]
        parent_json: $parent_json:expr,
        parser: $parser:expr,
        json: $json:expr
    ) => {
        remove_surrounding_brackets! {
            trim: [ $($item_type)+ ]    // The token tree from which to remove surrounding brackets
            callback: [ deserializer ]  // The macro that is called upon completion
            arguments_preceding: {      // Arguments preceding the result
                @deserialize Vec
            }
            arguments_following: {      // Arguments following the result
                parent_json: $parent_json,
                parser: $parser,
                json: $json
            }
        }
    };

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
        description: String,
        key: String,
    },
    SyntaxError {
        description: String,
        error: json::Error,
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
    pub fn default() -> Self {
        let mut parser = Self::empty();

        {
            let deserializers = &mut parser.deserializers;

            /// Creates deserializers for given aliases/names/keys and registers them.
            /// Thanks to talchas for the help with the munching algorithm!
            macro_rules! add_deserializer {
                // Inserts a deserializer for a single alias/name/key.
                (
                    @construct
                    $constructor_name:expr,
                    $($rest:tt)+
                ) => {
                    deserializers.insert(
                        $constructor_name,
                        Box::new(deserializer! {
                            $($rest)+
                        })
                    );
                };

                // Munches a single alias/name/key and constructs it.
                (
                    @recurse
                    ($($body:tt)+)
                    $alias:expr
                    $(, $aliases:expr)*
                ) => {
                    add_deserializer! {
                        @construct
                        $alias,
                        $($body)+
                    }
                    add_deserializer!(@recurse ($($body)+) $($aliases),*)
                };

                // No more munching required, we're fully fed now.
                (
                    @recurse
                    ($($body:tt)+)
                ) => {};

                // Entry matcher. Places the remaining arguments into parentheses
                // and puts aliases at the end.
                (
                    $($aliases:expr),+ ;
                    $($body:tt)+
                ) => {
                    add_deserializer!(@recurse ($($body)+) $($aliases),+)
                };
            }

            // General

            add_deserializer! {
                "Point3", "Point3::new";
                [x: F] [y: F] [z: F] -> Point3 {
                    Point3::new(x, y, z)
                }
            };

            add_deserializer! {
                "Point4", "Point4::new";
                [x: F] [y: F] [z: F] [w: F] -> Point4 {
                    Point4::new(x, y, z, w)
                }
            };

            add_deserializer! {
                "Vector3", "Vector3::new";
                [x: F] [y: F] [z: F] -> Vector3 {
                    Vector3::new(x, y, z)
                }
            };

            add_deserializer! {
                "Vector4", "Vector4::new";
                [x: F] [y: F] [z: F] [w: F] -> Vector4 {
                    Vector4::new(x, y, z, w)
                }
            };

            add_deserializer! {
                "Rgba", "Rgba::new";
                [r: F] [g: F] [b: F] [a: F] -> Rgba<F> {
                    Rgba::<F>::new(r, g, b, a)
                }
            };

            add_deserializer! {
                "Rgba::new_u8";
                [r: u8] [g: u8] [b: u8] [a: u8] -> Rgba<F> {
                    Rgba::<F>::new_u8(r, g, b, a)
                }
            };

            add_deserializer! {
                "Rgba::from_hsva";
                [hue: F] [saturation: F] [value: F] [alpha: F] -> Rgba<F> {
                    Hsva::new(hue.into(), saturation, value, alpha).into()
                }
            };

            // Entities

            add_deserializer! {
                "Void3", "Void3::new";
                [material: Box<Material<Point3, Vector3>>]
                -> Box<Entity<Point3, Vector3>> {
                    Box::new(Void::<Point3, Vector3>::new(material))
                }
            };

            add_deserializer! {
                "Void4", "Void4::new";
                [material: Box<Material<Point4, Vector4>>]
                -> Box<Entity<Point4, Vector4>> {
                    Box::new(Void::<Point4, Vector4>::new(material))
                }
            };

            add_deserializer! {
                "Void3::new_with_vacuum";
                -> Box<Entity<Point3, Vector3>> {
                    Box::new(Void::<Point3, Vector3>::new_with_vacuum())
                }
            };

            add_deserializer! {
                "Void4::new_with_vacuum";
                -> Box<Entity<Point4, Vector4>> {
                    Box::new(Void::<Point4, Vector4>::new_with_vacuum())
                }
            };

            add_deserializer! {
                "Entity3Impl", "Entity3Impl::new", "Entity3Impl::new_with_surface";
                [shape: Box<Shape<Point3, Vector3>>]
                [material: Box<Material<Point3, Vector3>>]
                [surface: Box<Surface<Point3, Vector3>>]
                -> Box<Entity<Point3, Vector3>> {
                    Box::new(Entity3Impl::new_with_surface(shape, material, surface))
                }
            }

            add_deserializer! {
                "Entity4Impl", "Entity4Impl::new", "Entity4Impl::new_with_surface";
                [shape: Box<Shape<Point4, Vector4>>]
                [material: Box<Material<Point4, Vector4>>]
                [surface: Box<Surface<Point4, Vector4>>]
                -> Box<Entity<Point4, Vector4>> {
                    Box::new(Entity4Impl::new_with_surface(shape, material, surface))
                }
            }

            add_deserializer! {
                "Entity3Impl::new_without_surface";
                [shape: Box<Shape<Point3, Vector3>>]
                [material: Box<Material<Point3, Vector3>>]
                -> Box<Entity<Point3, Vector3>> {
                    Box::new(Entity3Impl::new_without_surface(shape, material))
                }
            }

            add_deserializer! {
                "Entity4Impl::new_without_surface";
                [shape: Box<Shape<Point4, Vector4>>]
                [material: Box<Material<Point4, Vector4>>]
                -> Box<Entity<Point4, Vector4>> {
                    Box::new(Entity4Impl::new_without_surface(shape, material))
                }
            }

            // Shapes

            add_deserializer! {
                "VoidShape3", "VoidShape3::new";
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(VoidShape::new())
                }
            }

            add_deserializer! {
                "VoidShape4", "VoidShape4::new";
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(VoidShape::new())
                }
            }

            add_deserializer! {
                "ComposableShape3", "ComposableShape3::new", "ComposableShape3::of";
                [shapes: Vec<Box<Shape<Point3, Vector3>>> ]
                [operation: SetOperation]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(ComposableShape::of(shapes, operation))
                }
            }

            add_deserializer! {
                "ComposableShape4", "ComposableShape4::new", "ComposableShape4::of";
                [shapes: Vec<Box<Shape<Point4, Vector4>>> ]
                [operation: SetOperation]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(ComposableShape::of(shapes, operation))
                }
            }

            add_deserializer! {
                "SetOperation", "SetOperation::new";
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
                "Sphere3", "Sphere3::new";
                [center: Point3] [radius: F]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Sphere::<Point3, Vector3>::new(center, radius))
                }
            }

            add_deserializer! {
                "Sphere4", "Sphere4::new";
                [center: Point4] [radius: F]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(Sphere::<Point4, Vector4>::new(center, radius))
                }
            }

            add_deserializer! {
                "Hyperplane3", "Hyperplane3::new";
                [normal: Vector3] [constant: F]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Hyperplane::new(normal, constant))
                }
            }

            add_deserializer! {
                "Hyperplane4", "Hyperplane4::new";
                [normal: Vector4] [constant: F]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(Hyperplane::new(normal, constant))
                }
            }

            add_deserializer! {
                "Hyperplane3::new_with_point";
                [normal: Vector3] [point: Point3]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Hyperplane::new_with_point(normal, &point))
                }
            }

            add_deserializer! {
                "Hyperplane4::new_with_point";
                [normal: Vector4] [point: Point4]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(Hyperplane::new_with_point(normal, &point))
                }
            }

            add_deserializer! {
                "Hyperplane3::new_with_vectors";
                [first: Vector3] [second: Vector3] [point: Point3]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Hyperplane::new_with_vectors(&first, &second, &point))
                }
            }

            add_deserializer! {
                "HalfSpace3", "HalfSpace3::new";
                [plane: Box<Shape<Point3, Vector3>>]
                [sign: F] -> Box<Shape<Point3, Vector3>> {
                    let plane: Hyperplane<Point3, Vector3>
                        = *try!(<Shape<Point3, Vector3>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Hyperplane3`.".to_string(),
                            })));
                    Box::new(HalfSpace::new(plane, sign))
                }
            }

            add_deserializer! {
                "HalfSpace4", "HalfSpace4::new";
                [plane: Box<Shape<Point4, Vector4>>]
                [sign: F] -> Box<Shape<Point4, Vector4>> {
                    let plane: Hyperplane<Point4, Vector4>
                        = *try!(<Shape<Point4, Vector4>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Hyperplane4`.".to_string(),
                            })));
                    Box::new(HalfSpace::new(plane, sign))
                }
            }

            add_deserializer! {
                "HalfSpace3::new_with_point";
                [plane: Box<Shape<Point3, Vector3>>]
                [point: Point3] -> Box<Shape<Point3, Vector3>> {
                    let plane: Hyperplane<Point3, Vector3>
                        = *try!(<Shape<Point3, Vector3>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Hyperplane3`.".to_string(),
                            })));
                    Box::new(HalfSpace::new_with_point(plane, &point))
                }
            }

            add_deserializer! {
                "HalfSpace4::new_with_point";
                [plane: Box<Shape<Point4, Vector4>>]
                [point: Point4] -> Box<Shape<Point4, Vector4>> {
                    let plane: Hyperplane<Point4, Vector4>
                        = *try!(<Shape<Point4, Vector4>>::downcast(plane)
                            .or_else(|err| Err(ParserError::CustomError {
                                description: "Invalid type, expected a `Hyperplane4`.".to_string(),
                            })));
                    Box::new(HalfSpace::new_with_point(plane, &point))
                }
            }

            add_deserializer! {
                "HalfSpace3::cuboid";
                [center: Point3] [dimensions: Vector3]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(cuboid(center, dimensions))
                }
            }

            add_deserializer! {
                "HalfSpace4::hypercuboid";
                [center: Point4] [dimensions: Vector4]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(hypercuboid(center, dimensions))
                }
            }

            add_deserializer! {
                "Cylinder3", "Cylinder3::new";
                [center: Point3] [direction: Vector3] [radius: F]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Cylinder::new(center, &direction, radius))
                }
            }

            add_deserializer! {
                "Cylinder4", "Cylinder4::new";
                [center: Point4] [direction: Vector4] [radius: F]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(Cylinder::new(center, &direction, radius))
                }
            }

            add_deserializer! {
                "Cylinder3::new_with_height";
                [center: Point3] [direction: Vector3] [radius: F] [height: F]
                -> Box<Shape<Point3, Vector3>> {
                    Box::new(Cylinder::new_with_height(center, &direction, radius, height))
                }
            }

            add_deserializer! {
                "Cylinder4::new_with_height";
                [center: Point4] [direction: Vector4] [radius: F] [height: F]
                -> Box<Shape<Point4, Vector4>> {
                    Box::new(Cylinder::new_with_height(center, &direction, radius, height))
                }
            }

            // Materials

            add_deserializer! {
                "Vacuum3", "Vacuum3::new";
                -> Box<Material<Point3, Vector3>> {
                    Box::new(Vacuum::new())
                }
            }

            add_deserializer! {
                "Vacuum4", "Vacuum4::new";
                -> Box<Material<Point4, Vector4>> {
                    Box::new(Vacuum::new())
                }
            }

            add_deserializer! {
                "ComponentTransformationExpr", "ComponentTransformationExpr::new";
                [expression: &str] [inverse_expression: &str]
                -> ComponentTransformationExpr {
                    let expression = try!(
                        Expr::from_str(expression)
                        .map_err(|_| ParserError::CustomError {
                            description: format!(
                                "Invalid component transformation expression `{}`.",
                                expression
                            ),
                        })
                    );
                    let inverse_expression = try!(
                        Expr::from_str(inverse_expression)
                        .map_err(|_| ParserError::CustomError {
                            description: format!(
                                "Invalid component transformation expression `{}`.",
                                inverse_expression
                            ),
                        })
                    );

                    ComponentTransformationExpr {
                        expression: expression,
                        inverse_expression: inverse_expression,
                    }
                }
            }

            add_deserializer! {
                "ComponentTransformation3", "ComponentTransformation3::new";
                [expressions: Vec<ComponentTransformationExpr>]
                -> Box<LinearTransformation<Point3, Vector3>> {
                    Box::new(ComponentTransformation {
                        expressions: expressions,
                    })
                }
            }

            add_deserializer! {
                "ComponentTransformation4", "ComponentTransformation4::new";
                [expressions: Vec<ComponentTransformationExpr>]
                -> Box<LinearTransformation<Point4, Vector4>> {
                    Box::new(ComponentTransformation {
                        expressions: expressions,
                    })
                }
            }

            add_deserializer! {
                "LinearSpace3", "LinearSpace3::new";
                [legend: String]
                [transformations: Vec<Box<LinearTransformation<Point3, Vector3>>>]
                -> Box<Material<Point3, Vector3>> {
                    Box::new(LinearSpace {
                        legend: legend,
                        transformations: transformations,
                    })
                }
            }

            add_deserializer! {
                "LinearSpace4", "LinearSpace4::new";
                [legend: String]
                [transformations: Vec<Box<LinearTransformation<Point4, Vector4>>>]
                -> Box<Material<Point4, Vector4>> {
                    Box::new(LinearSpace {
                        legend: legend,
                        transformations: transformations,
                    })
                }
            }

            // Surfaces

            add_deserializer! {
                "uv_sphere_3";
                [center: Point3] -> Box<UVFn<Point3>> {
                    uv_sphere(center)
                }
            }

            add_deserializer! {
                "uv_derank_4";
                [uvfn: Box<UVFn<Point3>>] -> Box<UVFn<Point4>> {
                    uv_derank(uvfn)
                }
            }

            add_deserializer! {
                "texture_image_nearest_neighbor";
                [path: &str] -> Box<Texture> {
                    let data = try!(image::open(path)
                        .map_err(|_| ParserError::CustomError {
                            description: format!("Could not load texture `{}`", path),
                        }));

                    texture_image_nearest_neighbor(data)
                }
            }

            add_deserializer! {
                "texture_image_linear";
                [path: &str] -> Box<Texture> {
                    let data = try!(image::open(path)
                        .map_err(|_| ParserError::CustomError {
                            description: format!("Could not load texture `{}`", path),
                        }));

                    texture_image_linear(data)
                }
            }

            add_deserializer! {
                "MappedTextureImpl3", "MappedTextureImpl3::new";
                [uvfn: Box<UVFn<Point3>>]
                [texture: Box<Texture>]
                -> Box<MappedTexture<Point3, Vector3>> {
                    Box::new(MappedTextureImpl::new(uvfn, texture))
                }
            }

            add_deserializer! {
                "MappedTextureImpl4", "MappedTextureImpl4::new";
                [uvfn: Box<UVFn<Point4>>]
                [texture: Box<Texture>]
                -> Box<MappedTexture<Point4, Vector4>> {
                    Box::new(MappedTextureImpl::new(uvfn, texture))
                }
            }

            add_deserializer! {
                "ComposableSurface3", "ComposableSurface3::new";
                [reflection_ratio: Box<ReflectionRatioProvider<Point3, Vector3>>]
                [reflection_direction: Box<ReflectionDirectionProvider<Point3, Vector3>>]
                [threshold_direction: Box<ThresholdDirectionProvider<Point3, Vector3>>]
                [surface_color: Box<SurfaceColorProvider<Point3, Vector3>>]
                -> Box<Surface<Point3, Vector3>> {
                    Box::new(ComposableSurface {
                        reflection_ratio: reflection_ratio.into(),
                        reflection_direction: reflection_direction.into(),
                        threshold_direction: threshold_direction.into(),
                        surface_color: surface_color.into(),
                    })
                }
            }

            add_deserializer! {
                "ComposableSurface4", "ComposableSurface4::new";
                [reflection_ratio: Box<ReflectionRatioProvider<Point4, Vector4>>]
                [reflection_direction: Box<ReflectionDirectionProvider<Point4, Vector4>>]
                [threshold_direction: Box<ThresholdDirectionProvider<Point4, Vector4>>]
                [surface_color: Box<SurfaceColorProvider<Point4, Vector4>>]
                -> Box<Surface<Point4, Vector4>> {
                    Box::new(ComposableSurface {
                        reflection_ratio: reflection_ratio.into(),
                        reflection_direction: reflection_direction.into(),
                        threshold_direction: threshold_direction.into(),
                        surface_color: surface_color.into(),
                    })
                }
            }

            add_deserializer! {
                "blend_function_ratio";
                [ratio: F] -> Box<BlendFunction> {
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
                            );
                            -> Box<BlendFunction> {
                                mashup! {
                                    substitute["blend_function"] = blend_function_ $name;
                                }

                                substitute! {
                                    "blend_function"()
                                }
                                // concat_idents!(blend_function_, $name)()
                            }
                        }
                    )+
                }
            }

            deserialize_blending_functions!(over, inside, outside, atop, xor, plus, multiply,
                                            screen, overlay, darken, lighten, dodge, burn,
                                            hard_light, soft_light, difference, exclusion);

            add_deserializer! {
                "surface_color_blend_3";
                [source: Box<SurfaceColorProvider<Point3, Vector3>>]
                [destination: Box<SurfaceColorProvider<Point3, Vector3>>]
                [blend_function: Box<BlendFunction>]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_blend(source, destination, blend_function)
                }
            }

            add_deserializer! {
                "surface_color_blend_4";
                [source: Box<SurfaceColorProvider<Point4, Vector4>>]
                [destination: Box<SurfaceColorProvider<Point4, Vector4>>]
                [blend_function: Box<BlendFunction>]
                -> Box<SurfaceColorProvider<Point4, Vector4>> {
                    surface_color_blend(source, destination, blend_function)
                }
            }

            add_deserializer! {
                "surface_color_illumination_global_3";
                [light_color: Rgba<F>]
                [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_illumination_global(light_color, dark_color)
                }
            }

            add_deserializer! {
                "surface_color_illumination_global_4";
                [light_color: Rgba<F>]
                [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<Point4, Vector4>> {
                    surface_color_illumination_global(light_color, dark_color)
                }
            }

            add_deserializer! {
                "surface_color_perlin_hue_seed_3";
                [seed: u32] [size: F] [speed: F]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_perlin_hue_seed(seed, size, speed)
                }
            }

            add_deserializer! {
                "surface_color_perlin_hue_random_3";
                [size: F] [speed: F]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_perlin_hue_random(size, speed)
                }
            }

            add_deserializer! {
                "surface_color_illumination_directional_3";
                [direction: Vector3] [light_color: Rgba<F>] [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_illumination_directional(direction, light_color, dark_color)
                }
            }

            add_deserializer! {
                "surface_color_illumination_directional_4";
                [direction: Vector4] [light_color: Rgba<F>] [dark_color: Rgba<F>]
                -> Box<SurfaceColorProvider<Point4, Vector4>> {
                    surface_color_illumination_directional(direction, light_color, dark_color)
                }
            }

            add_deserializer! {
                "reflection_ratio_uniform_3";
                [ratio: F] -> Box<ReflectionRatioProvider<Point3, Vector3>> {
                    reflection_ratio_uniform(ratio)
                }
            }

            add_deserializer! {
                "reflection_ratio_uniform_4";
                [ratio: F] -> Box<ReflectionRatioProvider<Point4, Vector4>> {
                    reflection_ratio_uniform(ratio)
                }
            }

            add_deserializer! {
                "reflection_direction_specular_3";
                -> Box<ReflectionDirectionProvider<Point3, Vector3>> {
                    reflection_direction_specular()
                }
            }

            add_deserializer! {
                "reflection_direction_specular_4";
                -> Box<ReflectionDirectionProvider<Point4, Vector4>> {
                    reflection_direction_specular()
                }
            }

            add_deserializer! {
                "threshold_direction_snell_3";
                [refractive_index: F]
                -> Box<ThresholdDirectionProvider<Point3, Vector3>> {
                    threshold_direction_snell(refractive_index)
                }
            }

            add_deserializer! {
                "threshold_direction_snell_4";
                [refractive_index: F]
                -> Box<ThresholdDirectionProvider<Point4, Vector4>> {
                    threshold_direction_snell(refractive_index)
                }
            }

            add_deserializer! {
                "threshold_direction_identity_3";
                -> Box<ThresholdDirectionProvider<Point3, Vector3>> {
                    threshold_direction_identity()
                }
            }

            add_deserializer! {
                "threshold_direction_identity_4";
                -> Box<ThresholdDirectionProvider<Point4, Vector4>> {
                    threshold_direction_identity()
                }
            }

            add_deserializer! {
                "surface_color_uniform_3";
                [color: Rgba<F>] -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_uniform(color)
                }
            }

            add_deserializer! {
                "surface_color_uniform_4";
                [color: Rgba<F>] -> Box<SurfaceColorProvider<Point4, Vector4>> {
                    surface_color_uniform(color)
                }
            }

            add_deserializer! {
                "reflection_ratio_fresnel_3";
                [refractive_index_inside: F] [refractive_index_outside: F]
                -> Box<ReflectionRatioProvider<Point3, Vector3>> {
                    reflection_ratio_fresnel(refractive_index_inside,
                                             refractive_index_outside)
                }
            }

            add_deserializer! {
                "reflection_ratio_fresnel_4";
                [refractive_index_inside: F] [refractive_index_outside: F]
                -> Box<ReflectionRatioProvider<Point4, Vector4>> {
                    reflection_ratio_fresnel(refractive_index_inside,
                                             refractive_index_outside)
                }
            }

            add_deserializer! {
                "surface_color_texture_3";
                [mapped_texture: Box<MappedTexture<Point3, Vector3>>]
                -> Box<SurfaceColorProvider<Point3, Vector3>> {
                    surface_color_texture(mapped_texture)
                }
            }

            add_deserializer! {
                "surface_color_texture_4";
                [mapped_texture: Box<MappedTexture<Point4, Vector4>>]
                -> Box<SurfaceColorProvider<Point4, Vector4>> {
                    surface_color_texture(mapped_texture)
                }
            }

            // Environments

            add_deserializer! {
                "Universe3", "Universe3::new";
                [camera: Box<Camera3>]
                [entities: Vec<Box<Entity<Point3, Vector3>>>]
                [background: Box<MappedTexture<Point3, Vector3>>]
                -> Box<Environment> {
                    let mut universe = Universe3::construct(camera);

                    universe.set_entities(entities);
                    universe.set_background(background);

                    Box::new(universe)
                }
            }

            add_deserializer! {
                "PitchYawCamera3", "PitchYawCamera3::new";
                -> Box<Camera3> {
                    Box::new(PitchYawCamera3::new())
                }
            }

            add_deserializer! {
                "PitchYawCamera3::new_with_location";
                [location: Point3] -> Box<Camera3> {
                    Box::new(PitchYawCamera3::new_with_location(location))
                }
            }

            add_deserializer! {
                "FreeCamera3", "FreeCamera3::new";
                -> Box<Camera3> {
                    Box::new(FreeCamera3::new())
                }
            }

            add_deserializer! {
                "FreeCamera3::new_with_location";
                [location: Point3] -> Box<Camera3> {
                    Box::new(FreeCamera3::new_with_location(location))
                }
            }

            add_deserializer! {
                "Universe4", "Universe4::new";
                [camera: Box<Camera4>]
                [entities: Vec<Box<Entity<Point4, Vector4>>>]
                [background: Box<MappedTexture<Point4, Vector4>>]
                -> Box<Environment> {
                    let mut universe = Universe4::construct(camera);

                    universe.set_entities(entities);
                    universe.set_background(background);

                    Box::new(universe)
                }
            }

            add_deserializer! {
                "FreeCamera4", "FreeCamera4::new";
                -> Box<Camera4> {
                    Box::new(FreeCamera4::new())
                }
            }

            add_deserializer! {
                "FreeCamera4::new_with_location";
                [location: Point4] -> Box<Camera4> {
                    Box::new(FreeCamera4::new_with_location(location))
                }
            }
        }

        parser
    }

    pub fn deserializer(&self, key: &str) -> Result<&Deserializer<Box<Any>>, ParserError> {
        let option = self.deserializers.get(key);

        if option.is_some() {
            Ok(option.unwrap().as_ref())
        } else {
            Err(ParserError::NoDeserializer {
                description: format! {
                    "No deserializer registered for key `{}`.",
                    key
                },
                key: key.to_string(),
            })
        }
    }

    pub fn deserialize<T: Any>(&self, key: &str, json: &JsonValue, parent_json: &JsonValue) -> Result<Box<T>, ParserError> {
        let deserializer = try!(self.deserializer(key));
        let result = try!(deserializer(parent_json, json, self)).downcast::<T>();

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
            Err(ParserError::InvalidConstructor {
                description: concat! {
                    "A constructor must be an object containing a single key pointing to",
                    " either an object or an array."
                }.to_string(),
                parent_json: json.clone(),
            })
        }
    }

    pub fn parse<T: Any>(&self, json: &str) -> Result<Box<T>, ParserError> {
        let value = json::parse(json);

        match value {
            Ok(value) => self.deserialize_constructor::<T>(&value),
            Err(err) => {
                Err(ParserError::SyntaxError {
                    description: "Invalid JSON file. Please, check the syntax.".to_string(),
                    error: err,
                })
            }
        }
    }
}

#[allow(float_cmp)]
#[cfg(test)]
mod tests {
    use std::any::Any;
    use json::JsonValue;
    use util::JsonFloat;
    use util::CustomFloat;
    use super::*;

    #[allow(unused_variables)]
    fn parse_float_internal() -> F {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: F] -> F {
                component
            }
        }));

        *parser.parse::<F>(r##"{"item": [ 42 ]}"##).unwrap()
    }

    #[test]
    fn parse_float() {
        assert_eq!(parse_float_internal::<f32>(), 42_f32);
        assert_eq!(parse_float_internal::<f64>(), 42_f64);
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_str() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: &str] -> String {
                component.to_string()
            }
        }));

        assert_eq! {
            &*parser.parse::<String>(r##"{"item": [ "42" ]}"##).unwrap(),
            "42"
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_string() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: String] -> String {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<String>(r##"{"item": [ "42" ]}"##).unwrap(),
            "42".to_string()
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_f32() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: f32] -> f32 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<f32>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_f32
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_f64() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: f64] -> f64 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<f64>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_f64
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_u32() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: u32] -> u32 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<u32>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_u32
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_u64() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: u64] -> u64 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<u64>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_u64
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_u16() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: u16] -> u16 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<u16>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_u16
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_u8() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: u8] -> u8 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<u8>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_u8
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_usize() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: usize] -> usize {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<usize>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_usize
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_i32() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: i32] -> i32 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<i32>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_i32
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_i64() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: i64] -> i64 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<i64>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_i64
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_i16() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: i16] -> i16 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<i16>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_i16
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_i8() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: i8] -> i8 {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<i8>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_i8
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_isize() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: isize] -> isize {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<isize>(r##"{"item": [ 42 ]}"##).unwrap(),
            42_isize
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_bool() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: bool] -> bool {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<bool>(r##"{"item": [ true ]}"##).unwrap(),
            true
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_vec() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: Vec<f32>] -> Vec<f32> {
                component
            }
        }));

        assert_eq! {
            *parser.parse::<Vec<f32>>(r##"{"item": [ [ 42, 84 ] ]}"##).unwrap(),
            vec!(42_f32, 84_f32)
        }
    }

    #[allow(unused_variables)]
    #[test]
    fn parse_constructor() {
        let mut parser = Parser::empty();

        parser.deserializers.insert("item", Box::new(deserializer! {
            [component: Box<f32>] -> f32 {
                *component
            }
        }));

        parser.deserializers.insert("inner_item", Box::new(deserializer! {
            [component: f32] -> Box<f32> {
                Box::new(component)
            }
        }));

        assert_eq! {
            *parser.parse::<f32>(r##"{"item": [ { "inner_item": [ 42 ] } ]}"##).unwrap(),
            42_f32
        }
    }
}
