
use crate::scene::{Node, Vertex};
use crate::material::{Material};
use crate::geometry::{Ray, SurfacePoint, triangle_intersect};

pub trait Accelerator {
    fn from_scene_node(node: Node) -> Self;
    fn empty() -> Self;

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfacePoint)>;
    //fn does_intersect(&self, ray: Ray) -> bool;
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

    fn from_scene_node(node: Node) -> Self {
        let meshes = node.flatten();

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

    fn empty() -> Self {
        let materials = Vec::new();
        let vertices = Vec::new();
        let triangles = Vec::new();

        Trivial { materials, vertices, triangles }
    }



    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfacePoint)> {
        let (t, p, bary) = self.triangles.iter()
            .zip(self.vertices.iter())
            .map(
                |(triangles, vertices)| triangles.iter().map(
                    |triangle| [
                        &vertices[triangle[0] as usize],
                        &vertices[triangle[1] as usize],
                        &vertices[triangle[2] as usize]
                    ]
                )
            )
            .flatten()
            .map(
                |triangle| triangle_intersect(
                    &triangle[0].position,
                    &triangle[1].position,
                    &triangle[2].position,
                    &ray
                )
            )
            .flatten()
            .min_by(|(t1, _, _), (t2, _, _)| t1.total_cmp(t2))?;

        let p = SurfacePoint {
            position: p,
        };

        Some((t, p))
    }

}

