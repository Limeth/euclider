use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::any::TypeId;
use std::any::Any;
use palette::Rgba;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use util::TypePairMap;
use universe::entity::shape::TracingContext;
use na::Cast;

/// Ties two `Material`s (exiting, entering) to a `TransitionHandler`
pub type TransitionHandlers<F, P, V> = TypePairMap<Box<TransitionHandler<F, P, V>>>;

/// Computes the color of the surface (not the reflection).
// Send + Sync must be at the end of the type alias definition.
pub type TransitionHandler<F, P, V> = Fn(&Material<F, P, V>,
                                         &Material<F, P, V>,
                                         &TracingContext<F, P, V>
                                      ) -> Rgba<F> + Send + Sync;

pub trait Material<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    where Self: HasId + Debug + Display
{
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

impl Vacuum {
    pub fn new() -> Self {
        Vacuum {}
    }
}

material!(Vacuum);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Material<F, P, V> for Vacuum {}

#[allow(unused_variables)]
pub fn transition_vacuum_vacuum<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (from: &Material<F, P, V>,
     to: &Material<F, P, V>,
     context: &TracingContext<F, P, V>)
     -> Rgba<F> {
    let trace = context.trace;
    // Offset the new origin, so it doesn't hit the same shape over and over
    // The question is -- is there a better way? I think not.
    let new_origin = context.intersection.location +
                     -*context.intersection_normal_closer * F::epsilon() * Cast::from(128.0);

    trace(context.time,
          context.intersection_traceable,
          &new_origin,
          &context.intersection.direction)
}
