use nalgebra::Affine3;

use crate::scene::{Scene, Mesh, Vertex};
use crate::material::Material;

pub trait Accelerator {
    fn from_scene(scene: Scene) -> Self;
}

enum BvhNode {
    Interior {
        left: u32,
        right: u32,
    },
    Leaf {
        mesh_and_triangle: [u32; 4],
    },
}

pub struct Trivial {
    materials: Vec<Material>,
    mesh_to_material: Vec<u32>,
    vertices: Vec<Vec<Vertex>>,
    mesh_and_triangles: Vec<[u32; 4]>
}

impl Accelerator for Trivial {

    fn from_scene(scene: Scene) -> Self {
        let meshes = scene.get_transformed_meshes();
        
        Trivial {}
    }

}

