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
    pub fn get_transformed_meshes(self) -> Vec<Mesh> {
        fn get_meshes_recursive(node: Node, parent_transform: Affine3<f32>, meshes: &mut Vec<Mesh>) {
            let transform = parent_transform * node.transform;
            meshes.extend(node.meshes.into_iter().map(|mesh| transform * mesh));
            
            for child in node.children.into_iter() {
                get_meshes_recursive(child, transform, meshes)
            }
        }

        let mut meshes = Vec::<Mesh>::new();
        get_meshes_recursive(self.root, Affine3::identity(), &mut meshes);


        

        meshes
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
