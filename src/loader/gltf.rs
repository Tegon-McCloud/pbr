
pub use gltf::Gltf;
pub use gltf::Glb;
use itertools::izip;
use nalgebra::Vector3;
use nalgebra::Vector4;

use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::io::{Result, Error, ErrorKind};

use super::Loader;
use crate::scene::{Scene, Node, Mesh, Vertex};

use nalgebra::{Point3, Matrix4, Quaternion, convert, try_convert, Translation3, UnitQuaternion, Scale3, Affine3};
struct GltfData(Vec<gltf::buffer::Data>, Vec<gltf::image::Data>);

impl Loader for Gltf {
    fn load_from_file(path: &Path) -> Result<Scene> {
        let (document, buffers, images) = gltf::import(path).map_err(|_| Error::from(ErrorKind::InvalidData))?;
        let data = GltfData(buffers, images);

        let gltf_scene = document.default_scene().unwrap();

        let children = gltf_scene.nodes()
            .map(|gltf_node| make_node(gltf_node, &data))
            .collect();

        let root = Node { children, ..Default::default() };

        Ok(Scene { root, ..Default::default() })
    }

    fn load_from_reader<R: Read + Seek>(_rdr: &mut R) -> Result<Scene> {
        unimplemented!()
    }
}

fn make_node(gltf_node: gltf::Node, data: &GltfData) -> Node {

    let transform = make_affine(&gltf_node.transform());

    let meshes = gltf_node.mesh().iter()
        .flat_map(|gltf_mesh| gltf_mesh.primitives())
        .map(|gltf_prim| make_mesh(gltf_prim, &data))
        .collect();

    let children = gltf_node.children()
        .map(|gltf_child| make_node(gltf_child, data))
        .collect();

    Node { transform, meshes, children, }
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

    let indices = reader.read_indices()
        .unwrap()
        .into_u32()
        .collect();

    let vertices = izip!(
        reader.read_positions().unwrap(),
        reader.read_normals().unwrap(),
        reader.read_tangents().unwrap()
    )
        .map(|(p, n, t)| Vertex {
            position: Point3::from(p),
            normal: Vector3::from(n),
            tangent: Vector4::from(t).xyz() * t[3],
        })
        .collect();

    Mesh { indices, vertices, }
}

