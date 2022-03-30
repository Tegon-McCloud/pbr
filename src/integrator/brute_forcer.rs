use nalgebra::{Vector3};
use rayon::iter::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::geometry::Ray;
use crate::light::Emitter;
use crate::scene::Scene;
use crate::spectrum::Spectrum;
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

    fn sample_recursive<A: Accelerator>(&self, ray: Ray, scene: &Scene<A>, depth: u32) -> Spectrum<f32> {
        if depth == self.depth {
            return Spectrum::black();
        }
        
        if let Some(p) = scene.intersect(&ray) {
            let t2w = p.tangent_to_world();
            let w2t = t2w.transpose();

            let wo = w2t * -ray.direction;
            let sample = p.sample_brdf(&wo);
            let sample_dir = t2w * sample.wi;

            let next_ray = Ray {
                origin: p.position + 0.0001 * p.normal,
                direction: sample_dir,
            };  

            let sample_radiance = self.sample_recursive(next_ray, scene, depth+1);
            
            return sample.brdf * sample_radiance * (sample.wi.z / sample.pdf);

        } else {
            let mut radiance = Spectrum::black();
            
            for bg_light in scene.background_lights() {
                radiance += bg_light.emission(&ray.direction);
            }

            return radiance;
        }
        
    }

    fn sample_radiance<A: Accelerator>(&self, ray: Ray, scene: &Scene<A>) -> Spectrum<f32> {
        self.sample_recursive(ray, scene, 0)
    }
}

impl Integrator for BruteForcer
{   
    fn render<A: Accelerator>(&self, scene: &Scene<A>, target: &mut RenderTarget) {
        
        target
            .pixels_par_mut()
            .for_each(|(uv, px)| {
                let mut radiance = Spectrum::black();
                
                for _ in 0..self.spp {
                    let ray = scene.camera.get_ray(&uv);
                    radiance += self.sample_radiance(ray, scene);
                }

                radiance /= self.spp as f32;
                *px = radiance
            });
    }
}
