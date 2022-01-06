mod gltf;
use crate::loader::gltf::GltfLoader;
use crate::scene::Scene;

use std::path::Path;
use std::io::{Error, ErrorKind, Result};

trait Loader {
    fn load(path: &Path) -> Result<Scene>;
}

impl Scene {
    pub fn from_file(path: &Path) -> Result<Self> {

        if let Some(extension) = path.extension() {

            if extension.eq_ignore_ascii_case("gltf") || extension.eq_ignore_ascii_case("glb") {
                GltfLoader::load(path)
            } else {
                Err(Error::new(ErrorKind::InvalidData, format!("Unknown format: {:?}.", extension)))
            }
        } else {
            Err(Error::new(ErrorKind::InvalidData, format!("Format could not be determined for file: {:?}.", path.as_os_str())))
        }
    }
}

