use nalgebra::Vector3;
use rayon::iter::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::geometry::Ray;
use crate::render_target::RenderTarget;
use crate::scene::Scene;

pub trait Integrator {
    fn render(&self, scene: Scene, target: &mut RenderTarget);
}


pub struct Recursive<A> {
    depth: u32,
    spp: u32,
    phantom: std::marker::PhantomData<A>,
}

impl<A> Recursive<A> 
    where A: Accelerator
{
    pub fn new(depth: u32, spp: u32) -> Recursive<A> {
        Recursive {
            depth,
            spp,
            phantom: std::marker::PhantomData,
        }
    }

    // pub fn estimate_radiance(&self, ray: &Ray, accel: &A) -> Vector3<f32> {
        
        

    // }
}

impl<A> Integrator for Recursive<A> 
    where A: Accelerator
{
    fn render(&self, scene: Scene, target: &mut RenderTarget) {
        let accel = A::from_scene(scene);

        target.pixels_mut().for_each(|px| () );
    }
}