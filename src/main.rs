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

use std::{path::Path, f32::consts::PI};

use loader::Gltf;
use scene::Scene;
use camera::Camera;
use render_target::RenderTarget;
use accelerator::*;
use integrator::{BruteForce, Integrator};


use nalgebra::{Point3, Vector3};

fn main() {
    let mut render_target = RenderTarget::new(512, 512, &Vector3::new(0.0, 0.0, 0.0));

    let mut scene = Scene::from_file::<Gltf>(Path::new("resources/test.gltf")).unwrap();    

    Bvh::from_scene_node(scene.root);
    // scene.camera = Camera::perspective_look_at(
    //     &Point3::new(0.0, 2.0, 4.0), 
    //     &Point3::new(0.0, 1.0, 0.0), 
    //     &Vector3::new(0.0, 1.0, 0.0), 
    //     PI / 2.0,
    //     render_target.aspect_ratio(),
    // );
    
    // let integrator = BruteForce::<Trivial>::new(4, 512);
    // integrator.render(scene, &mut render_target);

    // //render_target.normalize();

    // render_target.save("test.png");
}
 