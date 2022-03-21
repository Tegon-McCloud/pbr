
use std::f32::consts::FRAC_1_PI;

use nalgebra::{Vector3, Point2};

use crate::texture::Texture;

pub trait Material: Sync + Send {
    fn brdf(&self, uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Vector3<f32>;
}

pub struct LambertianMaterial {
    color: Vector3<f32>,
    texture: Option<Texture<Vector3<f32>>>, 
}

impl LambertianMaterial {
    pub fn flat(color: &Vector3<f32>) -> Self {
        Self {
            color: *color,
            texture: None
        }
    }

    pub fn textured(texture: Texture<Vector3<f32>>) -> Self {
        Self {
            color: Vector3::from_element(FRAC_1_PI),
            texture: Some(texture),
        }
    }

    pub fn textured_with_factor(factor: &Vector3<f32>, texture: Texture<Vector3<f32>>) -> Self {
        Self {
            color: factor * FRAC_1_PI,
            texture: Some(texture),
        }
    }
}

impl Material for LambertianMaterial {
    fn brdf(&self, uv: &Point2<f32>,  _wi: &Vector3<f32>, _wo: &Vector3<f32>) -> Vector3<f32> {
        self.color.component_mul(&self.texture
            .as_ref()
            .map_or(Vector3::from_element(1.0), |tex| tex.sample(uv))
        )
    }   
}


pub struct GltfMaterial {
    
}
