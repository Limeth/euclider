use std;
use num::traits::NumCast;
use std::marker::PhantomData;
use palette;
use palette::Rgba;
use palette::Blend;
use image::DynamicImage;
use image::GenericImage;
use universe::entity::shape::TracingContext;
use universe::entity::shape::ColorTracingContext;
use universe::entity::shape::PathTracingContext;
use util;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use num::Zero;
use num::One;
use na;
use na::Cast;
use na::Point2;
use palette::Hsv;
use palette::RgbHue;
use palette::Alpha;
use palette::Rgb;
use na::BaseFloat;

pub type ReflectionRatioProvider<F, P, V> = Fn(&TracingContext<F, P, V>) -> F;
pub type ReflectionDirectionProvider<F, P, V> = Fn(&TracingContext<F, P, V>) -> V;
pub type SurfaceColorProvider<F, P, V> = Fn(&TracingContext<F, P, V>) -> Rgba<F>;

pub trait Surface<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    fn get_color(&self, context: ColorTracingContext<F, P, V>) -> Rgba<F>;
    fn get_path(&self, context: PathTracingContext<F, P, V>) -> Option<(P, V)>;
}

pub struct ComposableSurface<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub reflection_ratio: Box<ReflectionRatioProvider<F, P, V>>,
    pub reflection_direction: Box<ReflectionDirectionProvider<F, P, V>>,
    pub surface_color: Box<SurfaceColorProvider<F, P, V>>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableSurface<F, P, V> {
    fn get_reflection_ratio(&self, context: &TracingContext<F, P, V>) -> F {
        let reflection_ratio = self.reflection_ratio.as_ref();
        reflection_ratio(context)
    }

    fn get_reflection_direction(&self, context: &TracingContext<F, P, V>) -> V {
        let reflection_direction = self.reflection_direction.as_ref();
        reflection_direction(context)
    }

    fn get_surface_color(&self, context: &TracingContext<F, P, V>) -> Rgba<F> {
        let surface_color = self.surface_color.as_ref();
        surface_color(context)
    }

    fn get_intersection_color(&self,
                              reflection_ratio: F,
                              context: &ColorTracingContext<F, P, V>)
                              -> Option<Rgba<F>> {
        if reflection_ratio >= <F as NumCast>::from(1.0).unwrap() {
            return None;
        }

        Some({
            let surface_color = self.get_surface_color(&context.general);
            let surface_color_data: [u8; 4] = surface_color.to_pixel();
            let surface_color_alpha = surface_color_data[3];

            if surface_color_alpha == std::u8::MAX {
                surface_color
            } else {
                let trace = context.trace;

                // Offset the new origin, so it doesn't hit the same shape over and over
                // The question is -- is there a better way? I think not.
                let new_origin = context.general.intersection.location +
                                 -*context.general.intersection_normal_closer * F::epsilon() * Cast::from(128.0);

                // Apply the material transition
                let mut transitioned_direction = context.general.intersection.direction;

                context.general.origin_traceable.material().exit(&new_origin, &mut transitioned_direction);
                context.general.intersection_traceable.material().enter(&new_origin, &mut transitioned_direction);

                let transition_color = trace(context.general.time,
                      context.general.intersection_traceable,
                      &new_origin,
                      &transitioned_direction);
                let surface_palette: Rgba<F> = palette::Rgba::new_u8(surface_color_data[0],
                                                                     surface_color_data[1],
                                                                     surface_color_data[2],
                                                                     surface_color_data[3]);
                let transition_color: [u8; 4] = transition_color.to_pixel();
                let transition_palette = palette::Rgba::new_u8(transition_color[0],
                                                               transition_color[1],
                                                               transition_color[2],
                                                               transition_color[3]);

                surface_palette.over(transition_palette)
            }
        })
    }

    fn get_reflection_color(&self,
                            reflection_ratio: F,
                            context: &ColorTracingContext<F, P, V>)
                            -> Option<Rgba<F>> {
        if reflection_ratio <= <F as NumCast>::from(0.0).unwrap() {
            return None;
        }

        let reflection_direction = self.get_reflection_direction(&context.general);
        let trace = context.trace;
        // Offset the new origin, so it doesn't hit the same shape over and over
        // The question is -- is there a better way? I think not.
        let new_origin = context.general.intersection.location +
                         (reflection_direction * F::epsilon() * Cast::from(128.0));

        Some(trace(context.general.time,
                   context.general.origin_traceable,
                   &new_origin,
                   &reflection_direction))
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Surface<F, P, V>
        for ComposableSurface<F, P, V> {
    fn get_color(&self, context: ColorTracingContext<F, P, V>) -> Rgba<F> {
        let reflection_ratio = self.get_reflection_ratio(&context.general)
            .min(<F as NumCast>::from(1.0).unwrap())
            .max(<F as NumCast>::from(0.0).unwrap());
        let intersection_color: Option<Rgba<F>> =
            self.get_intersection_color(reflection_ratio, &context);
        let reflection_color: Option<Rgba<F>> =
            self.get_reflection_color(reflection_ratio, &context);

        if intersection_color.is_none() {
            return reflection_color.expect("No intersection color calculated; the reflection color should exist.");
        } else if reflection_color.is_none() {
            return intersection_color.expect("No reflection color calculated; the intersection color should exist.");
        }

        util::combine_palette_color(reflection_color.unwrap(),
                                    intersection_color.unwrap(),
                                    reflection_ratio)
    }

    fn get_path(&self, context: PathTracingContext<F, P, V>) -> Option<(P, V)> {
        let new_distance = *context.distance - context.general.intersection.distance_squared.sqrt();

        if new_distance <= <F as Zero>::zero() {
            None
        } else {
            let trace = context.trace;

            // Offset the new origin, so it doesn't hit the same shape over and over
            // The question is -- is there a better way? I think not.
            let new_origin = context.general.intersection.location +
                -*context.general.intersection_normal_closer * F::epsilon() * Cast::from(128.0);

            // Apply the material transition
            let mut transitioned_direction = context.general.intersection.direction;

            context.general.origin_traceable.material().exit(&new_origin, &mut transitioned_direction);
            context.general.intersection_traceable.material().enter(&new_origin, &mut transitioned_direction);

            Some(trace(context.general.time,
                       &new_distance,
                       context.general.intersection_traceable,
                       &new_origin,
                       &transitioned_direction))
        }
    }
}

#[allow(unused_variables)]
pub fn reflection_ratio_uniform<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (ratio: F)
     -> Box<ReflectionRatioProvider<F, P, V>> {
    Box::new(move |context: &TracingContext<F, P, V>| ratio)
}

pub fn reflection_direction_specular<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    ()
    -> Box<ReflectionDirectionProvider<F, P, V>>
{
    Box::new(move |context: &TracingContext<F, P, V>| {
        // R = 2*(V dot N)*N - V
        let mut normal = context.intersection.normal;

        if context.intersection.direction.angle_between(&normal) > BaseFloat::frac_pi_2() {
            normal = -normal;
        }

        normal * <F as NumCast>::from(-2.0).unwrap() *
        na::dot(&context.intersection.direction, &normal) + context.intersection.direction
    })
}

pub fn surface_color_illumination_directional<F: CustomFloat,
                                              P: CustomPoint<F, V>,
                                              V: CustomVector<F, P>>
    (light_direction: V)
     -> Box<SurfaceColorProvider<F, P, V>> {
    Box::new(move |context: &TracingContext<F, P, V>| {
        let mut normal = context.intersection.normal;

        if context.intersection.direction.angle_between(&normal) > BaseFloat::frac_pi_2() {
            normal = -normal;
        }

        let angle: F = normal.angle_between(&-light_direction);

        Alpha {
            color:
                Rgb::from(Hsv::new(RgbHue::from(<F as Zero>::zero()),
                                   <F as Zero>::zero(),
                                   <F as NumCast>::from(angle / <F as BaseFloat>::pi()).unwrap())),
            alpha: Cast::from(1.0),
        }
    })
}

#[allow(unused_variables)]
pub fn surface_color_uniform<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (color: Rgba<F>)
     -> Box<SurfaceColorProvider<F, P, V>> {
    Box::new(move |context: &TracingContext<F, P, V>| color)
}

pub type UVFn<F, P> = (Fn(&P) -> Point2<F>) + Send + Sync;
pub type Texture<F> = (Fn(&Point2<F>) -> Rgba<F>) + Send + Sync;

pub fn texture_image<F: CustomFloat>(dynamic_image: DynamicImage) -> Box<Texture<F>> {
    Box::new(move |point: &Point2<F>| {
        let (width, height) = dynamic_image.dimensions();
        let (x, y) = (point.x * <F as NumCast>::from(width).unwrap() - Cast::from(0.5),
                      point.y * <F as NumCast>::from(height).unwrap() - Cast::from(0.5));
        let (offset_x, offset_y) = (x - x.floor(), y - y.floor());
        let mut pixels: [[u8; 4]; 4] = [[0u8; 4]; 4];
        const PIXEL_OFFSETS: [[u32; 2]; 4] = [[0, 0], [1, 0], [0, 1], [1, 1]];

        for (index, pixel) in pixels.iter_mut().enumerate() {
            *pixel = dynamic_image.get_pixel(
                        <u32 as NumCast>::from(
                            util::remainder(x + <F as NumCast>::from(PIXEL_OFFSETS[index][0]).unwrap(),
                                            <F as NumCast>::from(width).unwrap())
                                ).unwrap(),
                        <u32 as NumCast>::from(
                            util::remainder(y + <F as NumCast>::from(PIXEL_OFFSETS[index][1]).unwrap(),
                                            <F as NumCast>::from(height).unwrap())
                                ).unwrap(),
                    ).data;
        }

        let mut data: [F; 4] = [Cast::from(0.0); 4];

        for (index, color) in data.iter_mut().enumerate() {
            *color =
                ((<F as NumCast>::from(pixels[0][index]).unwrap() * (<F as One>::one() - offset_x) +
                  <F as NumCast>::from(pixels[1][index]).unwrap() * offset_x) *
                 (<F as One>::one() - offset_y) +
                 (<F as NumCast>::from(pixels[2][index]).unwrap() * (<F as One>::one() - offset_x) +
                  <F as NumCast>::from(pixels[3][index]).unwrap() * offset_x) *
                 offset_y) / <F as NumCast>::from(std::u8::MAX).unwrap();
        }

        Rgba::new(data[0], data[1], data[2], data[3])
    })
}

pub trait MappedTexture<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    : Send + Sync {
    fn get_color(&self, point: &P) -> Rgba<F>;
}

#[derive(Default)]
pub struct MappedTextureTransparent;

impl MappedTextureTransparent {
    pub fn new() -> Self {
        MappedTextureTransparent {}
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> MappedTexture<F, P, V> for MappedTextureTransparent {
    #[allow(unused_variables)]
    fn get_color(&self, point: &P) -> Rgba<F> {
        Rgba::new(Zero::zero(), Zero::zero(), Zero::zero(), Zero::zero())
    }
}

pub struct MappedTextureImpl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub uvfn: Box<UVFn<F, P>>,
    pub texture: Box<Texture<F>>,
    marker_vector: PhantomData<V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> MappedTextureImpl<F, P, V> {
    pub fn new(uvfn: Box<UVFn<F, P>>, texture: Box<Texture<F>>) -> Self {
        MappedTextureImpl {
            uvfn: uvfn,
            texture: texture,
            marker_vector: PhantomData,
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> MappedTexture<F, P, V> for MappedTextureImpl<F, P, V> {
    fn get_color(&self, point: &P) -> Rgba<F> {
        let texture = &self.texture;
        let uvfn = &self.uvfn;
        texture(&uvfn(point))
    }
}

pub fn surface_color_texture<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (mapped_texture: Box<MappedTexture<F, P, V>>)
     -> Box<SurfaceColorProvider<F, P, V>> {
    Box::new(move |context: &TracingContext<F, P, V>| {
        mapped_texture.get_color(&context.intersection.location)
    })
}
