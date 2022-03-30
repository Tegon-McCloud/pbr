use std::f32::consts::FRAC_1_PI;

use nalgebra::{Vector3, Point2};
use rand::{thread_rng, Rng};

use crate::{texture::{FactoredTexture}, geometry::cosine_hemisphere_map, spectrum::Spectrum};

use super::{Material, BrdfSample};

pub struct LambertianMaterial {
    texture: FactoredTexture<Spectrum<f32>>,
}

impl LambertianMaterial {
    pub fn new(texture: FactoredTexture<Spectrum<f32>>) -> Self {
        let mut texture = texture;
        texture.factor *= FRAC_1_PI;
        Self { texture, }
    }

    pub fn flat(color: Spectrum<f32>) -> Self {
        let texture = FactoredTexture::new(color, None);
        Self::new(texture)
    }
}

impl Material for LambertianMaterial {
    fn brdf(&self, uv: &Point2<f32>,  _wi: &Vector3<f32>, _wo: &Vector3<f32>) -> Spectrum<f32> {
        self.texture.sample(uv)
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
