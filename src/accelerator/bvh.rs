
use crate::{geometry::{SurfacePoint, Bounds}, scene::{Node, Vertex, Mesh}};

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

struct Triangle {
    pub mesh: u32,
    pub indices: [u32; 3],
}

impl Bvh {
    fn split_recursive(
        meshes: &[Mesh],
        triangle_bounds: &[Vec<Bounds>],
        triangle_order: &mut [(u32, u32)],
        nodes: &mut Vec<BvhNode>
    ) -> usize {

        if triangle_order.len() == 1 {
            let (mesh_idx, tri_idx) = triangle_order[0];
            let mesh = &meshes[mesh_idx as usize];
            let triangle = Triangle {
                mesh: mesh_idx,
                indices: [
                    mesh.indices[3 * tri_idx as usize + 0],
                    mesh.indices[3 * tri_idx as usize + 1],
                    mesh.indices[3 * tri_idx as usize + 2]
                ],
            };
            let bounds = triangle_bounds[mesh_idx as usize][tri_idx as usize];

            nodes.push(BvhNode::Leaf { bounds, triangle });
            return nodes.len() - 1;
        }

        let bounds_iter = triangle_order
            .iter()
            .map(|(mesh, tri)| triangle_bounds[*mesh as usize][*tri as usize]);
        
        let bounds = Bounds::around_bounds(bounds_iter.clone());
        let center_bounds = Bounds::around_points(bounds_iter.map(|b| b.center()));

        let (split_dim, _) = center_bounds.extent().argmax();
        let split_idx = triangle_order.len() / 2;

        let (tris_left, tri_mid, tris_right) = triangle_order
            .select_nth_unstable_by(split_idx, |(mesh1, tri1), (mesh2, tri2)| {
                let tri1_center = triangle_bounds[*mesh1 as usize][*tri1 as usize].center()[split_dim];
                let tri2_center = triangle_bounds[*mesh2 as usize][*tri2 as usize].center()[split_dim];
                tri1_center.total_cmp(&tri2_center)
            });

        
        
        let node = BvhNode::Interior {
            bounds,
            left: Self::split_recursive(meshes, triangle_bounds, tris_left, nodes) as u32,
            right: Self::split_recursive(meshes, triangle_bounds, tris_right, nodes) as u32,
        };
        nodes.push(node);
        nodes.len() - 1
    }
}

impl Accelerator for Bvh {
    fn from_scene_node(node: Node) -> Self {
        let meshes = node.flatten();

        let triangle_bounds = meshes
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

        let mut triangle_order = meshes
            .iter()
            .enumerate()
            .map(|(mesh_idx, mesh)| (0..mesh.indices.len() / 3)
                .map(move |tri_idx| (mesh_idx as u32, tri_idx as u32))
            )
            .flatten()
            .collect::<Vec<_>>();

        let mut nodes = Vec::new();
        let root = Self::split_recursive(&meshes, &triangle_bounds, &mut triangle_order, &mut nodes) as u32;

        let vertices = meshes
            .into_iter()
            .map(|mesh| mesh.vertices)
            .collect::<Vec<_>>();
        
        Self { root, nodes, vertices }
    }

    fn intersect(&self, ray: &crate::geometry::Ray) -> Option<(f32, SurfacePoint)> {
        
        None
    }
}