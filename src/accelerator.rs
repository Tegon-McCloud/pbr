
mod trivial;
mod bvh;

pub use trivial::Trivial;
pub use bvh::Bvh;

use crate::scene::Node;
use crate::geometry::{Ray, SurfacePoint};

pub trait Accelerator {
    fn from_scene_node(node: Node) -> Self;
    
    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfacePoint)>;
    //fn does_intersect(&self, ray: Ray) -> bool;
}

