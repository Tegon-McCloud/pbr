use std::{marker::PhantomData};

use nalgebra::{Vector3, Point2};

use crate::spectrum::Spectrum;

use super::{MicrofacetDistribution, Material, BrdfSample, ndot};


pub struct MetalMaterial<T> {
    roughness: f32,
    base_reflectance: Spectrum<f32>,
    _marker: PhantomData<T>
}

impl<T> MetalMaterial<T> {
    pub fn new(roughness: f32, base_reflectance: Spectrum<f32>) -> MetalMaterial<T> {
        MetalMaterial {
            roughness,
            base_reflectance,
            _marker: Default::default()
        }
    }
}

fn conductor_fresnel(r0: &Spectrum<f32>, i: &Vector3<f32>, m: &Vector3<f32>) -> Spectrum<f32> {
    let pow5 = |x: f32| (x * x) * (x * x) * x;

    r0 + (Spectrum::constant(1.0) - r0) * pow5(1.0 - i.dot(&m))
}

impl<T> Material for MetalMaterial<T> where
    T: MicrofacetDistribution + Send + Sync,
{
    fn brdf(&self, _uv: &Point2<f32>, wi: &Vector3<f32>, wo: &Vector3<f32>) -> Spectrum<f32> {
        let m = (wi + wo).normalize();
        let distribution = T::new_isotropic(self.roughness);

        let fresnel = conductor_fresnel(&self.base_reflectance, wi, &m);
        let density = distribution.facet_density(&m);
        let shadowing = distribution.shadowing(wi, &m) * distribution.shadowing(wo, &m);

        let idotn = ndot(&wi);
        let odotn = ndot(&wo);

        fresnel * (density * shadowing / (4.0 * idotn * odotn))
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

    fn is_delta(&self, _uv: &Point2<f32>) -> bool {
        self.roughness == 0.0
    }
}

