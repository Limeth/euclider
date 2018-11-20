use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::any::TypeId;
use std::any::Any;
use std::collections::HashMap;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use na::Dimension;
use meval::{Expr, Context as MevalContext};
use num::NumCast;
use ::F;

pub trait Material<P: CustomPoint<V>, V: CustomVector<P>>
    where Self: HasId + Debug + Display + Send + Sync
{
    fn enter(&self, location: &P, direction: &mut V);
    fn exit(&self, location: &P, direction: &mut V);
    fn trace_path(&self, location: &P, direction: &V, distance: &F) -> (P, V);
}

#[macro_export]
macro_rules! material {
    ($($t:tt)*) => {
        has_id!($($t)*);
        name_as_display!($($t)*);
    }
}

#[derive(Default, Debug)]
pub struct Vacuum {}

material!(Vacuum);

impl Vacuum {
    pub fn new() -> Self {
        Vacuum {}
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> Material<P, V> for Vacuum {
    #[allow(unused_variables)]
    fn enter(&self, location: &P, direction: &mut V) {
        // Empty
    }

    #[allow(unused_variables)]
    fn exit(&self, location: &P, direction: &mut V) {
        // Empty
    }

    fn trace_path(&self, location: &P, direction: &V, distance: &F) -> (P, V) {
        (*location + *direction * *distance, *direction)
    }
}

pub trait LinearTransformation<P: CustomPoint<V>, V: CustomVector<P>>: Debug + Send + Sync {
    fn transform(&self, vector: &mut V, legend: &str);
    fn inverse_transform(&self, vector: &mut V, legend: &str);
}

#[derive(Debug)]
pub struct ComponentTransformationExpr {
    pub expression: Expr,
    pub inverse_expression: Expr,
}

#[derive(Debug)]
pub struct ComponentTransformation {
    pub expressions: Vec<ComponentTransformationExpr>,
}

impl ComponentTransformation {
    fn create_context<P: CustomPoint<V>, V: CustomVector<P>>(vector: &V, legend: &str) -> HashMap<String, f64> {
        let mut result: HashMap<String, f64> = HashMap::new();
        let mut chars = legend.chars();

        for component in vector.iter() {
            let character = chars.next()
                                 .expect(&format!("The legend is too short! Make sure it is sufficient for {} dimensions.", <P as Dimension>::dimension(None)))
                                 .to_string();

            result.insert(character, <f64 as NumCast>::from(*component).unwrap());
        }

        result
    }

    fn transform_with<P: CustomPoint<V>, V: CustomVector<P>, E: Fn(&ComponentTransformationExpr) -> &Expr>
        (&self, vector: &mut V, expr: &E, legend: &str) {
        let dim = <P as Dimension>::dimension(None);

        if self.expressions.len() != dim {
            panic!("The number of functions must be equal to the number of dimensions ({})!", dim);
        }

        let mut context = MevalContext::new();

        for (key, value) in Self::create_context(vector, legend)
        {
            context.var(key, value);
        }

        for (component, expression) in vector.iter_mut().zip(self.expressions.iter()) {
            let result = expr(expression).eval_with_context(context.clone())
                        .expect("Could not evaluate the expression.");

            *component = <F as NumCast>::from(result).unwrap();
        }
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> LinearTransformation<P, V> for ComponentTransformation {
    fn transform(&self, vector: &mut V, legend: &str) {
        self.transform_with(vector, &|expr| &expr.expression, legend)
    }

    fn inverse_transform(&self, vector: &mut V, legend: &str) {
        self.transform_with(vector, &|expr| &expr.inverse_expression, legend)
    }
}

#[derive(Debug)]
pub struct LinearSpace<P: CustomPoint<V>, V: CustomVector<P>> {
    pub legend: String,
    pub transformations: Vec<Box<LinearTransformation<P, V>>>,
}

material!(LinearSpace<P: CustomPoint<V>, V: CustomVector<P>>);

impl<P: CustomPoint<V>, V: CustomVector<P>> Material<P, V> for LinearSpace<P, V> {
    #[allow(unused_variables)]
    fn enter(&self, location: &P, direction: &mut V) {
        self.transform(direction);
    }

    #[allow(unused_variables)]
    fn exit(&self, location: &P, direction: &mut V) {
        self.inverse_transform(direction);
    }

    fn trace_path(&self, location: &P, direction: &V, distance: &F) -> (P, V) {
        (*location + *direction * *distance, *direction)
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> LinearSpace<P, V> {
    fn transform(&self, vector: &mut V) {
        for transformation in &self.transformations {
            transformation.transform(vector, &self.legend)
        }
    }

    fn inverse_transform(&self, vector: &mut V) {
        let mut inverse = self.transformations.iter();

        while let Some(transformation) = inverse.next_back() {
            transformation.inverse_transform(vector, &self.legend)
        }
    }
}
