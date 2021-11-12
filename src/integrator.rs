use crate::accelerator::Accelerator;
use crate::scene::Scene;

pub trait Integrator {
    fn render(&self, scene: Scene);
}


pub struct RecursiveIntegrator<A> {
    depth: u32,
    phantom: std::marker::PhantomData<A>,
}

impl<A> RecursiveIntegrator<A> 
    where A: Accelerator
{
    pub fn new(depth: u32) -> RecursiveIntegrator<A> {
        RecursiveIntegrator {
            depth: depth,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<A> Integrator for RecursiveIntegrator<A> 
    where A: Accelerator
{
    fn render(&self,  scene: Scene) {
        let accel = A::from_scene(scene);

    }
}