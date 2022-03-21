#![feature(total_cmp)]
#![allow(dead_code)]

extern crate nalgebra;
extern crate gltf;

mod geometry;
mod scene;
mod camera;
mod loader;
mod material;
mod accelerator;
mod integrator;
mod light;
mod texture;

use std::{path::Path, f32::consts::PI};


use loader::Gltf;
use scene::SceneBuilder;
use camera::Camera;
use accelerator::*;
use integrator::*;
use light::*;
use texture::*;

use nalgebra::{Point3, Vector3};
use texture::Texture;

fn main() {

    let mut scene = SceneBuilder::from_file::<Gltf>(Path::new("resources/test.gltf")).unwrap();

    let mut render_target = RenderTarget::new(1024, 512, &Vector3::new(0.0, 0.0, 0.0));

    let env_map = Texture::<Vector3<f32>>::from_hdr_file("resources/abandoned_greenhouse_4k.hdr");
    let sky_sphere = SkySphere::new(env_map);
    scene.light_sources.push(LightSource::SkySphere(sky_sphere));

    scene.camera = Camera::perspective_look_at(
        &Point3::new(0.0, 2.0, 4.0), 
        &Point3::new(0.0, 1.0, 0.0), 
        &Vector3::new(0.0, 1.0, 0.0), 
        PI / 2.0,
        render_target.aspect_ratio(),
    );

    // scene.light_sources.push(LightSource::Directional(DirectionalLight {
    //     neg_direction: Vector3::new(1.0, 1.0, 1.0).normalize(),
    //     irradiance: Vector3::new(1.0, 1.0, 1.0),
    // }));
    
    // scene.light_sources.push(LightSource::Test(TestLight {}));

    let scene = scene.build::<Bvh>();

    let integrator = PathTracer::new(4, 1024);
    integrator.render(&scene, &mut render_target);

    //render_target.normalize();

    render_target.save("test.png");
}
 