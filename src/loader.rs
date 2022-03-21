mod gltf;

pub use crate::loader::gltf::Gltf;

use crate::scene::SceneBuilder;

use std::path::Path;
use std::io::{Result, Read, Seek};

pub trait Loader {
    fn load_from_file(path: &Path) -> Result<SceneBuilder>;
    fn load_from_reader<R: Read + Seek>(rdr: &mut R) -> Result<SceneBuilder>;
}

impl SceneBuilder {
    pub fn from_file<L: Loader>(path: &Path) -> Result<Self> {
        L::load_from_file(path)
    }
}

