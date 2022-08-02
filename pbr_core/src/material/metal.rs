use std::{marker::PhantomData};

use nalgebra::Vector3;

use crate::texture::MaybeTexture;

use super::{MicrofacetDistribution, Material};


pub struct MetalMaterial<T: MicrofacetDistribution> {
    base_reflectance: Vector3<f32>,
    base_reflectance_texture: MaybeTexture<Vector3<f32>>,
    _distribution: PhantomData<T>
}

impl<T> MetalMaterial<T> where
    T: MicrofacetDistribution,
{
    fn new(color: &Vector3<f32>, texture: MaybeTexture<Vector3<f32>>) -> Self {
        Self {
            base_reflectance: color,
            base_reflectance_texture: texture,
        }
    }
}

impl<T> Material for MetalMaterial<T> where
    T: MicrofacetDistribution,
{
    fn brdf(&self, uv: &nalgebra::Point2<f32>, i: &Vector3<f32>, o: &Vector3<f32>) -> Vector3<f32> {
        let h = (i + o).normalize();
        
        
    }
}

fn fresnel(f0: &Vector3<f32>, costheta: f32) -> Vector3<f32> {

} 


