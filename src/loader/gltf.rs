
use std::path::Path;
use std::io::{Result, Error, ErrorKind};

use super::Loader;
use crate::scene::{Scene, Node, Mesh, Vertex};

use nalgebra::{Vector4, Point3, Matrix4, convert, try_convert, Translation3, UnitQuaternion, Scale3, Affine3, Quaternion};

pub struct GltfLoader {}

struct GltfData(Vec<gltf::buffer::Data>, Vec<gltf::image::Data>);

impl Loader for GltfLoader {
    fn load(path: &Path) -> Result<Scene> {
        let (document, buffers, images) = gltf::import(path).map_err(|_| Error::from(ErrorKind::InvalidData))?;
        let data = GltfData(buffers, images);

        let gltf_scene = document.default_scene().unwrap();

        Ok(Scene {
            root: Node { 
                children: gltf_scene.nodes()
                    .map(|gltf_node| Self::make_node(gltf_node, &data))
                    .collect(),
                ..Default::default()
            }
        })

    }
}

impl GltfLoader {

    fn make_node(gltf_node: gltf::Node, data: &GltfData) -> Node {
        
        Node {
            transform: Self::make_affine(&gltf_node.transform()),
            meshes: gltf_node.mesh().iter().flat_map(|gltf_mesh| gltf_mesh.primitives())
                .map(|gltf_prim| Self::make_mesh(gltf_prim, &data))
                .collect(),
            children: gltf_node.children()
                .map(|gltf_child| Self::make_node(gltf_child, data))
                .collect(),
        }
        
    }

    fn make_affine(gltf_transform: &gltf::scene::Transform) -> Affine3<f32> {
        match gltf_transform {
            gltf::scene::Transform::Matrix{matrix} => {
                try_convert(Matrix4::from_fn(|i, j| matrix[j][i])).unwrap()
            },
            gltf::scene::Transform::Decomposed{translation: t, rotation: r, scale: s} => {
                let translation = Translation3::new(t[0], t[1], t[2]);
                let rotation = UnitQuaternion::new_unchecked(Quaternion::new(r[3],r[0], r[1], r[2]));
                let scale: Affine3<f32> = convert(Scale3::new(s[0], s[1], s[2]));
                translation * rotation * scale
            }
        }
    }

    fn make_mesh(gltf_prim: gltf::Primitive, data: &GltfData) -> Mesh {
        let reader = gltf_prim.reader(|buffer| Some(&data.0[buffer.index()]));

        Mesh {
            indices: reader.read_indices().unwrap().into_u32().collect(),
            vertices: reader.read_positions().unwrap().map(|pos| Vertex { position: Point3::from(pos) } ).collect() 
        }
    }
}

