mod lambert;
//mod metal;
//mod gltf;

use crate::spectrum::Spectrum;

pub use self::lambert::LambertianMaterial;
//pub use self::metal::MetalMaterial;
//pub use self::gltf::GltfMaterial;

use std::f32::consts::PI;
use nalgebra::{Vector3, Point2};
use rand::{thread_rng, Rng};


pub struct BrdfSample {
    pub wi: Vector3<f32>,
    pub brdf: Spectrum<f32>,
    pub pdf: f32,
}

pub trait Material: Sync + Send {
    fn brdf(&self, uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Spectrum<f32>;
    fn sample_brdf(&self, uv: &Point2<f32>, wo: &Vector3<f32>) -> BrdfSample;
    fn is_delta(&self, uv: &Point2<f32>) -> bool;
}

pub trait MicrofacetDistribution {
    fn roughness_to_alpha(rough: f32) -> f32;
    fn facet_density(alpha: f32, m: &Vector3<f32>) -> f32;
    fn shadowing(alpha: f32, m: &Vector3<f32>, v: &Vector3<f32>) -> f32;
    fn sample_facet(alpha: f32, o: &Vector3<f32>) -> (Vector3<f32>, f32);
}

pub struct Ggx {}

impl MicrofacetDistribution for Ggx {
    fn roughness_to_alpha(rough: f32) -> f32 {
        rough * rough
    }
    
    fn facet_density(alpha: f32, m: &Vector3<f32>) -> f32 {
        let alpha2 = alpha * alpha;
        let temp = (alpha2 - 1.0) * m.z * m.z + 1.0;
        alpha2 / (PI * temp * temp)
    }

    fn shadowing(alpha: f32, m: &Vector3<f32>, v: &Vector3<f32>) -> f32 {
        let alpha2 = alpha * alpha;
        let costheta = v.dot(m);
        let temp = alpha2 / (costheta * costheta) - alpha2;
        
        2.0 / (1.0 + temp.sqrt())
    }

    fn sample_facet(alpha: f32, o: &Vector3<f32>) -> (Vector3<f32>, f32) {
        let alpha2 = alpha * alpha;
        let mut rng = thread_rng();
        let u: Point2<f32> = Point2::new(rng.gen(), rng.gen());

        let costheta = ((1.0 - u.x) / (u.x * (alpha2 - 1.0) + 1.0)).sqrt();
        let sintheta = (1.0 - costheta * costheta).sqrt();
        let phi = 2.0 * PI * u.y;

        let mut m = Vector3::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);

        if m.dot(o) < 0.0 {
            m = -m;
        }

        let pdf = Self::facet_density(alpha, &m) * costheta;

        (m, pdf)
    }
    
}