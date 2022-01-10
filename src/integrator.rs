use nalgebra::{Vector3, Vector4, Matrix3};
use rayon::iter::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::geometry::Ray;
use crate::render_target::RenderTarget;
use crate::scene::Scene;

pub trait Integrator {
    fn render(&self, scene: Scene, target: &mut RenderTarget);
}

pub struct BruteForce<A> {
    depth: u32,
    spp: u32,
    phantom: std::marker::PhantomData<A>,
}

impl<A> BruteForce<A> 
    where A: Accelerator
{
    pub fn new(depth: u32, spp: u32) -> BruteForce<A> {
        BruteForce {
            depth,
            spp,
            phantom: std::marker::PhantomData,
        }
    }

    fn sample_radiance(&self, ray: &Ray, accel: &A) -> Vector3<f32> {
        if let Some((_t, p)) = accel.intersect(ray) {
            
            let t = p.tangent;
            let n = p.normal;
            let b = n.cross(&t);

            let world_to_tangent = Matrix3::from_columns(&[t, b, n]).transpose_mut();



            Vector3::new(1.0, 1.0, 1.0)
        } else {
            Vector3::zeros()
        }
    }
}

impl<A> Integrator for BruteForce<A> 
    where A: Accelerator + std::marker::Sync
{   
    fn render(&self, scene: Scene, target: &mut RenderTarget) {
        
        let camera = scene.camera;
        let accel = A::from_scene_node(scene.root);

        target.pixels_par_mut().for_each(|(uv, px)| {
            let mut radiance = Vector3::zeros();

            for _ in 0..self.spp {
                let ray = camera.get_ray(&uv);
                radiance += self.sample_radiance(&ray, &accel);
            }

            radiance /= self.spp as f32;
            *px = Vector4::new(radiance.x, radiance.y, radiance.z, 1.0);
        });
    }
}