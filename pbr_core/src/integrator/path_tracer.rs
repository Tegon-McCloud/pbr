

use nalgebra::Point2;

use crate::{accelerator::Accelerator, geometry::{Ray, SurfacePoint}, scene::Scene, light::Emitter, spectrum::Spectrum};
use super::SamplingIntegrator;

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

    pub fn sample_direct<A: Accelerator>(&self, ray: &Ray, p: &SurfacePoint, scene: &Scene<A>) -> Spectrum<f32> {

        let (light, light_pdf) = scene.pick_light();

        let sample = light.sample(p);
        let pdf = sample.pdf * light_pdf;

        if sample.visibility_test.eval(&scene) {

            let w2t = p.tangent_to_world().transpose();
            let wi = w2t * sample.direction;
            let wo = w2t * -ray.direction;

            let brdf = p.brdf(&wi, &wo);

            sample.radiance * brdf * (wi.z / pdf)
        } else {
            Spectrum::black()
        }
        
    }

}

impl SamplingIntegrator for PathTracer {

    fn get_spp(&self) -> u32 {
        self.spp
    }

    fn sample<A: Accelerator>(&self, scene: &Scene<A>, xy: Point2<u32>, size: (u32, u32)) -> Spectrum<f32> {


        let mut ray = scene.get_camera().get_ray(xy, size);

        let mut radiance = Spectrum::black();
        let mut throughput = Spectrum::constant(1.0);

        for bounce in 0..self.depth {
            
            let isect = scene.intersect(&ray);

            // if there was no intersection stop bouncing.
            if isect.is_none() { 
                // if this was the camera ray, add the emission from the background (since it wasn't directly sampled)
                if bounce == 0 {
                    for bgl in scene.background_lights() {
                        radiance += bgl.emission(&ray.direction) * throughput;
                    }
                }
                break;
            }

            let p = isect.unwrap();

            radiance += self.sample_direct(&ray, &p, scene) * throughput;

            let t2w = p.tangent_to_world();
            let w2t = t2w.transpose();
            let wo = w2t * -ray.direction;
            let sample = p.sample_brdf(&wo);

            throughput = throughput * sample.brdf * (sample.wi.z / sample.pdf);
            
            ray = Ray {
                origin: p.position + 0.0001 * p.normal,
                direction: t2w * sample.wi,
            };

        }

        radiance
    }
}
