
use nalgebra::Point2;

use crate::accelerator::Accelerator;
use crate::geometry::Ray;
use crate::light::Emitter;
use crate::scene::Scene;
use crate::spectrum::Spectrum;
use super::SamplingIntegrator;

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
}

impl SamplingIntegrator for BruteForcer
{   
    fn get_spp(&self) -> u32 {
        self.spp
    }

    fn sample<A: Accelerator>(&self, scene: &Scene<A>, xy: Point2<u32>, size: (u32, u32)) -> Spectrum<f32> {
        let ray = scene.get_camera().get_ray(xy, size);
        self.sample_recursive(ray, scene, 0)
    }
}
