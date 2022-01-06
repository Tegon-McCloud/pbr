use nalgebra::{Point3, Vector3};

pub enum Camera {
    Perspective(PerspectiveCamera),
}

impl Camera {
    pub fn perspective_look_at(pos: &Point3<f32>, focus: &Point3<f32>, up_dir: &Vector3<f32>, vfov: f32, aspect: f32) -> Camera {
        Self::perspective_look_to(pos, &(pos - focus), up_dir, vfov, aspect)
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
