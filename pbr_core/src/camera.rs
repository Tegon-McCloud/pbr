use nalgebra::{Point3, Vector3, Point2, Vector2};
use rand::Rng;

use crate::geometry::Ray;

pub enum Camera {
    Perspective(PerspectiveCamera),
}

impl Camera {

    pub fn perspective_look_at(pos: &Point3<f32>, focus: &Point3<f32>, up_dir: &Vector3<f32>, vfov: f32, aspect: f32) -> Camera {
        Self::perspective_look_to(pos, &(focus - pos), up_dir, vfov, aspect)
    }

    pub fn perspective_look_to(pos: &Point3<f32>, forward: &Vector3<f32>, up_dir: &Vector3<f32>, vfov: f32, aspect: f32) -> Camera {

        let forward = forward.normalize();
        let right = forward.cross(up_dir).normalize();
        let up = right.cross(&forward);

        let tan_vfov = (vfov/2.0).tan();
        let tan_hfov = tan_vfov * aspect;

        Camera::Perspective(PerspectiveCamera {
            position: *pos,
            forward: forward,
            horizontal: right * tan_hfov,
            vertical: up * tan_vfov
        })
    }

    pub fn get_ray(&self, xy: Point2<u32>, img_size: (u32, u32)) -> Ray {
        let mut rng = rand::thread_rng();
        
        let uv = Point2::new(
            (xy.x as f32 + rng.gen::<f32>()) / img_size.0 as f32,
            1.0 - (xy.y as f32 + rng.gen::<f32>()) / img_size.1 as f32,
        );
        
        match self {
            Self::Perspective(camera) => camera.get_ray(&uv),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::Perspective(PerspectiveCamera {
            position: Point3::origin(),
            forward: Vector3::new(0.0, 0.0, -1.0),
            horizontal: Vector3::new(1.0, 0.0, 0.0),
            vertical: Vector3::new(0.0, 1.0,0.0),
        })
    }
}

pub struct PerspectiveCamera  {
    position: Point3<f32>,
    forward: Vector3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
}

impl PerspectiveCamera {
    fn get_ray(&self, uv: &Point2<f32>) -> Ray {
        let uv = 2.0 * uv - Vector2::new(1.0, 1.0);
        let origin = self.position;
        let direction = (self.forward + uv.x * self.horizontal + uv.y * self.vertical).normalize();
        Ray { origin, direction }
    }
}
