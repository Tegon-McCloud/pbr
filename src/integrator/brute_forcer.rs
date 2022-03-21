use std::f32::consts::PI;

use nalgebra::{Vector3, Point2};
use rand::{Rng, thread_rng};
use rayon::iter::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::geometry::{Ray, cosine_hemisphere_map};
use crate::light::Emitter;
use crate::scene::Scene;
use crate::texture::RenderTarget;
use super::Integrator;

pub struct BruteForcer {
    depth: u32,
    spp: u32,
}

impl BruteForcer
{
    pub fn new(depth: u32, spp: u32) -> BruteForcer {
        BruteForcer {
            depth,
            spp,
        }
    }

    fn sample_recursive<A: Accelerator>(&self, ray: Ray, scene: &Scene<A>, depth: u32) -> Vector3<f32> {
        if depth == self.depth {
            return Vector3::from_element(0.0);
        }
        
        if let Some(p) = scene.intersect(&ray) {
            let t2w = p.tangent_to_world();
            let w2t = t2w.transpose();

            let wo = w2t * ray.direction;

            let mut rng = thread_rng();
            let u = Point2::new(rng.gen(), rng.gen());
            let wi = cosine_hemisphere_map(&u);
            let sample_pdf = wi.z / PI;
            let sample_dir = t2w * wi;

            let next_ray = Ray {
                origin: p.position + 0.0001 * p.normal,
                direction: sample_dir,
            };

            let brdf = p.brdf(&wi, &wo);

            let sample_radiance = self.sample_recursive(next_ray, scene, depth+1);

            return brdf.component_mul(&sample_radiance) * wi.z / sample_pdf;
        } else {
            let mut radiance = Vector3::from_element(0.0);

            for bg_light in scene.background_lights() {
                radiance += bg_light.emission(&ray.direction);
            }

            return radiance;
        }
        
    }

    fn sample_radiance<A: Accelerator>(&self, ray: Ray, scene: &Scene<A>) -> Vector3<f32> {
        self.sample_recursive(ray, scene, 0)
    }
}

impl Integrator for BruteForcer
{   
    fn render<A: Accelerator>(&self, scene: &Scene<A>, target: &mut RenderTarget) {
        
        target
            .pixels_par_mut()
            .for_each(|(uv, px)| {
                let mut radiance = Vector3::zeros();

                for _ in 0..self.spp {
                    let ray = scene.camera.get_ray(&uv);
                    radiance += self.sample_radiance(ray, scene);
                }

                radiance /= self.spp as f32;
                *px = radiance
            });
    }
}
