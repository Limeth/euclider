use std;
use std::collections::HashSet;
use std::hash::Hash;
use std::u8;
use core::marker::Reflect;
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
use std::convert::From;
use std::num::One;
use std::num::Zero;
use std::default::Default;
use num::Num;
use num::num_traits::ParseFloatError;
use num::traits::NumCast;
use num::traits::ToPrimitive;
use image::Rgb;
use image::Rgba;
use na::BaseFloat;
use na::Cast;
use na::ApproxEq;

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

pub fn combine_color<F: CustomFloat>(a: Rgba<u8>, b: Rgba<u8>, a_ratio: F) -> Rgba<u8> {
    if a_ratio <= Cast::from(0.0) {
        b
    } else if a_ratio >= Cast::from(1.0) {
        a
    } else {
        let data: Vec<u8> = a.data.iter().zip(b.data.iter()).map(|(a, b)| {
            <u8 as NumCast>::from(
                <F as NumCast>::from(*a).unwrap() * a_ratio + <F as NumCast>::from(*b).unwrap() * (<F as NumCast>::from(1.0).unwrap() - a_ratio)
            ).unwrap()
        }).collect();
        Rgba {
            data: [data[0], data[1], data[2], data[3]],
        }
    }
}

pub fn overlay_color<F: CustomFloat>(bottom: Rgb<u8>, top: Rgba<u8>) -> Rgb<u8> {
    if top.data[3] == 0 {
        bottom
    } else if top.data[3] == std::u8::MAX {
        let mut data = [0; 3];
        data.clone_from_slice(&top.data[..3]);
        Rgb { data: data, }
    } else {
        let u8_max_f: F = NumCast::from(std::u8::MAX).unwrap();
        let alpha: F = <F as NumCast>::from(top.data[3]).unwrap() / u8_max_f;
        Rgb {
            data: [
                <u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() * (<F as NumCast>::from(bottom.data[0]).unwrap() / u8_max_f).powi(2) + alpha * (<F as NumCast>::from(top.data[0]).unwrap() / u8_max_f)).sqrt() * <F as NumCast>::from(255.0).unwrap()).unwrap(),
                <u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() * (<F as NumCast>::from(bottom.data[1]).unwrap() / u8_max_f).powi(2) + alpha * (<F as NumCast>::from(top.data[1]).unwrap() / u8_max_f)).sqrt() * <F as NumCast>::from(255.0).unwrap()).unwrap(),
                <u8 as NumCast>::from((<F as NumCast>::from(<F as One>::one() - alpha).unwrap() * (<F as NumCast>::from(bottom.data[2]).unwrap() / u8_max_f).powi(2) + alpha * (<F as NumCast>::from(top.data[2]).unwrap() / u8_max_f)).sqrt() * <F as NumCast>::from(255.0).unwrap()).unwrap(),
            ]
        }
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
