#![allow(unused_imports)]

use std::ops::Mul;

use pbr_core::{
    texture::Texture,
    spectrum::Spectrum,
    scene::{SceneBuilder, loader::Gltf},
    light::{LightSource, SkySphere},
    camera::Camera,
    accelerator::Bvh,
    integrator::{Integrator, PathTracer, BruteForcer},
    nalgebra::{Point3, Vector3},
    tone_map::{ToneMap, ReinhardToneMap, self, LinearToneMap},
};
use winit::{
    event_loop::{EventLoop, ControlFlow},
    dpi::LogicalSize,
    window::WindowBuilder,
    event::{Event, WindowEvent}
};
use pixels::{SurfaceTexture, Pixels};


fn main() {
    
    let begin_time = std::time::Instant::now();

    let render_size = (1280, 720);
    let env_map = Texture::<Spectrum<f32>>::from_hdr_file("resources/abandoned_greenhouse_4k.hdr");

    let scene = SceneBuilder::new()
        .add_file::<Gltf, _>("resources/bunny.gltf").unwrap()
        .add_light(LightSource::SkySphere(SkySphere::new(env_map)))
        .camera(Camera::perspective_look_at(
            &Point3::new(0.0, 3.0, 5.0), 
            &Point3::new(0.0, 2.0, 0.0), 
            &Vector3::new(0.0, 1.0, 0.0), 
            std::f32::consts::PI / 2.0,
            render_size.0 as f32 / render_size.1 as f32,
        ));

    println!("Load time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());
    let begin_time = std::time::Instant::now();
    
    let scene = scene.build::<Bvh>();
    
    println!("Build time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());
    
    let integrator = BruteForcer::new(4, 512);
    
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(render_size.0, render_size.1);
    let window = WindowBuilder::new()
        .with_title("PBR Output")
        .with_inner_size(size)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    
    let surface_texture = SurfaceTexture::new(render_size.0, render_size.1, &window);
    let mut pixels = Pixels::new(render_size.0, render_size.1, surface_texture).unwrap();

    let (tx, rx) = std::sync::mpsc::sync_channel(0);
    
    let _render_thread = std::thread::spawn(move || {
        let begin_time = std::time::Instant::now();
        
        let render_img = integrator.render(&scene, render_size, |img| tx.send(img.clone()).unwrap_or(()));

        println!("Render time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());

        let mut render_img_linear = render_img.clone();
        LinearToneMap::new().apply(&mut render_img_linear);
        render_img_linear.save("test_linear.png");

        let mut render_img_reinhard = render_img.clone();
        ReinhardToneMap::new().apply(&mut render_img_reinhard);
        render_img_reinhard.save("test_reinhard.png");
    });

    event_loop.run(move |event, _, control_flow| {

        if let Event::MainEventsCleared = event {
            window.request_redraw();
        }

        if let Event::RedrawRequested(_) = event {
            match rx.recv() {
                Ok(img) => {

                    for (wnd_px, (_, px)) in pixels.get_frame().chunks_exact_mut(4).zip(img.pixels()) {
                        let mut px = *px;
                        px.apply(|x| *x = x.powf(1.0/2.2).mul(255.0).clamp(0.0, 255.0));
                        wnd_px.copy_from_slice(&[px.r as u8, px.g as u8, px.b as u8, 255u8]);
                    }
        
                    pixels.render().unwrap();
                },
                Err(_) => {},
            }
        }
        
        if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => (),
            }
        } 
    });
}
 