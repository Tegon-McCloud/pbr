
mod trivial;
mod bvh;

use nalgebra::Vector3;
pub use trivial::Trivial;
pub use bvh::Bvh;

use crate::scene::Vertex;
use crate::geometry::Ray;

pub struct HitInfo<'a> {
    pub t: f32,
    pub vertices: [&'a Vertex; 3],
    pub barycentrics: Vector3<f32>,
    pub mesh: u32,
}

pub trait Accelerator: Sized + Send + Sync {
    fn build(geometry: Vec<(Vec<Vertex>, Vec<u32>)>) -> Self;
    fn intersect(&self, ray: &Ray) -> Option<HitInfo>;
    //fn does_intersect(&self, ray: Ray) -> bool;
}

