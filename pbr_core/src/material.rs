mod lambert;
mod micro;
mod metal;

use crate::spectrum::Spectrum;

pub use self::lambert::LambertianMaterial;
pub use micro::MicrofacetMaterial;
pub use self::metal::MetalMaterial;


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
    fn new_isotropic(roughness: f32) -> Self;
    
    fn facet_density(&self, m: &Vector3<f32>) -> f32;
    fn shadowing(&self, v: &Vector3<f32>, m: &Vector3<f32>) -> f32;
    fn sample_facet(&self, o: &Vector3<f32>) -> (Vector3<f32>, f32);
}

pub struct Ggx {
    alpha: f32,    
}

fn ndot(v: &Vector3<f32>) -> f32 {
    v.z
}

fn heavi(a: f32) -> f32 {
    if a > 0.0 { 1.0 } else { 0.0 }
} 

impl MicrofacetDistribution for Ggx {

    fn new_isotropic(roughness: f32) -> Self {
        Self { alpha: roughness * roughness }
    }

    fn facet_density(&self, m: &Vector3<f32>) -> f32 {
        let alpha2 = self.alpha * self.alpha;
        
        let mdotn = ndot(m);
        let paren = (alpha2 - 1.0) * mdotn * mdotn + 1.0;
        
        alpha2 * heavi(mdotn) / (PI * paren * paren)
    }


    fn shadowing(&self, v: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        let alpha2 = self.alpha * self.alpha;

        let vdotm = m.dot(v);
        let vdotn = ndot(v);
        let paren = alpha2 + (1.0 - alpha2) * vdotn * vdotn;
    
        heavi(vdotm / vdotn) * 2.0 * vdotn / (vdotn + paren.sqrt())   
    }

    fn sample_facet(&self, _o: &Vector3<f32>) -> (Vector3<f32>, f32) {
        let mut rng = thread_rng();

        let u: Point2<f32> = Point2::new(rng.gen(), rng.gen());
        let theta_m = (self.alpha * (u.x / (1.0 - u.x)).sqrt()).atan();
        let phi_m = 2.0 * PI * u.y;

        let m = Vector3::new(
            theta_m.sin() * phi_m.cos(),
            theta_m.sin() * phi_m.sin(),
            theta_m.cos()
        );

        let pdf = self.facet_density(&m) * ndot(&m);

        (m, pdf)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use nalgebra::Vector3;
    use rand::Rng;

    use super::{Ggx, MicrofacetDistribution};

    #[test]
    fn ggx_facet_density_test() {
        let mut rng = rand::thread_rng();

        let distr = Ggx::new_isotropic(f32::sqrt(0.2));

        for theta in (0..100).map(|x| x as f32 * PI / 100.0) {
            let phi = 2.0 * PI * rng.gen::<f32>();

            let m = Vector3::new(
                phi.cos() * theta.sin(),
                phi.sin() * theta.sin(),
                theta.cos()
            );

            println!("({}, {}),", theta.to_degrees(), distr.facet_density(&m));            
        }
        
    }

}
