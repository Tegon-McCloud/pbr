use nalgebra::{Vector3, Point3};



struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
}


struct Bounds {
    min: Point3<f32>,
    max: Point3<f32>,
}

