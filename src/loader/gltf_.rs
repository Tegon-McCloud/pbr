
use super::Loader;

use crate::scene::{Scene, Node, Mesh, Vertex};

use std::default;
use std::fs::File;
use std::path::Path;
use std::io::{Seek, SeekFrom, Error, ErrorKind, Result, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use nalgebra::Affine3;
use serde_json::{Value, Map};

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

        let magic = file.read_u32::<LittleEndian>()?;
        assert_eq!(magic, 0x46546c67);
        let version = file.read_u32::<LittleEndian>()?;
        assert_eq!(version, 2);
        let length = file.read_u32::<LittleEndian>()?;
        let chunks = Self::read_glb_chunks(&mut file, length as u64)?;
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_type, 0x4e4f534a);

        file.seek(SeekFrom::Start(chunks[0].start_pos as u64))?;
        let mut take = file.take(chunks[0].length);
        let gltf: Value = serde_json::from_reader(&mut take)?;
        let mut file = take.into_inner();

        let mut buffers = Vec::<GltfBuffer>::new();

        for (i, gltf_buf) in gltf["buffers"]
            .as_array()
            .ok_or(Error::from(ErrorKind::InvalidData))?
            .iter()
            .enumerate() {

            if i == 0 && chunks.len() >= 2 {
                buffers.push(Self::read_buffer_maybe_from_chunk(gltf_buf, &chunks[1], &mut file)?);
            } else {
                buffers.push(Self::read_buffer(gltf_buf)?);
            }
        }

        Self::scene_from_gltf_and_buffers(gltf, buffers)
    }

    fn load_gltf(path: &Path) -> Result<Scene> {
        let mut file = File::open(path)?;
        let gltf: Value = serde_json::from_reader(&mut file)?;

        let mut buffers = Vec::<GltfBuffer>::new();

        gltf.get("buffers").and_then(|gltf_buffers| {
            for gltf_buf in gltf_buffers
                .as_array()
                .ok_or(Error::from(ErrorKind::InvalidData))?
                .iter() {
                
            }
        });

        let gltf_buffers = gltf_buffers.unwrap();

        let buffers = gltf_buffers
            .as_array()
            .ok_or(Error::from(ErrorKind::InvalidData))?
            .iter()
            .map(Self::read_buffer)
            .collect::<Result<Vec<GltfBuffer>>>()?;
        
        Self::scene_from_gltf_and_buffers(gltf, buffers)
    }

    fn read_glb_chunks<R: ReadBytesExt + Seek>(rdr:&mut R, expected_length: u64) -> Result<Vec<GlbChunk>> {

        let mut length_counter = 12u64;
        let mut chunks = Vec::<GlbChunk>::new();

        loop {
            let chunk_length = rdr.read_u32::<LittleEndian>()? as u64;
            let chunk_type = rdr.read_u32::<LittleEndian>()?;
            length_counter += 8;

            chunks.push(GlbChunk {
                chunk_type: chunk_type,
                length: chunk_length,
                start_pos: length_counter,
            });

            length_counter += chunk_length;
            rdr.seek(SeekFrom::Current(chunk_length as i64))?;

            if length_counter == expected_length {
                break;
            }
        }

        Ok(chunks)
    }

    fn read_buffer_maybe_from_chunk<R: Read + Seek>(gltf_buf: &Value, data_chunk: &GlbChunk, chunk_rdr: &mut R) -> Result<GltfBuffer> {

        if !gltf_buf.as_object().ok_or(Error::from(ErrorKind::InvalidData))?.contains_key("uri") {
            assert_eq!(data_chunk.chunk_type, 0x004e4942);
            let buffer_length = gltf_buf["byteLength"]
                .as_u64()
                .ok_or(Error::from(ErrorKind::InvalidData))?;

            let mut buf = GltfBuffer { data: vec![0u8; buffer_length as usize] };
            chunk_rdr.seek(SeekFrom::Start(data_chunk.start_pos))?;
            chunk_rdr.read_exact(&mut buf.data)?;

            Ok(buf)
        } else {
            Self::read_buffer(gltf_buf)
        }
    }
 
    fn read_buffer(gltf_buf: &Value) -> Result<GltfBuffer> {
        unimplemented!();
    }

    fn scene_from_gltf_and_buffers(gltf: Value, buffers: Vec<GltfBuffer>) -> Result<Scene> {

        let gltf_scenes = gltf.get("scenes").ok_or(std::);
        


        Ok(Scene{ root: root })
    }

}

struct GlbChunk {
    pub chunk_type: u32,
    pub length: u64,
    pub start_pos: u64,
}

struct GltfBuffer {
    pub data: Vec<u8>,
}
