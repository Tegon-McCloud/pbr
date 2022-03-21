
use std::f32::consts::PI;

use nalgebra::{Vector3, Point2};
use rand::Rng;
use rayon::iter::ParallelIterator;

use crate::{accelerator::Accelerator, geometry::{Ray, SurfacePoint, self}, scene::Scene, light::Emitter};
use super::Integrator;

pub struct PathTracer {
    depth: u32,
    spp: u32,
}

impl PathTracer 
{
    pub fn new(depth: u32, spp: u32) -> Self {
        PathTracer {
            depth,
            spp,
        }
    }

    pub fn sample_direct<A: Accelerator>(&self, ray: &Ray, p: &SurfacePoint, scene: &Scene<A>) -> Vector3<f32> {

        let (light, light_pdf) = scene.pick_light();

        let sample = light.sample(p);
        let pdf = sample.pdf * light_pdf;

        if sample.visibility_test.eval(&scene) {

            let w2t = p.tangent_to_world().transpose();
            let wi = w2t * sample.direction;
            let wo = w2t * -ray.direction;

            let brdf = p.brdf(&wi, &wo);

            sample.radiance.component_mul(&brdf) * wi.z / pdf
        } else {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        
    }

    pub fn sample_radiance<A: Accelerator>(&self, mut ray: Ray, scene: &Scene<A>) -> Vector3<f32> {
        let mut radiance = Vector3::new(0.0, 0.0, 0.0);
        let mut throughput = Vector3::new(1.0, 1.0, 1.0);
        
        for bounce in 0..self.depth {
            
            let isect = scene.intersect(&ray);

            // if there was no intersection stop bouncing.
            if isect.is_none() { 
                // if this was the camera ray, add the emission from the background (since it wasn't directly sampled)
                if bounce == 0 {
                    for bgl in scene.background_lights() {
                        radiance += bgl.emission(&ray.direction).component_mul(&throughput);
                    }
                }
                break;
            }

            let p = isect.unwrap();

            radiance += self.sample_direct(&ray, &p, scene).component_mul(&throughput);

            let t2w = p.tangent_to_world();
            let w2t = t2w.transpose();
            
            let mut rng = rand::thread_rng();
            let u = Point2::new(rng.gen(), rng.gen());
            let wi = geometry::cosine_hemisphere_map(&u);
            let wo = w2t * ray.direction;
            let brdf = p.brdf(&wi, &wo);
            let pdf = wi.z / PI;

            throughput = throughput.component_mul(&brdf) * wi.z / pdf;
            
            ray = Ray {
                origin: p.position + 0.0001 * p.normal,
                direction: t2w * wi,
            };

        }

        radiance
    }

}

impl Integrator for PathTracer {
    fn render<A: Accelerator>(&self, scene: &Scene<A>, target: &mut crate::texture::RenderTarget) {
        target
            .pixels_par_mut()
            .for_each(|(uv, px)| {
                let mut radiance = Vector3::zeros();

                for _ in 0..self.spp {
                    let ray = scene.camera.get_ray(&uv);
                    radiance += self.sample_radiance(ray, &scene);
                }

                radiance /= self.spp as f32;
                *px = radiance
            });
    }
}
