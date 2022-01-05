use crate::material::Material;
use nalgebra::{Point3, Affine3};

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Point3<f32>,
}

pub struct Mesh {
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
}

pub struct Node {
    pub children: Vec<Node>,
    pub transform: Affine3<f32>,
    pub meshes: Vec<Mesh>,
}

#[derive(Default)]
pub struct Scene {
    pub root: Node,
}

impl Scene {


    
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