#![feature(total_cmp)]

extern crate nalgebra;
extern crate gltf;

mod geometry;
mod scene;
mod camera;
mod render_target;
mod loader;
mod material;
mod accelerator;
mod integrator;

use std::path::Path;

use loader::Gltf;
use scene::Scene;
use camera::Camera;
use render_target::RenderTarget;
use accelerator::Trivial;
use integrator::{Recursive, Integrator};


use nalgebra::{Point3, Vector3, Vector4};

fn main() {    
    let mut scene = Scene::from_file::<Gltf>(Path::new("resources/test.gltf")).unwrap();
    let mut render_target = RenderTarget::new(512, 512, &Vector4::new(0.0, 0.0, 0.0, 1.0));

    scene.camera = Camera::perspective_look_at(
        &Point3::new(0.0, 3.0, 5.0), 
        &Point3::new(0.0, 1.0, 0.0), 
        &Vector3::new(0.0, 1.0, 0.0), 
        1.0, 
        1.0
    );
    
    let integrator = Recursive::<Trivial>::new(4, 128);
    integrator.render(scene, &mut render_target);
}
 