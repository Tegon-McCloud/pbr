use std::f32::consts::PI;

use nalgebra::{Vector3, Matrix3, Point2};
use rand::Rng;
use rayon::iter::ParallelIterator;

use crate::accelerator::Accelerator;
use crate::geometry::{Ray, uniform_hemisphere_map, cosine_hemisphere_map, SurfacePoint};
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

    fn sample_ray(p: &SurfacePoint) -> Ray {
        let t = p.tangent;
        let n = p.normal;
        let b = n.cross(&t);

        let t2w = Matrix3::from_columns(&[t, b, n]);

        let mut rng = rand::thread_rng();
        let u = Point2::new(rng.gen(), rng.gen());

        let sample_dir_t = cosine_hemisphere_map(&u);
        let sample_dir_w = t2w * sample_dir_t;

        //assert!(sample_dir_w.dot(&p.normal) >= 0.0);

        Ray { origin: p.position + 0.0001 * p.normal, direction: sample_dir_w }
    }

    fn sample_radiance(&self, mut ray: Ray, accel: &A) -> Vector3<f32> {
        let mut radiance = Vector3::new(0.0, 0.0, 0.0);

        for _ in 0..self.depth {
            if let Some((_t, p)) = accel.intersect(&ray) {
                ray = Self::sample_ray(&p);
            } else {
                let light_dir = Vector3::new(1.0, 1.0, 1.0).normalize();
                let light_col = Vector3::new(1.0, 1.0, 1.0);
                let light_intensity = ray.direction.dot(&light_dir).max(0.0) * light_col;

                radiance += light_intensity;
                break;
            }
        }

        radiance
    }
}

impl<A> Integrator for BruteForce<A> 
    where A: Accelerator + std::marker::Sync
{   
    fn render(&self, scene: Scene, target: &mut RenderTarget) {
        
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