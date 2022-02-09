
use nalgebra::{Vector3, Matrix3, Point2, Vector};
use rand::Rng;
use rayon::iter::ParallelIterator;

use crate::{accelerator::Accelerator, geometry::{Ray, SurfacePoint}};
use super::Integrator;

pub struct PathTracer<A> {
    depth: u32,
    spp: u32,
    phantom: std::marker::PhantomData<A>,
}

impl<A> PathTracer<A> 
    where A: Accelerator
{
    pub fn new(depth: u32, spp: u32) -> PathTracer<A> {
        PathTracer {
            depth,
            spp,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn sample_light_ray(&self, p: &SurfacePoint) -> Ray {

    }

    pub fn sample_radiance(&self, mut ray: Ray, accel: &A) -> Vector3<f32> {
        let mut radiance = Vector3::new(0.0, 0.0, 0.0);
        let mut through_put = Vector3::new(1.0, 1.0, 1.0);
    
        for _ in 0..self.depth {
            if let Some((t, p)) = accel.intersect(&ray) {
                

            }


        }

        radiance
    }

}

impl<A> Integrator for PathTracer<A>
    where A: Accelerator + Sync
{
    fn render(&self, scene: crate::scene::Scene, target: &mut crate::render_target::RenderTarget) {
        let camera = scene.camera;
        let accel = A::from_scene_node(scene.root);

        target
            .pixels_par_mut()
            .for_each(|(uv, px)| {
                let mut radiance = Vector3::zeros();

                for _ in 0..self.spp {
                    let ray = camera.get_ray(&uv);
                    radiance += self.sample_radiance(ray, &accel);
                }

                radiance /= self.spp as f32;
                *px = radiance
            });
    }
}
