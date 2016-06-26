use std;
use std::any::TypeId;
use std::any::Any;
use std::collections::HashSet;
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
use std::num::One;
use std::num::Zero;
use std::default::Default;
use std::mem;
use std::cell::RefCell;
use std::cell::RefMut;
use num::Num;
use num::num_traits::ParseFloatError;
use num::traits::NumCast;
use num::traits::ToPrimitive;
use palette;
use image::Rgb;
use image::Rgba;
use na;
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
use na::Point3;
use na::Vector3;
use core::iter::FromIterator;
use core::marker::Reflect;
use core::ops::DerefMut;

pub trait RemoveIf<T, C> {
    fn remove_if<F>(&mut self, f: F) -> C where F: Fn(&T) -> bool;
}

impl<T> RemoveIf<T, HashSet<T>> for HashSet<T>
    where T: Eq + Copy + Hash
{
    fn remove_if<F>(&mut self, f: F) -> HashSet<T>
        where F: Fn(&T) -> bool
    {
        let mut removed: HashSet<T> = HashSet::new();

        for value in self.iter() {
            if f(value) {
                removed.insert(value.clone());
            }
        }

        for removed_value in &removed {
            self.remove(&removed_value);
        }

        removed
    }
}

pub trait HasId {
    fn id_static() -> TypeId
        where Self: Sized + Reflect + 'static
    {
        TypeId::of::<Self>()
    }

    fn id(&self) -> TypeId;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
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

pub fn combine_palette_color<F: CustomFloat>(a: palette::Rgba<F>, b: palette::Rgba<F>, a_ratio: F) -> palette::Rgba<F> {
    if a_ratio <= Cast::from(0.0) {
        b
    } else if a_ratio >= Cast::from(1.0) {
        a
    } else {
        let a_data = [
            a.color.red,
            a.color.green,
            a.color.blue,
            a.alpha,
        ];
        let b_data = [
            b.color.red,
            b.color.green,
            b.color.blue,
            b.alpha,
        ];
        let data: Vec<F> = a_data
            .iter()
            .zip(b_data.iter())
            .map(|(a, b)| {
                <F as NumCast>::from(*a).unwrap() * a_ratio +
                <F as NumCast>::from(*b).unwrap() *
                (<F as One>::one() - a_ratio)
            })
            .collect();
        palette::Rgba::new(data[0], data[1], data[2], data[3])
    }
}

pub fn overlay_color<F: CustomFloat>(bottom: Rgb<u8>, top: Rgba<u8>) -> Rgb<u8> {
    if top.data[3] == 0 {
        bottom
    } else if top.data[3] == std::u8::MAX {
        let mut data = [0; 3];
        data.clone_from_slice(&top.data[..3]);
        Rgb { data: data }
    } else {
        let u8_max_f: F = NumCast::from(std::u8::MAX).unwrap();
        let alpha: F = <F as NumCast>::from(top.data[3]).unwrap() / u8_max_f;
        Rgb {
            data: [<u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() *
                                          (<F as NumCast>::from(bottom.data[0]).unwrap() /
                                           u8_max_f)
                               .powi(2) +
                                          alpha *
                                          (<F as NumCast>::from(top.data[0]).unwrap() / u8_max_f))
                           .sqrt() *
                                         <F as NumCast>::from(255.0).unwrap())
                       .unwrap(),
                   <u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() *
                                          (<F as NumCast>::from(bottom.data[1]).unwrap() /
                                           u8_max_f)
                               .powi(2) +
                                          alpha *
                                          (<F as NumCast>::from(top.data[1]).unwrap() / u8_max_f))
                           .sqrt() *
                                         <F as NumCast>::from(255.0).unwrap())
                       .unwrap(),
                   <u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() *
                                          (<F as NumCast>::from(bottom.data[2]).unwrap() /
                                           u8_max_f)
                               .powi(2) +
                                          alpha *
                                          (<F as NumCast>::from(top.data[2]).unwrap() / u8_max_f))
                           .sqrt() *
                                         <F as NumCast>::from(255.0).unwrap())
                       .unwrap()],
        }
    }
}

pub struct ProviderData<T> {
    items: Vec<Option<T>>,
    iterator: Box<Iterator<Item=T>>,
}

pub struct Provider<T> {
    data: RefCell<ProviderData<T>>,
}

impl<T> Provider<T> {
    // Create an object that provides iterators which lazily compute values
    // that have not been requested yet
    pub fn new<I: Iterator<Item=T> + 'static>(a: I) -> Provider<T> {
        Provider {
        data: RefCell::new(ProviderData {
                items: Vec::new(),
                iterator: Box::new(a),
            }),
        }
    }

    pub fn iter(&self) -> Marcher<T> {
        Marcher {
            index: 0,
            provider: self.data.borrow_mut(),
        }
    }

    unsafe fn iter_index(&self, index: usize) -> Marcher<T> {
        Marcher {
            index: index,
            provider: self.data.borrow_mut(),
        }
    }
}

impl<T> Index<usize> for Provider<T> {
    type Output = Option<T>;

    fn index<'a>(&'a self, _index: usize) -> &'a Self::Output {
        let length = self.data.borrow().items.len();

        if _index >= length {
            let mut iter = unsafe { self.iter_index(length) };

            for _ in 0..(_index + 1 - length) {
                iter.next();
            }
        }

        return unsafe { mem::transmute(&self.data.borrow().items[_index]) };
    }
}

impl<T> IndexMut<usize> for Provider<T> {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut Self::Output {
        let length = self.data.borrow().items.len();

        if _index >= length {
            let mut iter = unsafe { self.iter_index(length) };

            for _ in 0..(_index + 1 - length) {
                iter.next();
            }
        }

        return unsafe { mem::transmute(&mut self.data.borrow_mut().items[_index]) };
    }
}

pub struct Marcher<'a, T: 'a> {
    index: usize,
    provider: RefMut<'a, ProviderData<T>>,
}

impl<'a, T> Iterator for Marcher<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let result: Option<Self::Item>;

        if self.index >= self.provider.items.len() {
            let item = self.provider.iterator.next();

            if item.is_some() {
                self.provider.items.push(item);
                let index = self.provider.items.len() - 1;
                let provider: &'a mut ProviderData<T> = unsafe {
                    mem::transmute(self.provider.deref_mut())
                };
                result = provider.items[index].as_mut();
            } else {
                self.provider.items.push(None);
                result = None;
            }
        } else {
            let provider: &'a mut ProviderData<T> = unsafe {
                mem::transmute(self.provider.deref_mut())
            };
            result = provider.items[self.index].as_mut();
        }

        self.index += 1;

        return result;
    }
}

pub trait CustomPoint<F: CustomFloat, V: CustomVector<F, Self>>:
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
    // Index<Output=[N]::Output> +
    // IndexMut<Output=[N]::Output> +
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
    'static {}

pub trait CustomVector<F: CustomFloat, P: CustomPoint<F, Self>>:
    // Rotate<O> +
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
    // Index<Output=[N]::Output> +
    // IndexMut<Output=[N]::Output> +
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
    VectorAsPoint +
    Copy +
    Debug +
    // Hash +
    Clone +
    // Decodable +
    // Encodable +
    PartialEq +
    // Eq +
    'static {}

pub trait VectorAsPoint {
    type Point;
    fn to_point(self) -> Self::Point where Self: Sized;
    fn as_point(&self) -> &Self::Point where Self: Sized;
    fn set_coords(&mut self, coords: Self::Point) where Self: Sized;
}

pub trait AngleBetween<F: CustomFloat> {
    fn angle_between(&self, other: &Self) -> F;
}

impl<F: CustomFloat> CustomPoint<F, Vector3<F>> for Point3<F> {}
impl<F: CustomFloat> CustomVector<F, Point3<F>> for Vector3<F> {}

impl<F: CustomFloat> VectorAsPoint for Vector3<F> {
    type Point = Point3<F>;

    fn to_point(self) -> Self::Point {
        na::origin::<Self::Point>() + self
    }

    fn as_point(&self) -> &Self::Point {
        unsafe {
            mem::transmute(self)
        }
    }

    fn set_coords(&mut self, coords: Self::Point) {
        self.x = coords.x;
        self.y = coords.y;
        self.z = coords.z;
    }
}

impl<F: CustomFloat> AngleBetween<F> for Vector3<F> {
    fn angle_between(&self, other: &Self) -> F {
        F::acos(
            na::dot(self, other) / (self.norm() * other.norm())
        )
    }
}

pub trait CustomFloat:
    BaseFloat +
    Consts +
    NumCast +
    Num<FromStrRadixErr=ParseFloatError> +
    ApproxEq<Self> +
    Reflect +
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
    'static {}

impl CustomFloat for f64 {}
impl CustomFloat for f32 {}

pub trait Consts {
    fn epsilon() -> Self;
}

impl Consts for f64 {
    fn epsilon() -> Self {
        std::f64::EPSILON
    }
}

impl Consts for f32 {
    fn epsilon() -> Self {
        std::f32::EPSILON
    }
}
