
use nalgebra::{Vector3, Point3};

use crate::{
    geometry::{Ray, SurfacePoint, Bounds, triangle_intersect, triangle_centroid},
    scene::{Node, Vertex, Mesh}
};

use super::Accelerator;

enum BvhNode {
    Interior {
        bounds: Bounds,
        left: u32,
        right: u32,
    },
    Leaf {
        bounds: Bounds,
        triangle: Triangle,
    },
}


pub struct Bvh {
    root: u32,
    nodes: Vec<BvhNode>,
    vertices: Vec<Vec<Vertex>>,
}

#[derive(Clone, Copy)]
struct Triangle {
    pub mesh: u32,
    pub indices: [u32; 3],
}

impl Bvh {
    fn split_recursive(
        meshes: &[Mesh],
        tri_bounds: &[Vec<Bounds>],
        tri_centers: &[Vec<Point3<f32>>],
        tri_order: &mut [(u32, u32)],
        nodes: &mut Vec<BvhNode>
    ) -> (usize, Bounds) {

        if tri_order.len() == 1 {
            let (mesh_idx, tri_idx) = tri_order[0];
            let mesh = &meshes[mesh_idx as usize];
            let triangle = Triangle {
                mesh: mesh_idx,
                indices: [
                    mesh.indices[3 * tri_idx as usize + 0],
                    mesh.indices[3 * tri_idx as usize + 1],
                    mesh.indices[3 * tri_idx as usize + 2]
                ],
            };
            let bounds = tri_bounds[mesh_idx as usize][tri_idx as usize];

            nodes.push(BvhNode::Leaf { bounds, triangle });
            return (nodes.len() - 1, bounds);
        }

        let center_iter = tri_order
            .iter()
            .map(|(mesh, tri)| tri_centers[*mesh as usize][*tri as usize]);
        let center_bounds = Bounds::around_points(center_iter);

        let (split_dim, _) = center_bounds.extent().argmax();
        let split_idx = tri_order.len() / 2;

        tri_order
            .select_nth_unstable_by(split_idx, |(mesh1, tri1), (mesh2, tri2)| {
                let tri1_center = tri_centers[*mesh1 as usize][*tri1 as usize][split_dim];
                let tri2_center = tri_centers[*mesh2 as usize][*tri2 as usize][split_dim];
                tri1_center.total_cmp(&tri2_center)
            });
        let (tris_left, tris_right) = tri_order.split_at_mut(split_idx);
        
        let (left, left_bounds) = Self::split_recursive(meshes, tri_bounds, tri_centers, tris_left, nodes);
        let (right, right_bounds) = Self::split_recursive(meshes, tri_bounds, tri_centers, tris_right, nodes);  
        let bounds = Bounds::around_bounds([left_bounds, right_bounds]);

        let node = BvhNode::Interior {
            bounds,
            left: left as u32,
            right: right as u32,
        };
        nodes.push(node);
        (nodes.len() - 1, bounds)
    }
    
    fn intersect_recursive(&self, ray: &Ray, node_idx: u32, hit: &mut Option<(f32, Triangle, Vector3<f32>)>) {

        match &self.nodes[node_idx as usize] {
            BvhNode::Interior{ bounds, left, right } => {
                if let Some(_) = bounds.does_intersect(ray) {
                    self.intersect_recursive(ray, *left, hit);
                    self.intersect_recursive(ray, *right, hit);
                }
            },

            BvhNode::Leaf{ bounds, triangle: tri } => {
                if let Some(_) = bounds.does_intersect(ray) {
                    let v1 = &self.vertices[tri.mesh as usize][tri.indices[0] as usize];
                    let v2 = &self.vertices[tri.mesh as usize][tri.indices[1] as usize];
                    let v3 = &self.vertices[tri.mesh as usize][tri.indices[2] as usize];
                    
                    let newhit = triangle_intersect(&v1.position, &v2.position, &v3.position, ray);

                    if let Some((t, b)) = newhit {
                        if hit.is_none() || t < hit.as_ref().unwrap().0 {
                            *hit = Some((t, *tri, b));
                        }
                    }

                }
            }
        }

    }
}

impl Accelerator for Bvh {
    fn from_scene_node(node: Node) -> Self {
        let meshes = node.flatten();

        let tri_bounds = meshes
            .iter()
            .map(|mesh| mesh.indices
                .chunks(3)
                .map(|idxs|
                    Bounds::around_points([
                        mesh.vertices[idxs[0] as usize].position,
                        mesh.vertices[idxs[1] as usize].position,
                        mesh.vertices[idxs[2] as usize].position,
                    ])
                )
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        let tri_centers = meshes
            .iter()
            .map(|mesh| mesh.indices
                .chunks(3)
                .map(|idxs|
                    triangle_centroid(
                        &mesh.vertices[idxs[0] as usize].position,
                        &mesh.vertices[idxs[0] as usize].position,
                        &mesh.vertices[idxs[0] as usize].position,
                    )
                )
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();
        
        let mut tri_order = meshes
            .iter()
            .enumerate()
            .map(|(mesh_idx, mesh)| (0..mesh.indices.len() / 3)
                .map(move |tri_idx| (mesh_idx as u32, tri_idx as u32))
            )
            .flatten()
            .collect::<Vec<_>>();

        let mut nodes = Vec::new();
        let (root, _) = Self::split_recursive(&meshes, &tri_bounds, &tri_centers, &mut tri_order, &mut nodes);

        let vertices = meshes
            .into_iter()
            .map(|mesh| mesh.vertices)
            .collect::<Vec<_>>();
        
        Self { root: root as u32, nodes, vertices }
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfacePoint)> {
        let mut hit = None;
        self.intersect_recursive(ray, self.root, &mut hit);
        hit.map(|(t, tri, b)| {
            let vertices = &self.vertices[tri.mesh as usize];
            let v1 = &vertices[tri.indices[0] as usize];
            let v2 = &vertices[tri.indices[1] as usize];
            let v3 = &vertices[tri.indices[2] as usize];

            (t, SurfacePoint::interpolate(&[v1, v2, v3], &b))
        })
    }
}