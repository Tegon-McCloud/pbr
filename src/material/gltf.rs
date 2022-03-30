use std::f32::consts::{FRAC_1_PI, PI};

use nalgebra::{Vector3, Vector2, Point2};
use rand::{thread_rng, Rng};

use crate::{texture::Texture, geometry::cosine_hemisphere_map};

use super::{Material, BrdfSample};


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

    fn base_color_at(&self, uv: &Point2<f32>) -> Vector3<f32> {
        self.base_color_factor.component_mul(&self.base_color_texture.sample_or(uv, Vector3::from_element(1.0)))
    }
 
    fn metal_rough_at(&self, uv: &Point2<f32>) -> (f32, f32) {
        let mr = self.metal_rough_factor.component_mul(&self.metal_rough_texture.sample_or(uv, Vector2::new(0.0, 1.0)));
        (mr.x, mr.y)
    }

    fn sample_diffuse() -> (Vector3<f32>, f32) {
        let mut rng = thread_rng();
        let u = Point2::new(rng.gen(), rng.gen());
        let i = cosine_hemisphere_map(&u);
        let pdf = i.z * FRAC_1_PI;
        (i, pdf)
    }

    fn sample_specular(alpha2: f32, o: &Vector3<f32>) -> (Vector3<f32>, f32) {
        let (m, m_pdf) = Self::sample_facet(alpha2, o);

        let mdoto = m.dot(o);
        let i = -o + 2.0 * mdoto * m;
        let pdf = m_pdf / (4.0 * mdoto);
        (i, pdf)
    }

    fn sample_facet(alpha2: f32, o: &Vector3<f32>) -> (Vector3<f32>, f32) {
        let mut rng = thread_rng();
        let u: Point2<f32> = Point2::new(rng.gen(), rng.gen());

        let costheta = ((1.0 - u.x) / (u.x * (alpha2 - 1.0) + 1.0)).sqrt();
        let sintheta = (1.0 - costheta * costheta).sqrt();
        let phi = 2.0 * PI * u.y;

        let mut m = Vector3::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);

        if m.dot(o) < 0.0 {
            m = -m;
        }

        let pdf = Self::facet_density(alpha2, &m) * costheta;

        (m, pdf)
    } 

    fn facet_density(alpha2: f32, m: &Vector3<f32>) -> f32 {
        let temp = (alpha2 - 1.0) * m.z * m.z + 1.0;
        alpha2 / (PI * temp * temp)
    }

    fn one_way_shadowing(alpha2: f32, v: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        let costheta = v.dot(m);
        let temp = alpha2 / (costheta * costheta) - alpha2;
        
        2.0 / (1.0 + temp.sqrt())
    }

    fn fresnel_dielectric(i: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        const R0: f32 = 0.04;
        R0 + (1.0 - R0) * (1.0 - i.dot(m)).powi(5)
    }

    fn fresnel_conductor(i: &Vector3<f32>, m: &Vector3<f32>, f0: &Vector3<f32>) -> Vector3<f32> {
        f0 + (Vector3::from_element(1.0) - f0) * (1.0 - i.dot(m)).powi(5)
    }


    fn shadowing(alpha2: f32, i: &Vector3<f32>, o: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        Self::one_way_shadowing(alpha2, i, m) * Self::one_way_shadowing(alpha2, o, m)
    }

}


impl Material for GltfMaterial {

    fn brdf(&self, uv: &Point2<f32>, i: &Vector3<f32>, o: &Vector3<f32>) -> Vector3<f32> {
        let base_color = self.base_color_at(uv);
        let (metal, rough) = self.metal_rough_at(uv);
        let alpha = rough * rough;
        let alpha2 = alpha * alpha;

        let diffuse = base_color * FRAC_1_PI;

        let m = (i + o).normalize();

        let d = Self::facet_density(alpha2, &m);
        let g = Self::shadowing(alpha2, i, o, &m);

        let specular = (d * g) / (4.0 * i.z * o.z);
        
        let f_dielectric = Self::fresnel_dielectric(i, &m);
        let f_conductor = Self::fresnel_conductor(i, &m, &base_color);

        let dielectric = f_dielectric * Vector3::from_element(specular) + (1.0 - f_dielectric) * diffuse;
        let metallic = specular * f_conductor;

        metal * metallic + (1.0 - metal) * dielectric
    }

    

    fn sample_brdf(&self, uv: &Point2<f32>, o: &Vector3<f32>) -> BrdfSample {
        
        let (metal, rough) = self.metal_rough_at(uv);
        let alpha = rough * rough;
        let alpha2 = alpha * alpha;

        let mut rng = thread_rng();
        let u = rng.gen::<f32>();

        let i;
        let mut pdf;

        if u < metal {
            (i, pdf) = Self::sample_specular(alpha2, o);
            pdf *= metal;
        } else {
            (i, pdf) = Self::sample_diffuse();
            pdf *= 1.0 - metal;
        }

        let brdf = self.brdf(uv, &i, o);

        // let mut rng = thread_rng();
        // let u = Point2::new(rng.gen(), rng.gen());
        // let i = cosine_hemisphere_map(&u);
        // let pdf = i.z * FRAC_1_PI;
        // let brdf = self.brdf(uv, &i, o);

        BrdfSample { wi: i, brdf, pdf }
    }

    fn is_delta(&self, _uv: &Point2<f32>) -> bool {
        false
    }
}
