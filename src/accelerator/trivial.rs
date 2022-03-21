
use itertools::izip;
use crate::scene::Vertex;
use crate::material::{Material};
use crate::geometry::{Ray, triangle_intersect};
use super::{Accelerator, HitInfo};

pub struct Trivial {
    materials: Vec<Box<dyn Material>>,
    vertices: Vec<Vec<Vertex>>,
    triangles: Vec<Vec<[u32; 3]>>,
}

impl Accelerator for Trivial {
    
    fn build(meshes: Vec<(Vec<Vertex>, Vec<u32>)>) -> Self {
        let (vertices, triangles) = meshes
            .into_iter()
            .map(|(vertices, indices)| (
                vertices,
                indices
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
    
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {

        let mut info: Option<HitInfo> = None;

        for (mesh, (triangles, vertices)) in izip!(self.triangles.iter(), self.vertices.iter()).enumerate() {
            for triangle in triangles.iter() {

                let v1 = &vertices[triangle[0] as usize];
                let v2 = &vertices[triangle[1] as usize];
                let v3 = &vertices[triangle[2] as usize];

                let hit_test = triangle_intersect(&v1.position, &v2.position,&v3.position, ray);

                if let Some((t, barycentrics)) = hit_test {
                    if info.is_none() || t < info.as_ref().unwrap().t {
                        info = Some(HitInfo { t, vertices: [v1, v2, v3], barycentrics, mesh: mesh as u32 });
                    }
                }

            }
        }   

        info
    }

}

