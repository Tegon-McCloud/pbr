mod brute_forcer;
mod path_tracer;

pub use brute_forcer::BruteForcer;
use nalgebra::Point2;
pub use path_tracer::PathTracer;
use rayon::prelude::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::spectrum::Spectrum;
use crate::scene::Scene;
use crate::texture::Texture;

pub trait Integrator {
    fn render<A: Accelerator>(
        &self,
        scene: &Scene<A>,
        img_size: (u32, u32),
        report_progress: impl Fn(&Texture<Spectrum<f32>>)
    ) -> Texture<Spectrum<f32>>;
}

trait SamplingIntegrator: Sync {
    fn get_spp(&self) -> u32;
    fn sample<A: Accelerator>(&self, scene: &Scene<A>, xy: Point2<u32>, size: (u32, u32)) -> Spectrum<f32>;
}

impl<T> Integrator for T
where
    T: SamplingIntegrator,
{
    fn render<A: Accelerator>(
        &self,
        scene: &Scene<A>,
        img_size: (u32, u32),
        report_progress: impl Fn(&Texture<Spectrum<f32>>)
    ) -> Texture<Spectrum<f32>> {
        let mut render_target = Texture::new(img_size.0, img_size.1, &Spectrum::black());   

        for sample_idx in 0..self.get_spp() {
            render_target.par_pixels_mut()
                .for_each(|(xy, pixel)| {
                    let new_sample = self.sample(scene, xy, img_size);
                    let prev_sum = *pixel * sample_idx as f32;
                    let new_sum = prev_sum + new_sample;
                    *pixel = new_sum / (sample_idx as f32 + 1.0);
                });

            report_progress(&render_target);
        }

        render_target
    }
}