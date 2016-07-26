use std::any::Any;
use std::collections::HashMap;
use util::CustomFloat;
use universe::Environment;
use universe::d3::Universe3D;
use json;
use json::JsonValue;
use json::object::Object;
use mopa;

pub type ConstructorFunction<T> = Constructor<Box<[Box<Any>]>, T>;
pub type ConstructorStruct<T> = Constructor<HashMap<String, Box<Any>>, T>;
pub type Constructor<A, T> = Fn(A) -> T;
pub type Deserializer<T> = Fn(&JsonValue, &Parser) -> Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    NoDeserializer(String),
    TypeMismatch(String, Box<Any + 'static>),
    SyntaxError(String, json::Error),
    InvalidStructure,
}

pub struct Legend<T> {
    constructor_struct: Option<Box<ConstructorStruct<T>>>,
    constructor_functions: Vec<(String, Box<ConstructorFunction<T>>)>,
}

pub trait Prop: Sized {
    fn get_legend() -> Legend<Self>;
}

pub trait Deserializable: mopa::Any {
    fn name() -> &'static str;
    fn deserialize(json: &JsonValue, parser: &Parser) -> Box<Self>;
}

pub struct Parser {
    pub deserializers: HashMap<&'static str, Box<Deserializer<Box<Any>>>>
}

impl Parser {
    pub fn empty() -> Self {
        Parser {
            deserializers: HashMap::new(),
        }
    }

    pub fn default<F: CustomFloat>() -> Self {
        let mut parser = Self::empty();

        {
            let deserializers = &mut parser.deserializers;

            deserializers.insert("Environment",
                                 Box::new(|json: &JsonValue, parser: &Parser| {
                                     let object: &Object = if let JsonValue::Object(ref object) = *json {
                                         object
                                     } else {
                                         return Err(ParserError::InvalidStructure);
                                     };

                                     let environment = object.get("environment").unwrap().as_str().unwrap();

                                     println!("{}", environment);

                                     let result: Box<Box<Environment<F>>> =
                                         Box::new(Box::new(Universe3D::<F>::new()));

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
            Err(ParserError::NoDeserializer(key.to_owned()))
        }
    }

    pub fn deserialize<T: Any>(&self, key: &str, json: &JsonValue) -> Result<Box<T>, ParserError> {
        let deserializer = try!(self.deserializer(key));
        let result = try!(deserializer(json, &self)).downcast::<T>();

        match result {
            Ok(value) => Ok(value),
            Err(err) => Err(ParserError::TypeMismatch(key.to_owned(), err))
        }
    }

    pub fn parse<T: Any>(&self, key: &str, json: &str) -> Result<Box<T>, ParserError> {
        let value = json::parse(json);

        match value {
            Ok(value) => self.deserialize::<T>(key, &value),
            Err(err) => Err(ParserError::SyntaxError(key.to_owned(), err))
        }
    }
}
