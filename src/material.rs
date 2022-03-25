
use std::f32::consts::{FRAC_1_PI, PI};

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
        self.color.component_mul(&self.texture.sample_or(uv, Vector3::from_element(1.0)))
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

    fn base_color_at(&self, uv: &Point2<f32>) -> Vector3<f32> {
        self.base_color_factor.component_mul(&self.base_color_texture.sample_or(uv, Vector3::from_element(1.0)))
    }
 
    fn metal_rough_at(&self, uv: &Point2<f32>) -> (f32, f32) {
        let mr = self.metal_rough_factor.component_mul(&self.metal_rough_texture.sample_or(uv, Vector2::new(0.0, 1.0)));
        (mr.x, mr.y)
    }


    fn facet_density(alpha2: f32, m: &Vector3<f32>) -> f32 {
        let temp = (alpha2 - 1.0) * m.z * m.z + 1.0;
        alpha2 / (PI * temp * temp)
    }

    pub fn sample_facet(alpha2: f32, o: &Vector3<f32>) -> (Vector3<f32>, f32) {
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

    fn one_way_shadowing(alpha2: f32, v: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        let costheta = v.dot(m);
        let temp = alpha2 / (costheta * costheta) - alpha2;
        
        2.0 / (1.0 + temp.sqrt())
    }

    fn fresnel(i: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        const R0: f32 = 0.04;
        R0 + (1.0 - R0) * (1.0 - i.dot(m)).powi(5)
    }

    fn shadowing(alpha2: f32, i: &Vector3<f32>, o: &Vector3<f32>, m: &Vector3<f32>) -> f32 {
        Self::one_way_shadowing(alpha2, i, m) * Self::one_way_shadowing(alpha2, o, m)
    }
}


impl Material for GltfMaterial {

    fn brdf(&self, uv: &Point2<f32>, i: &Vector3<f32>, o: &Vector3<f32>) -> Vector3<f32> {
        let (_metal, rough) = self.metal_rough_at(uv);
        let alpha = rough * rough;
        let alpha2 = alpha * alpha;

        let m = (i + o).normalize();

        let d = Self::facet_density(alpha2, &m);
        let f = Self::fresnel(i, &m);
        let g = Self::shadowing(alpha2, i, o, &m);

        let brdf = (d * g * f) / (4.0 * i.z * o.z);

        Vector3::from_element(brdf)
    }

    fn sample_brdf(&self, uv: &Point2<f32>, o: &Vector3<f32>) -> BrdfSample {
        
        let (_metal, rough) = self.metal_rough_at(uv);
        let alpha = rough * rough;
        let alpha2 = alpha * alpha;

        let (m, m_pdf) = Self::sample_facet(alpha2, o);

        let mdoto = m.dot(o);

        // if mdoto < 0.0 {
        //     println!("rough: {:?}, m: {:?}, o: {:?}", rough, m, o);
        // }

        let i = -o + 2.0 * mdoto * m;
        let pdf = m_pdf / (4.0 * mdoto);
        let brdf = self.brdf(uv, &i, o);

        BrdfSample { wi: i, brdf, pdf }
    }

    fn is_delta(&self, _uv: &Point2<f32>) -> bool {
        false
    }
}

trait MaybeTexture<T> {
    fn sample_or<U>(&self, uv: &Point2<f32>, default: U) -> U where
    U: From<T>;
}

impl<T> MaybeTexture<T> for Option<Texture<T>> where 
    T: Copy 
{
    fn sample_or<U>(&self, uv: &Point2<f32>, default: U) -> U where
        U: From<T>
    {
        match self {
            Some(texture) => texture.sample(uv),
            None => default,
        }
    }
}