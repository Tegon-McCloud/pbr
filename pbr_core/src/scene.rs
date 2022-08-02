pub mod loader;

use std::{ops::Mul, path::Path, io::Result};

use nalgebra::{Point3, Vector3, Affine3, Point2};
use rand::Rng;

use crate::{camera::Camera, light::{LightSource, Emitter}, accelerator::Accelerator, material::Material, geometry::{SurfacePoint, Ray}};
use loader::Loader;

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub tex_coords: Point2<f32>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Box<dyn Material>,
}

pub struct Node {
    children: Vec<Node>,
    transform: Affine3<f32>,
    meshes: Vec<Mesh>,
}

#[derive(Default)]
pub struct SceneBuilder {
    root: Node,
    camera: Camera,
    light_sources: Vec<LightSource>,
}

pub struct Scene<A> {
    accelerator: A,
    camera: Camera,
    light_sources: Vec<LightSource>,
    materials: Vec<Box<dyn Material>>,
}

impl SceneBuilder {
    pub fn new() -> SceneBuilder {
        SceneBuilder { root: Node::default(), camera: Camera::default(), light_sources: Vec::default() }
    }

    pub fn build<A: Accelerator>(self) -> Scene<A> {
        let meshes = self.root.flatten();
        let (geometry, materials) = meshes.into_iter()
            .map(|mesh| ((mesh.vertices, mesh.indices), mesh.material))
            .unzip();
        let accelerator = A::build(geometry);
        let camera = self.camera;
        let light_sources = self.light_sources;

        Scene::<A> {
            accelerator,
            camera,
            light_sources,
            materials,
        }
 
    }

    pub fn add_file<L: Loader, P: AsRef<Path>>(mut self, path: P) -> Result<SceneBuilder> {
        L::load_from_file(path, &mut self)?;
        Ok(self)
    }

    pub fn add_light(mut self, light: LightSource) -> SceneBuilder {
        self.light_sources.push(light);
        self
    }

    pub fn camera(mut self, camera: Camera) -> SceneBuilder {
        self.camera = camera;
        self
    }
}

impl Node {
    pub fn flatten(self) -> Vec<Mesh> {
        let mut meshes = Vec::new();
        self.flatten_recursive(&Affine3::identity(), &mut meshes);
        meshes
    }

    fn flatten_recursive(self, parent_transform: &Affine3<f32>, meshes: &mut Vec<Mesh>) {
        let transform = parent_transform * self.transform;
        meshes.extend(self.meshes.into_iter().map(|mesh| transform * mesh));

        for child in self.children.into_iter() {
            child.flatten_recursive(&transform, meshes)
        }
    }

}

impl Default for Node {
    fn default() -> Self {
        Node {
            children: Vec::new(),
            transform: Affine3::identity(),
            meshes: Vec::new(),
        }
    }
}

impl<A> Scene<A> where
    A: Accelerator 
{
    pub fn intersect_dist(&self, ray: &Ray) -> Option<f32> {
        self.accelerator.intersect(ray).map(|info| info.t)
    }

    pub fn intersect<'s>(&'s self, ray: &Ray) -> Option<SurfacePoint<'s>> {
        self.accelerator.intersect(ray).map(|info|  {
            let material = self.materials[info.mesh as usize].as_ref();
            SurfacePoint::new(&info.barycentrics, &info.vertices, material)
        })
    } 
    
    pub fn pick_light<'a>(&'a self) -> (&'a LightSource, f32) {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.light_sources.len());
        (&self.light_sources[idx], 1.0 / self.light_sources.len() as f32)
    }

    pub fn background_lights<'a>(&'a self) -> impl Iterator<Item = &'a LightSource> {
        self.light_sources.iter().filter(|&l| l.is_background())
    } 

    pub fn get_camera<'a>(&'a self) -> &'a Camera {
        &self.camera
    } 
}

impl Mul<Mesh> for Affine3<f32> {
    type Output = Mesh;
    fn mul(self, mut rhs: Mesh) -> Self::Output {
        for vertex in &mut rhs.vertices {
            vertex.position = self * vertex.position;
            vertex.normal = self * vertex.normal;
            vertex.tangent = self * vertex.tangent;
        }
        rhs
    }
}
