
use std::collections::btree_set::Iter;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::io::{Read, Take, Error, ErrorKind, Result};
use std::collections::HashMap;

use super::Loader;
use crate::scene::{Scene, Node, Mesh, Vertex};

use serde::{Serialize, Deserialize};
use byteorder::{LittleEndian, ReadBytesExt};
use nalgebra::{try_convert, convert, Vector3, Translation3, Scale3, UnitQuaternion, Affine3, Vector4, Vector2, Matrix2, Matrix3, Matrix4};
pub struct GltfLoader {}

impl Loader for GltfLoader {
    fn load(path: &Path) -> Result<Scene> {
        if let Some(extension) = path.extension() {
            if extension.eq_ignore_ascii_case("gltf") {
                Self::load_gltf(path)
            } else if extension.eq_ignore_ascii_case("glb") {
                Self::load_glb(path)
            } else {
                Err(Error::from(ErrorKind::InvalidInput))
            }
        } else {
            Err(Error::from(ErrorKind::InvalidInput))
        }
    }
}

impl GltfLoader {
    fn load_glb(path: &Path) -> Result<Scene> {
        let mut file = File::open(path)?;

        Self::read_and_check_glb_header(&mut file)?;

        let (gltf_chunk_type, mut gltf_chunk_rdr) = Self::take_glb_chunk(&mut file)?;
        assert_eq!(gltf_chunk_type, 0x4e4f534a);
        
        let json: GltfJson = serde_json::from_reader(&mut gltf_chunk_rdr)?;
        let mut file = gltf_chunk_rdr.into_inner();

        

        Ok(json.get_default_scene()?.unwrap())
    }

    fn load_gltf(path: &Path) -> Result<Scene> {
        let mut file = File::open(path)?;
        
        let json: GltfJson = serde_json::from_reader(&mut file)?;
        

        Ok(json.get_default_scene()?.unwrap())
    }

    fn read_and_check_glb_header<R: Read>(rdr: &mut R) -> Result<()> {
        let magic = rdr.read_u32::<LittleEndian>()?;
        assert_eq!(magic, 0x46546c67);
        let version = rdr.read_u32::<LittleEndian>()?;
        assert_eq!(version, 2);
        let _length = rdr.read_u32::<LittleEndian>()?;
        Ok(())
    }

    fn take_glb_chunk<R: Read>(mut rdr: R) -> Result<(u32, Take<R>)> {
        let chunk_length = rdr.read_u32::<LittleEndian>()?;
        let chunk_type = rdr.read_u32::<LittleEndian>()?;
        let handle = rdr.take(chunk_length as u64); 
        Ok((chunk_type, handle))
    }

}

struct Gltf {
    json: GltfJson,
    buffers: Vec<Vec<u8>>,
}

impl Gltf {
    pub fn from_json<R: Read>(json: GltfJson, path: &Path, data_chunk: Option<&mut R>) -> Result<Gltf> {
        let mut buffers = Vec::<Vec<u8>>::new();
        let dir = path.parent().unwrap();

        for gltf_buffer in json.buffers.iter() {
            let buf = vec![0u8; gltf_buffer.byte_length as usize];

            if let Some(uri) = gltf_buffer.uri {
                let buf_path = dir.join(uri);
                let buf_file = File::open(buf_path)?;
                buf_file.read(&mut buf)?;
            } else {
                let data_chunk = data_chunk.ok_or(Error::from(ErrorKind::InvalidData))?;
                data_chunk.read_exact(&mut buf)?;
            }

            buffers.push(buf);
        }

        Ok(Gltf {
            json,
            buffers,
        })
    }

    pub fn get_default_scene(&self) -> Result<Option<Scene>> {
        if let Some(scene_idx) = self.json.scene {
            Ok(Some(self.json.scenes.get(scene_idx as usize).ok_or(Error::from(ErrorKind::InvalidData))?.to_scene(&self)?))
        } else {
            Ok(None)
        }
    }
}

fn zero() -> u64 { 0 }

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfJson {
    pub scene: Option<u64>,
    #[serde(default)]
    pub scenes: Vec<GltfScene>,
    #[serde(default)]
    pub nodes: Vec<GltfNode>,
    #[serde(default)]
    pub meshes: Vec<GltfMesh>,
    #[serde(default)]
    pub accessors: Vec<GltfAccessor>,
    #[serde(rename = "bufferViews", default)]
    pub buffer_views: Vec<GltfBufferView>,
    #[serde(default)]
    pub buffers: Vec<GltfBuffer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfScene {   
    pub nodes: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfNode {
    pub children: Option<Vec<u64>>,
    pub mesh: Option<u64>,
    pub camera: Option<u64>,
    pub rotation: Option<UnitQuaternion<f32>>,
    pub scale: Option<Vector3<f32>>,
    pub translation: Option<Vector3<f32>>,
    pub matrix: Option<Matrix4<f32>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfMesh {
    pub primitives: Vec<GltfPrimitive>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfPrimitive {
    pub attributes: HashMap<String, u64>,
    pub indices: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfAccessor {
    #[serde(rename = "bufferView")]
    pub buffer_view: Option<u64>,
    #[serde(rename = "byteOffset", default = "zero")]
    pub byte_offset: u64,
    pub count: u64,
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(rename = "componentType")]
    pub component_type: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfBufferView {
    pub buffer: u64,
    #[serde(rename = "byteOffset", default = "zero")]
    pub byte_offset: u64,
    #[serde(rename = "byteLength")]
    pub byte_length: u64,
    #[serde(rename = "byteStride")]
    pub byte_stride: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GltfBuffer {
    pub uri: Option<String>,
    #[serde(rename = "byteLength")]
    pub byte_length: u64,
}

impl GltfScene {
    pub fn to_scene(&self, gltf: &Gltf) -> Result<Scene> {
        let mut scene = super::Scene::default();

        for &node_idx in self.nodes.iter().flatten() {
            let gltf_node = gltf.json.nodes
                .get(node_idx as usize)
                .ok_or(Error::from(ErrorKind::InvalidData))?;

            scene.root.children.push(gltf_node.to_node(gltf)?);
        }
        
        Ok(scene)
    }
}

impl GltfNode {
    pub fn to_node(&self, gltf: &Gltf) -> Result<Node> {
        let mut node = Node::default();
        
        node.transform = self.get_transform()?;
        node.meshes = self.get_meshes(gltf)?;
        
        for &child_idx in self.children.iter().flatten() {
            let gltf_child = gltf.json.nodes
                .get(child_idx as usize)
                .ok_or(Error::from(ErrorKind::InvalidData))?;

            node.children.push(gltf_child.to_node(gltf)?);
        }

        Ok(node)
    }

    fn get_transform(&self) -> Result<Affine3<f32>> {
        if let Some(matrix) = self.matrix {
            try_convert(matrix).ok_or(Error::from(ErrorKind::InvalidData))
        } else {
            let scale  = self.scale.unwrap_or(Vector3::<f32>::new(1.0, 1.0, 1.0));
            let rotation = self.rotation.unwrap_or(UnitQuaternion::identity());
            let translation = self.translation.unwrap_or(Vector3::<f32>::new(0.0, 0.0, 0.0));
            
            let scale = convert::<_, Affine3<f32>>(Scale3::from(scale));
            let translation = Translation3::from(translation);
            
            Ok(translation * rotation * scale)
        }
    }

    fn get_meshes(&self, gltf: &Gltf) -> Result<Vec<Mesh>> {
        if let Some(mesh_idx) = self.mesh {
            let gltf_mesh = gltf.json.meshes
                .get(mesh_idx as usize)
                .ok_or(Error::from(ErrorKind::InvalidData))?;
            Ok(gltf_mesh.to_meshes(gltf)?)
        } else {
            Ok(Vec::new())
        }
    }
}

impl GltfMesh {
    pub fn to_meshes(&self, gltf: &Gltf) -> Result<Vec<Mesh>> {
        self.primitives
            .iter()
            .map(|primitive| primitive.to_mesh(gltf))
            // .filter_map(|mesh| match mesh {
            //     Ok(mesh) => match mesh {
            //         Some(mesh) => Some(Ok(mesh)),
            //         None => None
            //     },
            //     Err(err) => Some(Err(err))
            // })
            .collect()
    }
}

impl GltfPrimitive {
    pub fn to_mesh(&self, gltf: &Gltf) -> Result<Mesh> {  
        
        let &positions_idx = self.attributes.get("POSITION").ok_or(Error::from(ErrorKind::InvalidData))?;
        // let normals_idx = self.attributes.get("NORMAL");
        // let tangents_idx = self.attributes.get("TANGENT");
        
        let positions_accessor = gltf.json.accessors
            .get(positions_idx as usize)
            .ok_or(Error::from(ErrorKind::InvalidData))?;
        
        let vertices = vec![Vertex::default(); positions_accessor.count as usize];

        Ok( Mesh{ vertices: Vec::new(), indices: Vec::new() } )
    }
}

enum AccessorComponentType {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    UnsignedInt,
    Float,
}

enum AccessorElementType {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

trait AccessorItem {
    fn from_bytes(bytes: &[u8]) {
        
    }

}

impl GltfAccessor {

    pub fn get_reader<'a, T>(&self, gltf: &'a Gltf) -> Result<AccessorIter<'a, T>> {

        Ok(AccessorIter::Dense {
            buffer: self.buffer_view.and_then(
                |buffer_view_idx| gltf.json.buffer_views.get(buffer_view_idx as usize).and_then(
                    |buffer_view| gltf.buffers.get(buffer_view.buffer as usize)
                )
            ).ok_or(Error::from(ErrorKind::InvalidData))?,
            position: self.total_offset(gltf)? as usize,
            stride: self.stride(gltf)? as usize,
            component_count: self.element_component_count()? as usize,
            component_counter: 0,
            phantom: PhantomData::<T>::default(),
        })
    }

    fn total_offset(&self, gltf: &Gltf) -> Result<u64> {
        Ok(self.byte_offset + gltf.json.buffer_views
                .get(self.buffer_view.ok_or(Error::from(ErrorKind::InvalidData))? as usize)
                .ok_or(Error::from(ErrorKind::InvalidData))?
                .byte_offset
        )
    }

    fn stride(&self, gltf: &Gltf) -> Result<u64> {
        Ok(match self.buffer_view {
            Some(buffer_view_idx) => gltf.json.buffer_views.get(buffer_view_idx as usize)
                .ok_or(Error::from(ErrorKind::InvalidData))?
                .byte_stride
                .unwrap_or(self.element_size()?),
            None => self.element_size()?,
        })
    }

    fn element_size(&self) -> Result<u64> {
        Ok(self.element_component_count()? * self.component_size()?)
    }

    fn element_component_count(&self) -> Result<u64> {
        match self.element_type.as_str() {
            "SCALAR" => Ok(1),
            "VEC2" => Ok(2),
            "VEC3" => Ok(3),
            "VEC4" => Ok(4),
            "MAT2" => Ok(4),
            "MAT3" => Ok(9),
            "MAT4" => Ok(16),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    fn component_size(&self) -> Result<u64> {
        match self.component_type {
            5120 => Ok(1),
            5121 => Ok(1),
            5122 => Ok(2),
            5123 => Ok(2),
            5125 => Ok(4),
            5126 => Ok(4),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }
}


enum AccessorIter<'a, T> {
    Dense(DenseIter<'a, T>),
}

impl<T> Iterator for AccessorIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AccessorIter::Dense(iter) => iter.next(),
        }
    }
}


struct DenseIter<'a, T> {
    buffer: &'a [u8],
    position: usize,
    stride: usize,
    _phantom: PhantomData<T>,
}

