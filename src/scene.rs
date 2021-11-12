
pub struct Vertex {
    position: [f32; 3],
}

pub struct Mesh {
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
}



pub struct Scene {



    meshes: Vec<Mesh>,
}