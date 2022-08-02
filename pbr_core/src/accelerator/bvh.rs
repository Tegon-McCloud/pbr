
use itertools::Itertools;
use nalgebra::Point3;

use crate::{
    geometry::{Ray, Bounds, triangle_intersect, triangle_centroid},
    scene::Vertex,
};

use super::{Accelerator, HitInfo};

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

#[derive(Clone, Copy)]
pub struct Triangle {
    pub mesh: u32,
    pub indices: [u32; 3],
}

pub struct Bvh {
    root: u32,
    nodes: Vec<BvhNode>,
    vertices: Vec<Vec<Vertex>>,
}

impl Bvh {
    fn split_recursive(
        meshes: &[(Vec<Vertex>, Vec<u32>)],
        tri_bounds: &[Vec<Bounds>],
        tri_centers: &[Vec<Point3<f32>>],
        tri_order: &mut [(u32, u32)],
        nodes: &mut Vec<BvhNode>
    ) -> (usize, Bounds) {

        if tri_order.len() == 1 {
            let (mesh, tri) = tri_order[0];
            let (_, indices) = &meshes[mesh as usize];
            let triangle = Triangle {
                mesh: mesh,
                indices: [
                    indices[3 * tri as usize + 0],
                    indices[3 * tri as usize + 1],
                    indices[3 * tri as usize + 2]
                ],
            };
            let bounds = tri_bounds[mesh as usize][tri as usize];

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
    
    fn intersect_recursive<'s>(&'s self, ray: &Ray, node_idx: u32, hit: &mut Option<HitInfo<'s>>) {

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
                    
                    let new_hit = triangle_intersect(&v1.position, &v2.position, &v3.position, ray);

                    if let Some((t, barycentrics)) = new_hit {
                        if hit.is_none() || t < hit.as_ref().unwrap().t {
                            *hit = Some(HitInfo { t, vertices: [v1, v2, v3], barycentrics, mesh: tri.mesh });
                        }
                    }

                }
            }
        }

    }
}

impl Accelerator for Bvh {
    fn build(meshes: Vec<(Vec<Vertex>, Vec<u32>)>) -> Self {
        let tri_bounds = meshes.iter()
            .map(|(vertices, indices)| indices 
                .chunks(3)
                .map(|idxs|
                    Bounds::around_points([
                        vertices[idxs[0] as usize].position,
                        vertices[idxs[1] as usize].position,
                        vertices[idxs[2] as usize].position,
                    ])
                )
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        let tri_centers = meshes
            .iter()
            .map(|(vertices, indices)| indices
                .chunks(3)
                .map(|idxs|
                    triangle_centroid(
                        &vertices[idxs[0] as usize].position,
                        &vertices[idxs[0] as usize].position,
                        &vertices[idxs[0] as usize].position,
                    )
                )
                .collect_vec()
            )
            .collect_vec();
        
        let mut tri_order = meshes
            .iter()
            .enumerate()
            .map(|(mesh, (_, indices))| (0..indices.len() / 3)
                .map(move |tri| (mesh as u32, tri as u32))
            )
            .flatten()
            .collect_vec();

        let mut nodes = Vec::new();
        let (root, _) = Self::split_recursive(&meshes, &tri_bounds, &tri_centers, &mut tri_order, &mut nodes);

        let vertices = meshes
            .into_iter()
            .map(|(vertices, _)| vertices)
            .collect_vec();
        
        Self { root: root as u32, nodes, vertices }

    }

    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {
        let mut info = None;
        self.intersect_recursive(ray, self.root, &mut info);
        info
    }
}