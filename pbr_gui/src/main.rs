
use pbr_core::{
    texture::{RenderTarget, Texture},
    spectrum::Spectrum,
    scene::{SceneBuilder, loader::Gltf},
    light::{LightSource, SkySphere},
    camera::Camera,
    accelerator::Bvh,
    integrator::{Integrator, PathTracer},
    nalgebra::{Point3, Vector3}
};
use winit::{
    event_loop::{EventLoop, ControlFlow},
    dpi::LogicalSize,
    window::WindowBuilder,
    event::{Event, WindowEvent}
};
use pixels::{SurfaceTexture, Pixels};
use rand::Rng;


fn open_window() {

    let event_loop = EventLoop::new();
    let size = LogicalSize::new(400.0, 400.0);
    let window = WindowBuilder::new()
        .with_title("PBR Output")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            let mut rng = rand::thread_rng();
            let color: u8 = rng.gen();

            for (_i, pixel) in pixels.get_frame().chunks_exact_mut(4).enumerate() {
                pixel.copy_from_slice(&[color, color, color, 255]);
            }

            pixels.render()
                .expect("failed to render");
            
        } else if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            }

        } else if let Event::MainEventsCleared = event {
            window.request_redraw();
        }
    });

}


fn main() {


    
    let begin_time = std::time::Instant::now();

    let mut render_target = RenderTarget::new(1024, 512, &Spectrum::black());
    let env_map = Texture::<Spectrum<f32>>::from_hdr_file("resources/abandoned_greenhouse_4k.hdr");

    let scene = SceneBuilder::new()
        .add_file::<Gltf, _>("resources/bunny.gltf").unwrap()
        .add_light(LightSource::SkySphere(SkySphere::new(env_map)))
        .camera(Camera::perspective_look_at(
            &Point3::new(0.0, 3.0, 5.0), 
            &Point3::new(0.0, 2.0, 0.0), 
            &Vector3::new(0.0, 1.0, 0.0), 
            std::f32::consts::PI / 2.0,
            render_target.aspect_ratio(),
        ));

    println!("Load time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());
    let begin_time = std::time::Instant::now();

    let scene = scene.build::<Bvh>();
    
    println!("Build time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());
    let begin_time = std::time::Instant::now();
    
    let integrator = PathTracer::new(4, 2048);
    integrator.render(&scene, &mut render_target);



    
    println!("Render time: {}s", (std::time::Instant::now() - begin_time).as_secs_f32());
    
    render_target.save("test.png");
}
 