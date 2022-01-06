mod gltf;

pub use crate::loader::gltf::Gltf;

use crate::scene::Scene;

use std::path::Path;
use std::io::{Result, Read, Seek};

pub trait Loader {
    fn load_from_file(path: &Path) -> Result<Scene>;
    fn load_from_reader<R: Read + Seek>(rdr: &mut R) -> Result<Scene>;
}

impl Scene {
    pub fn from_file<L: Loader>(path: &Path) -> Result<Self> {
        L::load_from_file(path)
    }
}

