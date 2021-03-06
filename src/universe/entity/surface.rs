use ::F;
use std;
use std::sync::Arc;
use palette::ComponentWise;
use palette::blend::PreAlpha;
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
use na::Cast;
use na::Point2;
use na::ApproxEq;
use palette::Rgb;
use na::BaseFloat;

pub type ReflectionRatioProvider<P, V> = (Fn(&TracingContext<P, V>) -> F) + Send + Sync;
pub type ReflectionDirectionProvider<P, V> = (Fn(&TracingContext<P, V>) -> V) + Send + Sync;
pub type ThresholdDirectionProvider<P, V> = (Fn(&TracingContext<P, V>) -> V) + Send + Sync;
pub type SurfaceColorProvider<P, V> = (Fn(&TracingContext<P, V>) -> Rgba<F>) + Send + Sync;
pub type ThresholdPathModifier<P, V> = (Fn(&PathTracingContext<P, V>, &mut P, &mut V)) + Send + Sync;

pub trait Surface<P: CustomPoint<V>, V: CustomVector<P>>: Send + Sync {
    fn get_color(&self, context: ColorTracingContext<P, V>) -> Rgba<F>;
    fn get_path(&self, context: PathTracingContext<P, V>) -> Option<(P, V)>;
}

pub struct ComposableSurface<P: CustomPoint<V>, V: CustomVector<P>> {
    pub reflection_ratio: Arc<ReflectionRatioProvider<P, V>>,
    pub reflection_direction: Arc<ReflectionDirectionProvider<P, V>>,
    pub threshold_direction: Arc<ThresholdDirectionProvider<P, V>>,
    pub surface_color: Arc<SurfaceColorProvider<P, V>>,
}

impl<P: CustomPoint<V>, V: CustomVector<P>> ComposableSurface<P, V> {
    fn get_reflection_ratio(&self, context: &TracingContext<P, V>) -> F {
        let reflection_ratio = self.reflection_ratio.as_ref();
        reflection_ratio(context)
    }

    fn get_reflection_direction(&self, context: &TracingContext<P, V>) -> V {
        let reflection_direction = self.reflection_direction.as_ref();
        reflection_direction(context)
    }

    fn get_surface_color(&self, context: &TracingContext<P, V>) -> Rgba<F> {
        let surface_color = self.surface_color.as_ref();
        surface_color(context)
    }

    fn get_intersection_color(&self,
                              reflection_ratio: F,
                              context: &ColorTracingContext<P, V>)
                              -> Option<Rgba<F>> {
        if reflection_ratio >= <F as One>::one() {
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

                let mut transitioned_direction = (self.threshold_direction)(&context.general);
                // Offset the new origin, so it doesn't hit the same shape over and over
                // The question is -- is there a better way? I think not.
                let new_origin = context.general.intersection.location +
                                 -context.general.intersection_normal_closer * <F as ApproxEq<F>>::approx_epsilon(None) * Cast::from(128.0);

                // Apply the material transition
                let destination_traceable = if context.general.exiting {
                    if let Some(result) = (context.material_at)(&new_origin) {
                        result
                    } else {
                        return None;
                    }
                } else {
                    context.general.intersection_traceable
                };

                context.general.origin_traceable.material().exit(&new_origin, &mut transitioned_direction);
                destination_traceable.material().enter(&new_origin, &mut transitioned_direction);

                let transition_color = trace(&context.general.time,
                                             destination_traceable,
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
                            context: &ColorTracingContext<P, V>)
                            -> Option<Rgba<F>> {
        if reflection_ratio <= <F as Zero>::zero() {
            return None;
        }

        let reflection_direction = self.get_reflection_direction(&context.general);
        let trace = context.trace;
        // Offset the new origin, so it doesn't hit the same shape over and over
        // The question is -- is there a better way? I think not.
        let new_origin = context.general.intersection.location +
                         (context.general.intersection_normal_closer
                            * <F as ApproxEq<F>>::approx_epsilon(None) * Cast::from(128.0));

        Some(trace(&context.general.time,
                   context.general.origin_traceable,
                   &new_origin,
                   &reflection_direction))
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> Surface<P, V>
        for ComposableSurface<P, V> {
    fn get_color(&self, context: ColorTracingContext<P, V>) -> Rgba<F> {
        let reflection_ratio = self.get_reflection_ratio(&context.general)
            .min(<F as One>::one())
            .max(<F as Zero>::zero());
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

    fn get_path(&self, context: PathTracingContext<P, V>) -> Option<(P, V)> {
        if *context.distance - context.general.intersection.distance <= <F as Zero>::zero() {
            None
        } else {
            let new_distance = *context.distance - context.general.intersection.distance;
            let trace = context.trace;

            // Offset the new origin, so it doesn't hit the same shape over and over
            // The question is -- is there a better way? I think not.
            let new_origin = context.general.intersection.location +
                -context.general.intersection_normal_closer * <F as ApproxEq<F>>::approx_epsilon(None) * Cast::from(128.0);

            // Apply the material transition
            let destination_traceable = if context.general.exiting {
                if let Some(result) = (context.material_at)(&new_origin) {
                    result
                } else {
                    return None;
                }
            } else {
                context.general.intersection_traceable
            };
            let mut transitioned_direction = context.general.intersection.direction;

            context.general.origin_traceable.material().exit(&new_origin, &mut transitioned_direction);
            destination_traceable.material().enter(&new_origin, &mut transitioned_direction);

            Some(trace(&context.general.time,
                       &new_distance,
                       destination_traceable,
                       &new_origin,
                       &transitioned_direction))
        }
    }
}

#[allow(unused_variables)]
pub fn reflection_ratio_uniform<P: CustomPoint<V>, V: CustomVector<P>>
    (ratio: F)
     -> Box<ReflectionRatioProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| {
        if context.exiting {
            <F as Zero>::zero()
        } else {
            ratio
        }
    })
}

#[allow(unused_variables)]
pub fn reflection_ratio_fresnel<P: CustomPoint<V>, V: CustomVector<P>>
    (refractive_index_inside: F, refractive_index_outside: F)
     -> Box<ReflectionRatioProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| {
        let normal = -context.intersection_normal_closer;
        let from_theta = context.intersection.direction.angle_between(&normal);
        let (from_index, to_index) = if context.exiting {
            (refractive_index_inside, refractive_index_outside)
        } else {
            (refractive_index_outside, refractive_index_inside)
        };
        let to_theta = ((from_index / to_index) * from_theta.sin()).asin();

        if to_theta.is_nan() {
            <F as One>::one()
        } else {
            // s-polarized light
            let product_1_s = from_index * from_theta.cos();
            let product_2_s = to_index * to_theta.cos();
            // p-polarized light
            let product_1_p = from_index * to_theta.cos();
            let product_2_p = to_index * from_theta.cos();

            let reflectance_s = ((product_1_s - product_2_s) / (product_1_s + product_2_s)).powi(2);
            let reflectance_p = ((product_1_p - product_2_p) / (product_1_p + product_2_p)).powi(2);

            // to get the reflectance of unpolarised light, we take the average
            (reflectance_s + reflectance_p) / (<F as One>::one() + <F as One>::one())
        }
    })
}

pub fn reflection_direction_specular<P: CustomPoint<V>, V: CustomVector<P>>
    ()
    -> Box<ReflectionDirectionProvider<P, V>>
{
    Box::new(move |context: &TracingContext<P, V>| {
        // R = 2*(V dot N)*N - V
        context.intersection_normal_closer * <F as NumCast>::from(-2.0).unwrap()
            * context.intersection.direction.dot(&context.intersection_normal_closer)
            + context.intersection.direction
    })
}

#[allow(unused_variables)]
pub fn threshold_direction_identity<P: CustomPoint<V>, V: CustomVector<P>>
    ()
    -> Box<ThresholdDirectionProvider<P, V>>
{
    Box::new(move |context: &TracingContext<P, V>| {
        context.intersection.direction
    })
}

pub fn threshold_direction_snell<P: CustomPoint<V>, V: CustomVector<P>>
    (refractive_index: F)
    -> Box<ThresholdDirectionProvider<P, V>>
{
    Box::new(move |context: &TracingContext<P, V>| {
        let normal = -context.intersection_normal_closer;
        let from_theta = context.intersection.direction.angle_between(&normal);
        let refractive_index_modifier = if context.exiting {
            refractive_index
        } else {
            <F as One>::one() / refractive_index
        };
        let to_theta = (refractive_index_modifier * from_theta.sin()).asin();
        let angle_delta = to_theta - from_theta;
        let mut data = [context.intersection.direction];

        normal.general_rotation(&context.intersection.direction, angle_delta, &mut data);

        data[0]
    })
}

pub type BlendFunction = (Fn(Rgba<F>, Rgba<F>) -> Rgba<F>) + Send + Sync;
pub type PaletteBlendFunction<C: Blend<Color=C> + ComponentWise> =
    (Fn(PreAlpha<C, <C as ComponentWise>::Scalar>, PreAlpha<C, <C as ComponentWise>::Scalar>)
    -> PreAlpha<C, <C as ComponentWise>::Scalar>) + Send + Sync;

pub fn surface_color_blend<P: CustomPoint<V>,
                           V: CustomVector<P>>
    (source: Box<SurfaceColorProvider<P, V>>,
     destination: Box<SurfaceColorProvider<P, V>>,
     blend_function: Box<BlendFunction>)
     -> Box<SurfaceColorProvider<P, V>> {
    let source: Arc<SurfaceColorProvider<P, V>> = source.into();
    let destination: Arc<SurfaceColorProvider<P, V>> = destination.into();
    let blend_function: Arc<BlendFunction> = blend_function.into();
    Box::new(move |context: &TracingContext<P, V>| {
        blend_function(source(context), destination(context))
    })
}

pub fn blend_function_ratio(ratio: F) -> Box<BlendFunction> {
    Box::new(move |source, destination| {
        util::combine_palette_color(source, destination, ratio)
    })
}

pub fn blend_premultiplied(blend_function: Box<PaletteBlendFunction<Rgb<F>>>)
        -> Box<BlendFunction> {
    Box::new(move |source_color: Rgba<F>, destination_color: Rgba<F>| {
        Blend::from_premultiplied(
            (&blend_function)(source_color.into_premultiplied(), destination_color.into_premultiplied())
        )
    })
}

pub fn blend_function_over() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::over))
}

pub fn blend_function_inside() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::inside))
}

pub fn blend_function_outside() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::outside))
}

pub fn blend_function_atop() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::atop))
}

pub fn blend_function_xor() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::xor))
}

pub fn blend_function_plus() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::plus))
}

pub fn blend_function_multiply() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::multiply))
}

pub fn blend_function_screen() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::screen))
}

pub fn blend_function_overlay() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::overlay))
}

pub fn blend_function_darken() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::darken))
}

pub fn blend_function_lighten() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::lighten))
}

pub fn blend_function_dodge() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::dodge))
}

pub fn blend_function_burn() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::burn))
}

pub fn blend_function_hard_light() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::hard_light))
}

pub fn blend_function_soft_light() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::soft_light))
}

pub fn blend_function_difference() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::difference))
}

pub fn blend_function_exclusion() -> Box<BlendFunction> {
    blend_premultiplied(Box::new(Blend::exclusion))
}

pub fn surface_color_illumination_directional<P: CustomPoint<V>,
                                              V: CustomVector<P>>
    (light_direction: V, light_color: Rgba<F>, dark_color: Rgba<F>)
     -> Box<SurfaceColorProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| {
        let mut normal = context.intersection.normal;

        if context.intersection.direction.angle_between(&normal) > BaseFloat::frac_pi_2() {
            normal = -normal;
        }

        let angle: F = normal.angle_between(&-light_direction);
        let ratio: F = <F as One>::one() - angle / <F as BaseFloat>::pi();

        util::combine_palette_color(dark_color, light_color, ratio)
    })
}

pub fn surface_color_illumination_global< P: CustomPoint<V>,
                                         V: CustomVector<P>>
    (light_color: Rgba<F>, dark_color: Rgba<F>)
     -> Box<SurfaceColorProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| {
        let original_angle = context.intersection_normal_closer
            .angle_between(&context.intersection.direction);
        let angle = <F as BaseFloat>::pi() - original_angle;
        let ratio = angle / <F as BaseFloat>::frac_pi_2();

        util::combine_palette_color(dark_color, light_color, ratio)
    })
}

#[allow(unused_variables)]
pub fn surface_color_uniform<P: CustomPoint<V>, V: CustomVector<P>>
    (color: Rgba<F>)
     -> Box<SurfaceColorProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| color)
}

pub type UVFn<P> = (Fn(&P) -> Point2<F>) + Send + Sync;
pub type Texture = (Fn(&Point2<F>) -> Rgba<F>) + Send + Sync;

pub fn texture_image_nearest_neighbor(dynamic_image: DynamicImage) -> Box<Texture> {
    Box::new(move |point: &Point2<F>| {
        let (width, height) = dynamic_image.dimensions();
        let (x, y) = (point.x * <F as NumCast>::from(width).unwrap(),
                      point.y * <F as NumCast>::from(height).unwrap());
        let (x, y) = (x.floor(), y.floor());
        let (x, y) = (<u32 as NumCast>::from(x).unwrap(), <u32 as NumCast>::from(y).unwrap());
        let (x, y) = (util::remainder(x, width), util::remainder(y, height));
        let pixel = dynamic_image.get_pixel(x, y);

        Rgba::new(
            <F as NumCast>::from(pixel[0]).unwrap() / <F as NumCast>::from(std::u8::MAX).unwrap(),
            <F as NumCast>::from(pixel[1]).unwrap() / <F as NumCast>::from(std::u8::MAX).unwrap(),
            <F as NumCast>::from(pixel[2]).unwrap() / <F as NumCast>::from(std::u8::MAX).unwrap(),
            <F as NumCast>::from(pixel[3]).unwrap() / <F as NumCast>::from(std::u8::MAX).unwrap(),
        )
    })
}

pub fn texture_image_linear(dynamic_image: DynamicImage) -> Box<Texture> {
    Box::new(move |point: &Point2<F>| {
        let (width, height) = dynamic_image.dimensions();
        let (x, y): (F, F) = (point.x * <F as NumCast>::from(width).unwrap() - 0.5,
                      point.y * <F as NumCast>::from(height).unwrap() - 0.5);
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

pub trait MappedTexture<P: CustomPoint<V>, V: CustomVector<P>>
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

impl<P: CustomPoint<V>, V: CustomVector<P>> MappedTexture<P, V> for MappedTextureTransparent {
    #[allow(unused_variables)]
    fn get_color(&self, point: &P) -> Rgba<F> {
        Rgba::new(Zero::zero(), Zero::zero(), Zero::zero(), Zero::zero())
    }
}

pub struct MappedTextureImpl<P: CustomPoint<V>, V: CustomVector<P>> {
    pub uvfn: Box<UVFn<P>>,
    pub texture: Box<Texture>,
    marker_vector: PhantomData<V>,
}

impl<P: CustomPoint<V>, V: CustomVector<P>> MappedTextureImpl<P, V> {
    pub fn new(uvfn: Box<UVFn<P>>, texture: Box<Texture>) -> Self {
        MappedTextureImpl {
            uvfn: uvfn,
            texture: texture,
            marker_vector: PhantomData,
        }
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> MappedTexture<P, V> for MappedTextureImpl<P, V> {
    fn get_color(&self, point: &P) -> Rgba<F> {
        let texture = &self.texture;
        let uvfn = &self.uvfn;
        texture(&uvfn(point))
    }
}

pub fn surface_color_texture<P: CustomPoint<V>, V: CustomVector<P>>
    (mapped_texture: Box<MappedTexture<P, V>>)
     -> Box<SurfaceColorProvider<P, V>> {
    Box::new(move |context: &TracingContext<P, V>| {
        mapped_texture.get_color(&context.intersection.location)
    })
}
