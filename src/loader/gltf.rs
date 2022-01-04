use std::default;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Take, Seek, SeekFrom, Error, ErrorKind, Result};

use super::Loader;
use crate::scene::{Scene, Node, Mesh, Vertex};

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
        
        let gltf: json::Gltf = serde_json::from_reader(&mut gltf_chunk_rdr)?;
        let mut file = gltf_chunk_rdr.into_inner();

        gltf.get_default_scene()
    }

    fn load_gltf(path: &Path) -> Result<Scene> {
        let mut file = File::open(path)?;
        
        let gltf: json::Gltf = serde_json::from_reader(&mut file)?;


        gltf.get_default_scene()
    }

    fn read_and_check_glb_header<R: ReadBytesExt>(rdr: &mut R) -> Result<()> {
        let magic = rdr.read_u32::<LittleEndian>()?;
        assert_eq!(magic, 0x46546c67);
        let version = rdr.read_u32::<LittleEndian>()?;
        assert_eq!(version, 2);
        let _length = rdr.read_u32::<LittleEndian>()?;
        Ok(())
    }

    fn take_glb_chunk<R: ReadBytesExt>(mut rdr: R) -> Result<(u32, Take<R>)> {
        let chunk_length = rdr.read_u32::<LittleEndian>()?;
        let chunk_type = rdr.read_u32::<LittleEndian>()?;
        let handle = rdr.take(chunk_length as u64); 
        Ok((chunk_type, handle))
    }

}

mod json {
    use std::collections::HashMap;

    use serde::{Serialize, Deserialize};
    use nalgebra::{Vector3, UnitQuaternion, Matrix4};

    fn zero() -> u64 { 0 }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Gltf {
        pub scene: Option<u64>,
        pub scenes: Option<Vec<Scene>>,
        pub nodes: Option<Vec<Node>>,
        pub meshes: Option<Vec<Mesh>>,
        pub accessors: Option<Vec<Accessor>>,
        #[serde(rename = "bufferViews")]
        pub buffer_views: Option<Vec<BufferView>>,
        pub buffers: Option<Vec<Buffer>>,
        #[serde(skip)]
        pub storage: Vec<Vec<u8>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Scene {   
        pub nodes: Option<Vec<u64>>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Node {
        pub children: Option<Vec<u64>>,
        pub mesh: Option<u64>,
        pub camera: Option<u64>,
        pub rotation: Option<UnitQuaternion<f32>>,
        pub scale: Option<Vector3<f32>>,
        pub translation: Option<Vector3<f32>>,
        pub matrix: Option<Matrix4<f32>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Mesh {
        pub primitives: Vec<Primitive>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Primitive {
        pub attributes: HashMap<String, u64>,
        pub indices: Option<u64>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Accessor {
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
    pub struct BufferView {
        pub buffer: u64,
        #[serde(rename = "byteOffset", default = "zero")]
        pub byte_offset: u64,
        #[serde(rename = "byteLength")]
        pub byte_length: u64,
        #[serde(rename = "byteStride")]
        pub byte_stride: Option<u64>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Buffer {
        pub uri: Option<String>,
        #[serde(rename = "byteLength")]
        pub byte_length: u64,
    }
}


impl json::Gltf {
    pub fn get_default_scene(&self) -> Result<Scene> {
        if let Some(scenes) = &self.scenes {
            let scene_idx = self.scene.unwrap_or(0) as usize;
            scenes.get(scene_idx).ok_or(Error::from(ErrorKind::InvalidData))?.to_scene(&self)
        } else {
            Err(Error::from(ErrorKind::InvalidData))
        }
    }

    pub fn try_get_buffer_view<'a>(&'a self, idx: u64) -> Result<&'a json::BufferView> {
        Ok(self.buffer_views
            .as_ref()
            .and_then(|buffer_views| buffer_views.get(idx as usize))
            .ok_or(Error::from(ErrorKind::InvalidData))?)
    }

    pub fn try_get_buffer<'a>(&'a self, idx: u64) -> Result<&'a json::Buffer> {
        Ok(self.buffers
            .as_ref()
            .and_then(|buffers| buffers.get(idx as usize))
            .ok_or(Error::from(ErrorKind::InvalidData))?)
    }

}

impl json::Scene {
    pub fn to_scene(&self, gltf: &json::Gltf) -> Result<Scene> {
        let mut scene = Scene { root: Node::default() };
        
        for &node_idx in self.nodes.iter().flatten() {
            let gltf_node = gltf.nodes
                .as_ref()
                .and_then(|nodes| nodes.get(node_idx as usize))
                .ok_or(Error::from(ErrorKind::InvalidData))?;

            scene.root.children.push(gltf_node.to_node(gltf)?);
        }
        
        Ok(scene)
    }

}

impl json::Node {
    pub fn to_node(&self, gltf: &json::Gltf) -> Result<Node> {
        let mut node = Node::default();
        
        node.transform = self.get_transform()?;
        node.meshes = self.get_meshes(gltf)?;
        
        for &child_idx in self.children.iter().flatten() {
            let gltf_child = gltf.nodes
                .as_ref()
                .and_then(|nodes| nodes.get(child_idx as usize))
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

    fn get_meshes(&self, gltf: &json::Gltf) -> Result<Vec<Mesh>> {
        if let Some(mesh_idx) = self.mesh {
            let gltf_mesh = gltf.meshes
                .as_ref()
                .and_then(|meshes| meshes.get(mesh_idx as usize))
                .ok_or(Error::from(ErrorKind::InvalidData))?;
            Ok(gltf_mesh.to_meshes(gltf)?)
        } else {
            Ok(Vec::new())
        }
    }

}

impl json::Mesh {
    pub fn to_meshes(&self, gltf: &json::Gltf) -> Result<Vec<Mesh>> {
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

impl json::Primitive {
    pub fn to_mesh(&self, gltf: &json::Gltf) -> Result<Mesh> {  
        
        let &positions_idx = self.attributes.get("POSITION").ok_or(Error::from(ErrorKind::InvalidData))?;
        // let normals_idx = self.attributes.get("NORMAL");
        // let tangents_idx = self.attributes.get("TANGENT");
        
        let positions_accessor = gltf.accessors
            .as_ref()
            .and_then(|accessors| accessors.get(positions_idx as usize))
            .ok_or(Error::from(ErrorKind::InvalidData))?;
        
        let vertices = vec![Vertex::default(); positions_accessor.count as usize];

        

        Ok( Mesh{ vertices: Vec::new(), indices: Vec::new() } )
    }
}

enum AccessorComponent {
    Byte(i8),
    UnsignedByte(u8),
    Short(i16),
    UnsignedShort(u16),
    UnsignedInt(u32),
    Float(f32),
}

enum AccessorElement {
    Scalar(AccessorComponent),
    Vec2(Vector2<AccessorComponent>),
    Vec3(Vector3<AccessorComponent>),
    Vec4(Vector4<AccessorComponent>),
    Mat2(Matrix2<AccessorComponent>),
    Mat3(Matrix3<AccessorComponent>),
    Mat4(Matrix4<AccessorComponent>),
}

enum AccessorIter<'a> {
    Dense {
        buffer: &'a json::Buffer,
        position: usize,
        stride: usize,
        element_size: usize,
    }
}

impl json::Accessor {

    
    // pub fn iter<'a>(&self, gltf: &'a json::Gltf) -> Result<AccessorIter<'a>> {
    //     Ok(AccessorIter::Dense {
    //         buffer: gltf
    //             .try_get_buffer(gltf
    //                 .try_get_buffer_view(self.buffer_view
    //                     .ok_or(Error::from(ErrorKind::InvalidData))?
    //                 )?.buffer
    //             )?,
    //         position: self.total_offset(gltf)?,
            
            
    //     })
    // }

    fn total_offset(&self, gltf: &json::Gltf) -> Result<u64> {
        Ok(self.byte_offset + gltf
            .try_get_buffer_view(self.buffer_view.ok_or(Error::from(ErrorKind::InvalidData))?)?
            .byte_offset
        )
    }

    fn stride(&self, gltf: &json::Gltf) -> Result<u64> {
        Ok(match self.buffer_view {
            Some(buffer_view_idx) => gltf.try_get_buffer_view(buffer_view_idx)?
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

