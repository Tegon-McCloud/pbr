
extern crate nalgebra;
extern crate gltf;

mod scene;
mod loader;
mod material;
mod accelerator;
mod integrator;


use std::path::Path;

use scene::Scene;
use accelerator::Trivial;
use integrator::{Recursive, Integrator};

fn main() {    
    let scene = Scene::from_file(Path::new("resources/test.gltf")).unwrap();
    let integrator = Recursive::<Trivial>::new(4);
    integrator.render(scene);
    
}
 