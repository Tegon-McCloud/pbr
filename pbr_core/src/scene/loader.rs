mod gltf;

pub use crate::scene::loader::gltf::Gltf;

use crate::scene::SceneBuilder;

use std::path::Path;
use std::io::{Result, Read, Seek};

pub trait Loader {
    fn load_from_file<P: AsRef<Path>>(path: P, builder: &mut SceneBuilder) -> Result<()>;
    fn load_from_reader<R: Read + Seek>(rdr: &mut R, builder: &mut SceneBuilder) -> Result<()>;
}
