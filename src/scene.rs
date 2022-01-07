use std::ops::Mul;

use crate::material::Material;
use crate::camera::Camera;
use nalgebra::{Point3, Affine3};

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Point3<f32>,
}

#[derive(Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Node {
    pub children: Vec<Node>,
    pub transform: Affine3<f32>,
    pub meshes: Vec<Mesh>,
}

#[derive(Default)]
pub struct Scene {
    pub root: Node,
    pub camera: Camera,
}

impl Scene {

    
}

impl Node {
    pub fn flatten(self) -> Vec<Mesh> {
        let mut meshes = Vec::new();
        self.flatten_recursive(&Affine3::identity(), &mut meshes);
        meshes
    }

    fn flatten_recursive(self, parent_transform: &Affine3<f32>, meshes: &mut Vec<Mesh>) {
        let transform = parent_transform * self.transform;
        meshes.extend(self.meshes.into_iter().map(|mesh| transform * mesh));

        for child in self.children.into_iter() {
            child.flatten_recursive(&transform, meshes)
        }
    }

}

impl Default for Node {
    fn default() -> Self {
        Node {
            children: Vec::new(),
            transform: Affine3::identity(),
            meshes: Vec::new(),
        }
    }
}

impl Mul<Mesh> for Affine3<f32> {
    type Output = Mesh;
    fn mul(self, mut rhs: Mesh) -> Self::Output {
        for vertex in &mut rhs.vertices {
            vertex.position = self * vertex.position;
        }
        rhs
    }
}
