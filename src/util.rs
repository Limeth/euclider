use std;
use std::collections::HashSet;
use std::hash::Hash;
use std::u8;
use image::Rgb;
use image::Rgba;

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

pub fn overlay_color(bottom: Rgb<u8>, top: Rgba<u8>) -> Rgb<u8> {
    if top.data[3] == 0 {
        bottom
    } else if top.data[3] == std::u8::MAX {
        let mut data = [0; 3];
        data.clone_from_slice(&top.data[..3]);
        Rgb { data: data, }
    } else {
        let alpha = top.data[3] as f32 / std::u8::MAX as f32;
        Rgb {
            data: [
                (((1.0 - alpha) * (bottom.data[0] as f32 / std::u8::MAX as f32).powi(2) + alpha * (top.data[0] as f32 / std::u8::MAX as f32)).sqrt() * 255.0) as u8,
                (((1.0 - alpha) * (bottom.data[1] as f32 / std::u8::MAX as f32).powi(2) + alpha * (top.data[1] as f32 / std::u8::MAX as f32)).sqrt() * 255.0) as u8,
                (((1.0 - alpha) * (bottom.data[2] as f32 / std::u8::MAX as f32).powi(2) + alpha * (top.data[2] as f32 / std::u8::MAX as f32)).sqrt() * 255.0) as u8,
            ]
        }
    }
}
