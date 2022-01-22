use itertools::izip;
use nalgebra::Vector3;

use crate::scene::{Node, Vertex};
use crate::material::{Material};
use crate::geometry::{Ray, SurfacePoint, triangle_intersect};
use super::Accelerator;

pub struct Trivial {
    materials: Vec<Material>,
    vertices: Vec<Vec<Vertex>>,
    triangles: Vec<Vec<[u32; 3]>>,
}

impl Accelerator for Trivial {

    fn from_scene_node(node: Node) -> Self {
        let meshes = node.flatten();

        let (vertices, triangles) = meshes
            .into_iter()
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
    
    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfacePoint)> {

        struct HitInfo<'a> {
            t: f32,
            b: Vector3<f32>,
            vertices: [&'a Vertex; 3],
        }

        let mut info: Option<HitInfo> = None;
        
        for (triangles, vertices) in izip!(self.triangles.iter(), self.vertices.iter()) {
            for triangle in triangles.iter() {

                let v1 = &vertices[triangle[0] as usize];
                let v2 = &vertices[triangle[1] as usize];
                let v3 = &vertices[triangle[2] as usize];

                if let Some((t, b)) = 
                    triangle_intersect(&v1.position, &v2.position,&v3.position, ray) {
                    
                    if info.is_none() || t < info.as_ref().unwrap().t {
                        info = Some(HitInfo { t, b, vertices: [v1, v2, v3] });
                    }
                }

            }
        }

        let info = info?;
        Some((info.t, SurfacePoint::interpolate(&info.vertices, &info.b)))
    }

}

