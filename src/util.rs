use std::any::TypeId;
use std::any::Any;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::u8;
use std::fmt::UpperExp;
use std::fmt::LowerExp;
use std::fmt::Display;
use std::fmt::Debug;
use std::clone::Clone;
use std::cmp::PartialOrd;
use std::cmp::PartialEq;
use std::ops::RemAssign;
use std::ops::DivAssign;
use std::ops::MulAssign;
use std::ops::SubAssign;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Rem;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::Add;
use std::ops::Index;
use std::ops::IndexMut;
use std::convert::From;
use std::default::Default;
use std::mem;
use std::cell::RefCell;
use std::cell::RefMut;
use std::sync::Arc;
use std::sync::RwLock;
use num::Num;
use num::One;
use num::Zero;
use num::traits::ParseFloatError;
use num::traits::NumCast;
use num::traits::ToPrimitive;
use palette;
use image::Rgba;
use na;
use na::Eye;
use na::Column;
use na::Transpose;
use na::BaseFloat;
use na::Cast;
use na::ApproxEq;
use na::NumPoint;
use na::NumVector;
use na::PointAsVector;
use na::FloatPoint;
use na::FloatVector;
use na::PartialOrder;
use na::Shape;
use na::Indexable;
use na::Repeat;
use na::Dimension;
use na::Axpy;
use na::Iterable;
use na::IterableMut;
use na::Dot;
use na::Norm;
use na::Mean;
use na::Translate;
use na::Point2;
use na::Vector2;
use na::Matrix2;
use na::Point3;
use na::Vector3;
use na::Matrix3;
use na::Point4;
use na::Vector4;
use na::Matrix4;
use na::Point5;
use na::Vector5;
use na::Matrix5;
use na::Point6;
use na::Vector6;
use na::Matrix6;
use core::iter::FromIterator;
use core::ops::DerefMut;
use json::JsonValue;
use smallvec::Array;
use smallvec::IntoIter;
use ::F;
use mopa;

macro_rules! assert_eq_ulps {
    ($a:expr, $b:expr, $ulps:expr) => ({
        use float_cmp::Ulps;
        assert!($a.approx_eq_ulps(&$b, $ulps),
                "assertion failed: `(left !== right)` \
                           (left: `{:?}`, right: `{:?}`, expect ulps: `{:?}`, real ulps: `{:?}`)",
                 $a, $b, $ulps, $a.ulps(&$b).abs());
    })  
}

/// Ties a combination of ordered `TypeId`s to a value
pub type TypePairMap<V> = HashMap<(TypeId, TypeId), V>;

pub trait RemoveIf<T, C> {
    fn remove_if<F>(&mut self, f: F) -> C where F: Fn(&T) -> bool;
}

impl<T, S> RemoveIf<T, HashSet<T>> for HashSet<T, S>
    where T: Eq + Copy + Hash,
          S: ::std::hash::BuildHasher + Default
{
    fn remove_if<F>(&mut self, f: F) -> HashSet<T>
        where F: Fn(&T) -> bool
    {
        let mut removed: HashSet<T> = HashSet::default();

        for value in self.iter() {
            if f(value) {
                removed.insert(*value);
            }
        }

        for removed_value in &removed {
            self.remove(removed_value);
        }

        removed
    }
}

pub trait HasId {
    fn id_static() -> TypeId
        where Self: Sized + 'static
    {
        TypeId::of::<Self>()
    }

    fn id(&self) -> TypeId;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

#[macro_export]
macro_rules! debug_as_display_internal {
    (
        $trait_:ident
        {
            constr: [ $($constr:tt)* ],
            params: [ $($args:tt)* ],
            $($_fields:tt)*
        },
    ) => {
        as_item! {
            #[allow(dead_code)]
            impl<$($constr)*> Display for $trait_<$($args)*> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    <Self as Debug>::fmt(self, f)
                }
            }
        }
    }
}

#[macro_export]
macro_rules! debug_as_display {
    ($trait_:ident $($t:tt)*) => {
        parse_generics_shim! {
            { .. },
            then debug_as_display_internal!($trait_),
            $($t)*
        }
    }
}

#[macro_export]
macro_rules! name_as_display_internal {
    (
        $trait_:ident
        {
            constr: [ $($constr:tt)* ],
            params: [ $($args:tt)* ],
            $($_fields:tt)*
        },
    ) => {
        as_item! {
            #[allow(dead_code)]
            impl<$($constr)*> Display for $trait_<$($args)*> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, stringify!($trait_<$($constr)*>))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! name_as_display {
    ($trait_:ident $($t:tt)*) => {
        parse_generics_shim! {
            { .. },
            then name_as_display_internal!($trait_),
            $($t)*
        }
    }
}

#[macro_export]
macro_rules! has_id_internal {
    (
        $trait_:ident
        {
            constr: [ $($constr:tt)* ],
            params: [ $($args:tt)* ],
            $($_fields:tt)*
        },
    ) => {
        as_item! {
            #[allow(dead_code)]
            impl<$($constr)*> HasId for $trait_<$($args)*> {
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
        }
    }
}

#[macro_export]
macro_rules! has_id {
    ($trait_:ident $($t:tt)*) => {
        parse_generics_shim! {
            { .. },
            then has_id_internal!($trait_),
            $($t)*
        }
    }
}

pub fn combine_color<F: CustomFloat>(a: Rgba<u8>, b: Rgba<u8>, a_ratio: F) -> Rgba<u8> {
    if a_ratio <= Cast::from(0.0) {
        b
    } else if a_ratio >= Cast::from(1.0) {
        a
    } else {
        let data: Vec<u8> = a.data
            .iter()
            .zip(b.data.iter())
            .map(|(a, b)| {
                <u8 as NumCast>::from(<F as NumCast>::from(*a).unwrap() * a_ratio +
                                      <F as NumCast>::from(*b).unwrap() *
                                      (<F as NumCast>::from(1.0).unwrap() - a_ratio))
                    .unwrap()
            })
            .collect();
        Rgba { data: [data[0], data[1], data[2], data[3]] }
    }
}

pub fn combine_palette_color<F: CustomFloat>(a: palette::Rgba<F>,
                                             b: palette::Rgba<F>,
                                             a_ratio: F)
                                             -> palette::Rgba<F> {
    if a_ratio <= Cast::from(0.0) {
        b
    } else if a_ratio >= Cast::from(1.0) {
        a
    } else {
        let a_data = [a.color.red, a.color.green, a.color.blue, a.alpha];
        let b_data = [b.color.red, b.color.green, b.color.blue, b.alpha];
        let data: Vec<F> = a_data.iter()
            .zip(b_data.iter())
            .map(|(a, b)| {
                <F as NumCast>::from(*a).unwrap() * a_ratio +
                <F as NumCast>::from(*b).unwrap() * (<F as One>::one() - a_ratio)
            })
            .collect();
        palette::Rgba::new(data[0], data[1], data[2], data[3])
    }
}

pub fn remainder<T: Add<Output = T> + Rem<Output = T> + PartialOrd<T> + Zero + Copy>(a: T,
                                                                                     b: T)
                                                                                     -> T {
    let rem = a % b;

    if rem == <T as Zero>::zero() {
        <T as Zero>::zero()
    } else if a < <T as Zero>::zero() {
        b + rem
    } else {
        rem
    }
}

pub fn find_orthonormal_4<F: CustomFloat>(a: &Vector4<F>, b: &Vector4<F>, c: &Vector4<F>)
        -> Vector4<F> {
    det_copy!(Vector4::x(), Vector4::y(), Vector4::z(), Vector4::w(),
              a.x,          a.y,          a.z,          a.w,
              b.x,          b.y,          b.z,          b.w,
              c.x,          c.y,          c.z,          c.w         )
}

pub fn reorthonormalize_4<F: CustomFloat>(a: &mut Vector4<F>, b: &mut Vector4<F>,
                                          c: &mut Vector4<F>, d: &Vector4<F>) {
    *a = find_orthonormal_4(&*c, &*b, &*d);

    a.normalize_mut();

    *b = find_orthonormal_4(&*a, &*c, &*d);

    b.normalize_mut();

    *c = find_orthonormal_4(&*a, &*d, &*b);

    c.normalize_mut();
}

pub type VecLazy<'a, T> = Vec<Box<Fn() -> Option<T> + 'a>>;

pub struct IterLazy<'a, T> {
    closures: VecLazy<'a, T>,
    index: usize,
}

impl<'a, T> IterLazy<'a, T> {
    pub fn new(closures: VecLazy<'a, T>) -> IterLazy<T> {
        IterLazy {
            closures: closures,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for IterLazy<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.closures.len() {
            let closure = &self.closures[self.index];
            self.index += 1;
            return closure();
        }

        self.index += 1;

        None
    }
}

pub enum PossiblyImmediateIterator<I: Clone, A: Array<Item=I>> {
    Immediate(IntoIter<A>),
    Dynamic(Box<Iterator<Item=I>>),
}

impl<I: Clone, A: Array<Item=I>> Iterator for PossiblyImmediateIterator<I, A> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut PossiblyImmediateIterator::Immediate(ref mut small_iter) => small_iter.next(),
            &mut PossiblyImmediateIterator::Dynamic(ref mut boxed_iter) => boxed_iter.next(),
        }
    }
}

pub struct ProviderData<T: Clone, A: Array<Item=T>> {
    items: Vec<Option<T>>,
    iterator: PossiblyImmediateIterator<T, A>,
}

pub struct Provider<T: Clone, A: Array<Item=T>> {
    data: Arc<RwLock<ProviderData<T, A>>>,
}

impl<T: Clone, A: Array<Item=T>> Provider<T, A> {
    // Create an object that provides iterators which lazily compute values
    // that have not been requested yet
    pub fn new(iterator: PossiblyImmediateIterator<T, A>) -> Provider<T, A> {
        Provider {
            data: Arc::new(RwLock::new(ProviderData {
                items: Vec::new(),
                iterator,
            })),
        }
    }

    pub fn iter(&self) -> ProviderIter<T, A> {
        ProviderIter {
            index: 0,
            data: self.data.clone(),
        }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        {
            let data = self.data.read().unwrap();
            let cache_len = data.items.len();

            if index < cache_len {
                return data.items[index].clone();
            }
        };

        let mut data = self.data.write().unwrap();

        while index >= data.items.len() {
            let item = data.iterator.next();

            data.items.push(item);
        }

        data.items[index].clone()
    }
}

pub struct ProviderIter<T: Clone, A: Array<Item=T>> {
    index: usize,
    data: Arc<RwLock<ProviderData<T, A>>>,
}

impl<T: Clone, A: Array<Item=T>> Iterator for ProviderIter<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data = self.data.write().unwrap();
        let result = if self.index >= data.items.len() {
            let item = data.iterator.next();

            if item.is_some() {
                data.items.push(item.clone());
                item
            } else {
                data.items.push(None);
                None
            }
        } else {
            data.items[self.index].clone()
        };

        self.index += 1;

        result
    }
}

pub trait Mopafied: mopa::Any {}

mopafy!(Mopafied);

pub trait CustomPoint<V: CustomVector<Self>>:
// Rotate<O> +
    PartialOrder +
    Div<F, Output=Self> +
    DivAssign<F> +
    Mul<F, Output=Self> +
    MulAssign<F> +
    Add<F, Output=Self> +
    AddAssign<F> +
    Sub<F, Output=Self> +
    SubAssign<F> +
// Cast<Self> +
// Implement for generic array lengths
// AsRef<[N; 3]>
// AsMut<[N; 3]>
// From<&'a [N; 3]>
// From<&'a mut [N; 3]>
    Index<usize, Output=F> +
    IndexMut<usize, Output=F> +
    Shape<usize> +
    Indexable<usize, F> +
    Repeat<F> +
    Dimension +
    PointAsVector<Vector=V> +
    Sub<Self, Output=V> +
    Neg<Output=Self> +
    Add<V, Output=Self> +
    AddAssign<V> +
    Sub<V, Output=Self> +
    SubAssign<V> +
    ApproxEq<F> +
    FromIterator<F> +
// Bounded +
    Axpy<F> +
    Iterable<F> +
    IterableMut<F> +
// ToHomogeneous<Point4<N>>
// FromHomogeneous<Point4<N>>
    NumPoint<F> +
    FloatPoint<F> +
// Arbitrary +
// Rand +
    Display +
// Mul<UnitQuaternion<F>>
// MulAssign<UnitQuaternion<F>>
// Mul<Rotation3<F>>
// MulAssign<Rotation3<F>>
    Copy +
    Debug +
// Hash +
    Clone +
// Decodable +
// Encodable +
    PartialEq +
// Eq +
    Send +
    Sync +
    'static {}

pub trait CustomVector<P: CustomPoint<Self>>:
// Rotate<O> +
    GeneralRotation<F> +
    AngleBetween<F> +
    PartialOrder +
    Div<F, Output=Self> +
    DivAssign<F> +
    Mul<F, Output=Self> +
    MulAssign<F> +
    Add<F, Output=Self> +
    AddAssign<F> +
    Sub<F, Output=Self> +
    SubAssign<F> +
// Cast<Self> +
// Implement for generic array lengths
// AsRef<[N; 3]>
// AsMut<[N; 3]>
// From<&'a [N; 3]>
// From<&'a mut [N; 3]>
    Index<usize, Output=F> +
    IndexMut<usize, Output=F> +
    Shape<usize> +
    Indexable<usize, F> +
    Repeat<F> +
    Dimension +
    Neg<Output=Self> +
    Dot<F> +
    Norm<F> +
    Mean<F> +
    Add<Self, Output=Self> +
    AddAssign<Self> +
    Sub<Self, Output=Self> +
    SubAssign<Self> +
    ApproxEq<F> +
    FromIterator<F> +
// Bounded +
    Axpy<F> +
    Iterable<F> +
    IterableMut<F> +
// ToHomogeneous<Point4<N>>
// FromHomogeneous<Point4<N>>
    NumVector<F> +
    FloatVector<F> +
// Arbitrary +
// Rand +
    Display +
// Mul<UnitQuaternion<F>>
// MulAssign<UnitQuaternion<F>>
// Mul<Rotation3<F>>
// MulAssign<Rotation3<F>>
    Translate<P> +
// Cross<CrossProductType=Self> +
    VectorAsPoint<Point=P> +
    Copy +
    Debug +
// Hash +
    Clone +
// Decodable +
// Encodable +
    PartialEq +
// Eq +
    Send +
    Sync +
    'static {}

pub trait VectorAsPoint {
    type Point;
    fn to_point(self) -> Self::Point where Self: Sized;
    // fn as_point(&self) -> &Self::Point where Self: Sized;
    fn set_coords(&mut self, coords: Self::Point) where Self: Sized;
}

pub trait AngleBetween<F: CustomFloat> {
    fn angle_between(&self, other: &Self) -> F;
}

pub trait RankUp {
    type Type;

    fn rankup(&self) -> Self::Type;
}

pub trait Derank {
    type Type;

    fn derank(&self) -> Self::Type;
}

/// Use the Gram-Schmidt process to orthonormalize this vector in relation to the other
/// and return the rotation matrix.
// TODO: Not very ergonomic having to pass in a slice.
pub trait GeneralRotation<F: CustomFloat>: Sized {
    fn general_rotation(&self, other: &Self, angle: F, vectors_to_rotate: &mut [Self]);
}

macro_rules! dimension {
    ($point:ident, $vector:ident, $matrix:ident, $dimension:expr) => {
        impl CustomPoint<$vector<F>> for $point<F> {}
        impl CustomVector<$point<F>> for $vector<F> {}

        impl<F: CustomFloat> VectorAsPoint for $vector<F> {
            type Point = $point<F>;

            fn to_point(self) -> Self::Point {
                na::origin::<Self::Point>() + self
            }

            // fn as_point(&self) -> &Self::Point {
            //     unsafe { mem::transmute(self) }
            // }

            fn set_coords(&mut self, coords: Self::Point) {
                *self.as_mut() = *coords.as_ref();
            }
        }

        impl<F: CustomFloat> GeneralRotation<F> for $vector<F> {
            fn general_rotation(&self, other: &Self, angle: F, vectors_to_rotate: &mut [$vector<F>]) {
                let mut original = $matrix::new_identity($dimension);

                // Set the rotation plane
                original.set_column(0, *self);
                original.set_column(1, *other);

                // Orthonormalize the rotation plane vectors using the Gram-Schmidt process
                // in order to get the translation matrix
                let mut result = original;

                {
                    result.set_column(0, original.column(0));

                    for i in 1 .. $dimension {
                        for j in 0 .. i {
                            let original_column = original.column(i);
                            original.set_column(i, original_column - result.column(j) * result.column(j).dot(&original_column));
                        }

                        result.set_column(i, original.column(i).normalize());
                    }
                }

                // Create the rotation matrix that rotates translated vectors
                let mut rotation_matrix = $matrix::new_identity($dimension);
                rotation_matrix.m11 = angle.cos(); rotation_matrix.m12 = -angle.sin();
                rotation_matrix.m21 = angle.sin(); rotation_matrix.m22 = angle.cos();
                let result = result * (rotation_matrix * result.transpose());

                for vector in vectors_to_rotate {
                    *vector = result * *vector;
                }
            }
        }
    }
}

dimension!(Point2, Vector2, Matrix2, 2);
dimension!(Point3, Vector3, Matrix3, 3);
dimension!(Point4, Vector4, Matrix4, 4);
dimension!(Point5, Vector5, Matrix5, 5);
dimension!(Point6, Vector6, Matrix6, 6);

impl<F: CustomFloat> Derank for Point4<F> {
    type Type = Point3<F>;

    fn derank(&self) -> Self::Type {
        let slice = self.as_ref();
        Point3::new(slice[0], slice[1], slice[2])
    }
}

impl<F: CustomFloat> Derank for Vector4<F> {
    type Type = Vector3<F>;

    fn derank(&self) -> Self::Type {
        let slice = self.as_ref();
        Vector3::new(slice[0], slice[1], slice[2])
    }
}

impl<F: CustomFloat> RankUp for Point3<F> {
    type Type = Point4<F>;

    fn rankup(&self) -> Self::Type {
        let slice = self.as_ref();
        Point4::new(slice[0], slice[1], slice[2], <F as Zero>::zero())
    }
}

impl<F: CustomFloat> RankUp for Vector3<F> {
    type Type = Vector4<F>;

    fn rankup(&self) -> Self::Type {
        let slice = self.as_ref();
        Vector4::new(slice[0], slice[1], slice[2], <F as Zero>::zero())
    }
}

impl<F: CustomFloat, V: Dot<F> + Norm<F>> AngleBetween<F> for V {
    fn angle_between(&self, other: &Self) -> F {
        let result = F::acos(na::dot(self, other) / (self.norm() * other.norm()));

        if result.is_nan() {
            <F as Zero>::zero()
        } else {
            result
        }
    }
}

pub trait CustomFloat:
    ApproxEq<Self> +
    BaseFloat +
    NumCast +
    Num<FromStrRadixErr=ParseFloatError> +
    ApproxEq<Self> +
    UpperExp +
    LowerExp +
    Display +
    Debug +
    Default +
    Clone +
    Copy +
    PartialOrd<Self> +
    PartialEq<Self> +
    RemAssign<Self> +
    DivAssign<Self> +
    MulAssign<Self> +
    SubAssign<Self> +
    AddAssign<Self> +
    Neg<Output=Self> +
    Rem<Self, Output=Self> +
    Div<Self, Output=Self> +
    Mul<Self, Output=Self> +
    Sub<Self, Output=Self> +
    Add<Self, Output=Self> +
    From<u16> +
    From<u8> +
    From<i16> +
    From<i8> +
    One +
    Zero +
    NumCast +
    ToPrimitive +
    JsonFloat +
    Send +
    Sync +
    'static {}

impl CustomFloat for f64 {}
impl CustomFloat for f32 {}

pub trait JsonFloat {
    fn float_from_json(val: &JsonValue) -> Option<Self> where Self: Sized;
}

impl JsonFloat for f32 {
    fn float_from_json(val: &JsonValue) -> Option<Self>
        where Self: Sized
    {
        val.as_f32()
    }
}

impl JsonFloat for f64 {
    fn float_from_json(val: &JsonValue) -> Option<Self>
        where Self: Sized
    {
        val.as_f64()
    }
}

macro_rules! remove_surrounding_brackets {
    // Thanks to durka42 for this parsing algorithm for generics, much appreciated!
    // done parsing: just the outer < and > from Vec are left over
    (
        counter:   (<)                      // counter for angle brackets
        remaining: (>)                      // tokens remaining to be chomped
        processed: [ $($item_type:tt)+ ]    // already-chomped tokens

        callback: [ $callback:ident ]
        arguments_preceding: { $($arguments_preceding:tt)* }
        arguments_following: { $($arguments_following:tt)* }
    ) => {
        $callback! {
            $($arguments_preceding)*
            [ $($item_type)+ ]
            $($arguments_following)*
        }
    };

    // the next two rules implement the angle bracket counter

    // chomp a single <
    (
        counter:   ($($left:tt)*)
        remaining: (< $($rest:tt)*)
        processed: [ $($item_type:tt)* ]

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   ($($left)* <)
            remaining: ($($rest)*)
            processed: [ $($item_type)* < ]

            $($callback)*
        }
    };

    // chomp a single >
    (
        counter:   (< $($left:tt)*)
        remaining: (> $($rest:tt)*)
        processed: [ $($item_type:tt)* ]

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   ($($left)*)
            remaining: ($($rest)*)
            processed: [ $($item_type)* > ]

            $($callback)*
        }
    };

    // annoyingly, << and >> count as single tokens
    // to solve this problem, I split them and push the two individual angle brackets back onto the stream of tokens to be parsed

    // split << into < <
    (
        counter:   ($($left:tt)*)
        remaining: (<< $($rest:tt)*)
        processed: [ $($item_type:tt)* ]

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   ($($left)*)
            remaining: (< < $($rest)*)
            processed: [ $($item_type)* ]

            $($callback)*
        }
    };

    // split >> into > >
    (
        counter:   ($($left:tt)*)
        remaining: (>> $($rest:tt)*)
        processed: [ $($item_type:tt)* ]

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   ($($left)*)
            remaining: (> > $($rest)*)
            processed: [ $($item_type)* ]

            $($callback)*
        }
    };

    // chomp any non-angle-bracket token
    (
        counter:   ($($left:tt)*)
        remaining: ($first:tt $($rest:tt)*)
        processed: [ $($item_type:tt)* ]

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   ($($left)*)
            remaining: ($($rest)*)
            processed: [ $($item_type)* $first ]

            $($callback)*
        }
    };

    // Entry matcher. Note, that the result will be wrapped in `[]` brackets.
    (
        trim: [ < $($trim:tt)+ ]  // The token tree from which to remove surrounding brackets.

        $($callback:tt)*
    ) => {
        remove_surrounding_brackets! {
            counter:   (<)
            remaining: ($($trim)+)
            processed: []

            $($callback)*
        }
    };
}

#[allow(float_cmp)]
#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use na::Vector3;
    use na::Point3;
    use na::BaseFloat;
    use palette::Rgba;
    use na::ApproxEq;
    use super::*;

    #[test]
    fn remove_if() {
        let mut original: HashSet<u8> = HashSet::new();
        
        original.insert(0);
        original.insert(1);
        original.insert(2);
        original.insert(3);

        let result = original.remove_if(|val| val % 2 == 0);

        let mut retained: HashSet<u8> = HashSet::new();

        retained.insert(1);
        retained.insert(3);

        let mut removed: HashSet<u8> = HashSet::new();

        removed.insert(0);
        removed.insert(2);

        assert_eq!(original, retained);
        assert_eq!(result, removed);
    }

    #[test]
    fn combine_palette_color_test() {
        let a = combine_palette_color(Rgba::new(1.0, 0.5, 0.0, 1.0),
                                      Rgba::new(0.0, 1.0, 0.5, 0.5),
                                      1.0 / 3.0);
        let b = Rgba::new(1.0 / 3.0, 0.5 / 3.0 + 2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0 + 1.0 / 3.0);

        assert!(a.color.red.approx_eq_ulps(&b.color.red, 2));
        assert!(a.color.green.approx_eq_ulps(&b.color.green, 2));
        assert!(a.color.blue.approx_eq_ulps(&b.color.blue, 2));
        assert!(a.alpha.approx_eq_ulps(&b.alpha, 2));
    }

    #[test]
    fn remainder_test() {
        assert_eq!(remainder(-3, 3), 0);
        assert_eq!(remainder(-2, 3), 1);
        assert_eq!(remainder(-1, 3), 2);
        assert_eq!(remainder(0, 3), 0);
        assert_eq!(remainder(1, 3), 1);
        assert_eq!(remainder(2, 3), 2);
        assert_eq!(remainder(3, 3), 0);
    }

    #[test]
    fn iter_lazy() {
        let vec: VecLazy<f32> = vec! {
            Box::new(|| Some(42.0)),
            Box::new(|| None),
            Box::new(|| Some(84.0))
        };
        let mut iter = IterLazy::new(vec);

        assert_eq!(iter.next(), Some(42.0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(84.0));
        assert_eq!(iter.next(), None);
    }

    // #[test]
    // fn vector_as_point() {
    //     assert_eq! {
    //         Vector3::new(21.0, 42.0, 84.0).to_point(),
    //         Point3::new(21.0, 42.0, 84.0)
    //     }
    //     assert_eq! {
    //         Vector3::new(21.0, 42.0, 84.0).as_point(),
    //         &Point3::new(21.0, 42.0, 84.0)
    //     }

    //     let mut vector = Vector3::new(0.0, 0.0, 0.0);
        
    //     vector.set_coords(Point3::new(21.0, 42.0, 84.0));

    //     assert_eq! {
    //         vector,
    //         Vector3::new(21.0, 42.0, 84.0)
    //     }
    // }

    #[test]
    fn angle_between() {
        assert_eq_ulps! {
            Vector3::new(1.0, 0.0, 0.0).angle_between(
                &Vector3::new(0.0, 1.0, 0.0)
            ),
            <f32 as BaseFloat>::frac_pi_2(),
            2
        }
        assert_eq_ulps! {
            Vector3::new(1.0, 0.0, 0.0).angle_between(
                &Vector3::new(1.0, 1.0, 0.0)
            ),
            <f32 as BaseFloat>::frac_pi_4(),
            2
        }
        assert_eq_ulps! {
            Vector3::new(1.0, 0.0, 0.0).angle_between(
                &Vector3::new(-1.0, 1.0, 0.0)
            ),
            <f32 as BaseFloat>::frac_pi_4() * 3.0,
            2
        }
        assert_eq_ulps! {
            Vector3::new(1.0, 0.0, 0.0).angle_between(
                &Vector3::new(-1.0, 0.0, 0.0)
            ),
            <f32 as BaseFloat>::pi(),
            2
        }
    }

    #[test]
    fn remove_surrounding_brackets() {
        macro_rules! return_result {
            (
                Pre [ $($result:tt)+ ] Post
            ) => {
                stringify!($($result)+)
            }
        }

        assert_eq! {
            remove_surrounding_brackets! {
                trim: [ <Hello<World>> ]
                callback: [ return_result ]
                arguments_preceding: { Pre }
                arguments_following: { Post }
            },
            "Hello < World >"
        }
    }
}
