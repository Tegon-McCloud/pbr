use crate::accelerator::Accelerator;
use crate::scene::Scene;

pub trait Integrator {
    fn render(&self, scene: Scene);
}


pub struct Recursive<A> {
    depth: u32,
    phantom: std::marker::PhantomData<A>,
}

impl<A> Recursive<A> 
    where A: Accelerator
{
    pub fn new(depth: u32) -> Recursive<A> {
        Recursive {
            depth: depth,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<A> Integrator for Recursive<A> 
    where A: Accelerator
{
    fn render(&self,  scene: Scene) {
        let accel = A::from_scene(scene);

    }
}