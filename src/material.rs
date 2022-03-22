
use std::f32::consts::FRAC_1_PI;

use nalgebra::{Vector3, Point2, Vector2};
use rand::{thread_rng, Rng};

use crate::{texture::Texture, geometry::cosine_hemisphere_map};


pub struct BrdfSample {
    pub wi: Vector3<f32>,
    pub brdf: Vector3<f32>,
    pub pdf: f32,
}

pub trait Material: Sync + Send {
    fn brdf(&self, uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Vector3<f32>;
    fn sample_brdf(&self, uv: &Point2<f32>, wo: &Vector3<f32>) -> BrdfSample;
    fn is_delta(&self, uv: &Point2<f32>) -> bool;
}

pub struct LambertianMaterial {
    color: Vector3<f32>,
    texture: Option<Texture<Vector3<f32>>>, 
}

impl LambertianMaterial {
    pub fn flat(color: &Vector3<f32>) -> Self {
        Self {
            color: color * FRAC_1_PI,
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

    fn sample_brdf(&self, uv: &Point2<f32>, wo: &Vector3<f32>) -> BrdfSample {
        let mut rng = thread_rng();
        let u = Point2::new(rng.gen(), rng.gen());
        let wi = cosine_hemisphere_map(&u);
        let pdf = wi.z * FRAC_1_PI;
        let brdf = self.brdf(uv, &wi, wo);

        BrdfSample { wi, brdf, pdf }
    }

    fn is_delta(&self, _uv: &Point2<f32>) -> bool {
        false
    }

}


pub struct GltfMaterial {
    base_color_factor: Vector3<f32>,
    metal_rough_factor: Vector2<f32>,
    base_color_texture: Option<Texture<Vector3<f32>>>,
    metal_rough_texture: Option<Texture<Vector2<f32>>>,
}

impl GltfMaterial {
    pub fn new(
        base_color_factor: Vector3<f32>,
        metal_rough_factor: Vector2<f32>,
        base_color_texture: Option<Texture<Vector3<f32>>>,
        metal_rough_texture: Option<Texture<Vector2<f32>>>,
    ) -> Self {
        Self {
            base_color_factor,
            metal_rough_factor,
            base_color_texture,
            metal_rough_texture,
        }
    }
}

// impl Material for GltfMaterial {

//     fn brdf(&self, uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Vector3<f32> {
        
//     }

// }