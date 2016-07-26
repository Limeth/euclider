use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use util::CustomFloat;
use universe::Environment;
use universe::Universe;
use universe::d3::Universe3D;
use json;
use json::JsonValue;
use json::object::Object;
use mopa;
use universe::entity::material;
use universe::entity::material::TransitionHandler;
use universe::entity::material::Vacuum;
use na::Point3;
use na::Vector3;

pub type Deserializer<T> = Fn(&JsonValue, &Parser) -> Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    NoDeserializer {
        key: String,
    },
    TypeMismatch {
        key: String,
        value: Box<Any + 'static>,
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
    pub type_map: HashMap<&'static str, TypeId>,
}

impl Parser {
    pub fn empty() -> Self {
        Parser {
            deserializers: HashMap::new(),
            type_map: HashMap::new(),
        }
    }

    #[allow(unused_variables)]
    pub fn default<F: CustomFloat>() -> Self {
        let mut parser = Self::empty();

        {
            let type_map = &mut parser.type_map;

            // Materials

            type_map.insert("Vacuum", TypeId::of::<Vacuum>());
        }

        {
            let deserializers = &mut parser.deserializers;

            // Materials

            deserializers.insert("Vacuum",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     Ok(Box::new(Vacuum::new()))
                                 }));

            // Material transition handlers

            deserializers.insert("transition_3d_vacuum_vacuum",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let value: Box<TransitionHandler<F, Point3<F>, Vector3<F>>> = 
                                         Box::new(material::transition_vacuum_vacuum::<F, Point3<F>, Vector3<F>>);
                                     Ok(Box::new(value))
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

                                     let mut universe = Universe3D::<F>::new();

                                     {
                                         let json_transitions: &Vec<JsonValue> =
                                             if let JsonValue::Array(ref json_transitions) = *object.get("transitions")
                                                .expect("The array `transitions` is missing.") {
                                             json_transitions
                                         } else {
                                             return Err(ParserError::InvalidStructure {
                                                 description: "The field `transitions` is not an array.".to_owned(),
                                                 json: json.clone(),
                                             });
                                         };

                                         let universe_transitions = universe.transitions_mut();

                                         for json_transition in json_transitions {
                                             let json_transition = if let JsonValue::Object(ref json_transition) = *json_transition {
                                                 json_transition
                                             } else {
                                                 return Err(ParserError::InvalidStructure {
                                                     description: "The transition must be an object!".to_owned(),
                                                     json: json_transition.clone(),
                                                 });
                                             };

                                             let from_str = try!(try!(json_transition.get("from")
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Missing a `from` field in transition definition.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }))
                                                    .as_str()
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "The field `from` must be a string.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }));
                                             let to_str = try!(try!(json_transition.get("to")
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Missing a `to` field in transition definition.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }))
                                                    .as_str()
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "The field `to` must be a string.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }));
                                             let transition_field = try!(json_transition.get("transition")
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "Missing a `transition` field in transition definition.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }));
                                             let transition_str = try!(transition_field                                                .as_str()
                                                    .ok_or_else(|| ParserError::InvalidStructure {
                                                        description: "The field `transition` must be a string.".to_owned(),
                                                        json: JsonValue::Object(json_transition.clone()),
                                                    }));
                                             let from_id = try!(parser.type_map.get(from_str)
                                                              .ok_or_else(|| ParserError::MissingType {
                                                                  type_str: from_str.to_owned(),
                                                              }));
                                             let to_id = try!(parser.type_map.get(to_str)
                                                              .ok_or_else(|| ParserError::MissingType {
                                                                  type_str: to_str.to_owned(),
                                                              }));
                                             let transition = try!(parser
                                                .deserialize::<Box<TransitionHandler<F, Point3<F>, Vector3<F>>>>(transition_str, transition_field));

                                             universe_transitions.insert((*from_id, *to_id), *transition);
                                         }
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
