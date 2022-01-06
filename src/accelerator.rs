use nalgebra::Affine3;

use crate::scene::{Scene, Mesh, Vertex};
use crate::material::Material;
use crate::common::Ray;

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
    vertices: Vec<Vec<Vertex>>,
    triangles: Vec<Vec<[u32; 3]>>,
}

impl Accelerator for Trivial {

    fn from_scene(scene: Scene) -> Self {
        let meshes = scene.get_transformed_meshes();
        
        let (vertices, triangles): (Vec<Vec<Vertex>>, Vec<Vec<[u32; 3]>>) = meshes.into_iter()
            .map(|mesh| (
                mesh.vertices,
                mesh.indices
                    .chunks(3)
                    .map(|idxs| [idxs[0], idxs[1], idxs[2]])
                    .collect()
            ))
            .unzip();

        Trivial {
            materials: Vec::new(),
            vertices,
            triangles,
        }
    }

}

