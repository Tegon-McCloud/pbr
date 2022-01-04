
use std::path::Path;

use scene::Scene;


extern crate nalgebra;
extern crate serde;
extern crate serde_json;

mod scene;
mod loader;
mod material;
mod accelerator;
mod integrator;

fn main() {


    let scene = Scene::from_file(Path::new("resources/test.gltf")).unwrap();
    println!("{}", scene.root.children.len());

}
 