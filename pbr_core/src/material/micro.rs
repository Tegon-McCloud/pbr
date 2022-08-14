use std::marker::PhantomData;

use nalgebra::{Vector3, Point2};

use crate::spectrum::Spectrum;

use super::{Material, BrdfSample, MicrofacetDistribution, ndot};

fn fresnel_schlick(i: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
    let pow5 = |x: f32| (x * x) * (x * x) * x;
    let r0 = 0.04;

    r0 + (1.0 - r0) * pow5(1.0 - i.dot(m)) 
}

pub struct MicrofacetMaterial<T> {
    roughness: f32,
    _marker: PhantomData<T>,
}

impl<T> MicrofacetMaterial<T> {
    pub fn new(roughness: f32) -> MicrofacetMaterial<T> {
        MicrofacetMaterial { roughness, _marker: Default::default() }
    }
}

impl<T> Material for MicrofacetMaterial<T> where
    T: MicrofacetDistribution + Send + Sync
{

    fn brdf(&self, _uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Spectrum<f32> {

        let distribution = T::new_isotropic(self.roughness);
        
        let idotn = ndot(&wi);
        let odotn = ndot(&wo);
        if idotn == 0.0 || odotn == 0.0 {
            return Spectrum::black();
        }

        let m = wi + wo;
        if m.x == 0.0 && m.y == 0.0 && m.z == 0.0 {
            return Spectrum::black();
        }
        let m = m.normalize();

        let fresnel   = fresnel_schlick(wi, &m);
        let density   = distribution.facet_density(&m);
        let shadowing = distribution.shadowing(wi, &m) * distribution.shadowing(wo, &m);

        let brdf_value = fresnel * density * shadowing / (4.0 * idotn * odotn);

        Spectrum::constant(brdf_value)
    }

    fn sample_brdf(&self, uv: &Point2<f32>, wo: &Vector3<f32>) -> BrdfSample {
        
        let distribution = T::new_isotropic(self.roughness);

        let (m, pdf_m) = distribution.sample_facet(wo);
        let mdoto = m.dot(wo);

        let wi = (2.0 * mdoto) * m - wo;

        let pdf = pdf_m / (4.0 * mdoto);
        let brdf = self.brdf(uv, &wi, wo);
        
        BrdfSample { wi, brdf, pdf }
    }

    fn is_delta(&self, _uv: &nalgebra::Point2<f32>) -> bool {
        self.roughness == 0.0
    }
    
}
