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

/// Ties two `Material`s (exiting, entering) to a `TransitionHandler`
pub type TransitionHandlers<F, P, V> = TypePairMap<Box<TransitionHandler<F, P, V>>>;

/// Computes the color of the surface (not the reflection).
// Send + Sync must be at the end of the type alias definition.
pub type TransitionHandler<F, P, V> = Fn(&Material<F, P, V>,
                                         &Material<F, P, V>,
                                         &TracingContext<F, P, V>
                                      ) -> Option<Rgba<F>> + Send + Sync;

pub trait Material<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> where Self: HasId + Debug + Display {}

#[derive(Default)]
pub struct Vacuum {}

impl Vacuum {
    pub fn new() -> Self {
        Vacuum {}
    }
}

impl HasId for Vacuum {
    fn id(&self) -> TypeId {
        Self::id_static()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Material<F, P, V> for Vacuum {}

impl Debug for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vacuum")
    }
}

impl Display for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vacuum")
    }
}
