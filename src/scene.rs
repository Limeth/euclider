use std::any::Any;
use std::collections::HashMap;
use util::CustomFloat;
use universe::Environment;
use universe::Universe;
use universe::d3::Universe3D;
use json;
use json::JsonValue;
use json::iterators::Members;
use json::object::Object;
use mopa;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::entity::surface::*;
use universe::d3::entity::surface::*;
use na::Point3;
use na::Vector3;
use util::JsonFloat;
use image;

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
    }
}

pub trait Deserializable: mopa::Any {
    fn name() -> &'static str;
    fn deserialize(json: &JsonValue, parser: &Parser) -> Box<Self>;
}

pub struct Parser {
    pub deserializers: HashMap<&'static str, Box<Deserializer<Box<Any>>>>,
}

impl Parser {
    pub fn empty() -> Self {
        Parser {
            deserializers: HashMap::new(),
        }
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

            // Shapes

            deserializers.insert("VoidShape",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     Ok(Box::new(VoidShape::new()))
                                 }));

            // deserializers.insert("Sphere3",
            //                      Box::new(|json: &JsonValue, parser: &Parser| {
            //                          Ok(Box::new(/* TODO */))
            //                      }));

            // Materials

            deserializers.insert("Vacuum",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     Ok(Box::new(Vacuum::new()))
                                 }));

            // Surfaces

            deserializers.insert("uv_sphere",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let mut members: Members = json.members();
                                     let center = try!(parser.deserialize_constructor::<Point3<F>>(
                                                    try!(members.next().ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Center location not specified.".to_owned(),
                                                        json: json.clone()
                                                    }))
                                                ));

                                     Ok(Box::new(
                                             uv_sphere(*center)
                                     ))
                                 }));

            deserializers.insert("texture_image",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let path = try!(json.as_str()
                                                     .ok_or_else(|| ParserError::InvalidStructure {
                                                         description: "The `texture_image` must be a string.".to_owned(),
                                                         json: json.clone(),
                                                     }));

                                     let result: Box<Box<Texture<F>>> = Box::new(
                                         texture_image(
                                             image::open(path).expect(&format!("Could not find texture `{}`.", path))
                                         )
                                     );

                                     Ok(result)
                                 }));

            deserializers.insert("MappedTextureImpl3::new",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let members: Vec<&JsonValue> = json.members().collect();
                                     let uvfn = try!(parser.deserialize_constructor::<Box<UVFn<F, Point3<F>>>>(
                                                    members[0]
                                                ));
                                     let texture = try!(parser.deserialize_constructor::<Box<Texture<F>>>(
                                                    members[1]
                                                ));

                                     let result: Box<Box<MappedTexture<F, Point3<F>, Vector3<F>>>> = Box::new(Box::new(MappedTextureImpl::new(
                                                 *uvfn,
                                                 *texture
                                             )));

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

                                     let environment = object.get("environment").unwrap().as_str().unwrap();

                                     let result: Box<Box<Environment<F>>> =
                                         try!(parser.deserialize::<Box<Environment<F>>>(environment, json));

                                     Ok(result)
                                 }));

            deserializers.insert("Universe3D",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure {
                                             description: "The JSON value must be an object.".to_owned(),
                                             json: json.clone(),
                                         });
                                     };

                                     let mut universe = Universe3D::<F>::default();

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
            Err(ParserError::NoDeserializer {
                key: key.to_owned(),
            })
        }
    }

    pub fn deserialize<T: Any>(&self, key: &str, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let deserializer = try!(self.deserializer(key));
        let result = try!(deserializer(json, &self)).downcast::<T>();

        match result {
            Ok(value) => Ok(value),
            Err(original_value) => Err(ParserError::TypeMismatch {
                key: key.to_owned(),
                value: original_value,
                json: json.clone(),
            })
        }
    }

    pub fn deserialize_constructor<T: Any>(&self, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let entries: Vec<(&str, &JsonValue)> = json.entries().collect();

        if entries.len() == 1 {
            let (constructor_key, constructor_value) = entries[0];
            self.deserialize::<T>(constructor_key, constructor_value)
        } else {
            Err(ParserError::InvalidStructure {
                description: "A constructor must be an object containing a single key pointing to either an object or an array.".to_owned(),
                json: json.clone(),
            })
        }
    }

    pub fn parse<T: Any>(&self, key: &str, json: &str) -> Result<Box<T>, ParserError> {
        let value = json::parse(json);

        match value {
            Ok(value) => self.deserialize::<T>(key, &value),
            Err(err) => Err(ParserError::SyntaxError {
                key: key.to_owned(),
                error: err,
            })
        }
    }
}
