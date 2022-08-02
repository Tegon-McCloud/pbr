mod brute_forcer;
mod path_tracer;

pub use brute_forcer::BruteForcer;
pub use path_tracer::PathTracer;

use crate::accelerator::Accelerator;
use crate::texture::RenderTarget;
use crate::scene::Scene;

pub trait Integrator {
    fn render<A: Accelerator>(&self, scene: &Scene<A>, target: &mut RenderTarget);
}
