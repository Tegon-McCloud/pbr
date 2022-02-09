mod brute_forcer;
mod path_tracer;

pub use brute_forcer::BruteForcer;
pub use path_tracer::PathTracer;

use crate::render_target::RenderTarget;
use crate::scene::Scene;

pub trait Integrator {
    fn render(&self, scene: Scene, target: &mut RenderTarget);
}
