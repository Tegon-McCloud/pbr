use std::f32::consts::PI;

use float_cmp::approx_eq;
use nalgebra::{Vector3, Point3, Point2};

use crate::scene::Vertex;

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Bounds {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

pub struct SurfacePoint {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
}

impl SurfacePoint {
    pub fn interpolate(vertices: &[&Vertex; 3], barycentrics: &Vector3<f32>) -> SurfacePoint {
        let b = barycentrics;
        let [v1, v2, v3] = vertices;
        

        let position = Point3::from(v1.position.coords * b.x + v2.position.coords * b.y + v3.position.coords * b.z);
        let normal = (v1.normal * b.x + v2.normal * b.y + v3.normal * b.z).normalize();
        let tangent =  v1.tangent * b.x + v2.tangent * b.y + v3.tangent * b.z;
        // gram-schmidt
        let tangent = (tangent - normal * tangent.dot(&normal)).normalize();

        SurfacePoint { position, normal, tangent }
    }

}

pub fn uniform_hemisphere_map(p: &Point2<f32>) -> Vector3<f32> {
    let costheta = 1.0 - 2.0 * p.x;
    let phi = 2.0 * PI * p.y;

    let sintheta = costheta.acos().sin();

    Vector3::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta)
}

pub fn triangle_intersect(p1: &Point3<f32>, p2: &Point3<f32>, p3: &Point3<f32>, ray: &Ray) -> Option<(f32, Vector3<f32>)> {
    let e1 = p2 - p1;
    let e2 = p3 - p2;
    let e3 = p1 - p3;

    let n = e1.cross(&e2);
    let ddotn = ray.direction.dot(&n);

    if ddotn >= 0.0 {
        return None;
    }

    let odotn = (p1 - ray.origin).dot(&n);

    if odotn >= 0.0 {
        return None;
    }

    let t = odotn / ddotn;
    let p = ray.origin + ray.direction * t;
    
    let n1 = e1.cross(&(p - p1));
    let n2 = e2.cross(&(p - p2));
    let n3 = e3.cross(&(p - p3));

    if n.dot(&n1) >= 0.0 && n.dot(&n2) >= 0.0 && n.dot(&n3) > 0.0 {
        let b = Vector3::new(n2.norm(), n3.norm(), n1.norm());
        let area = b.x + b.y + b.z;

        debug_assert!(approx_eq!(f32, n.norm(), area, ulps = 2));

        return Some((t, b/area));
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn triangle_intersect_hits() {
        let result = triangle_intersect(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
            &Ray {
                origin: Point3::new(0.25, 0.25, 1.0),
                direction: Vector3::new(0.0, 0.0, -1.0),
            }
        );

        assert!(result.is_some());
        let (t, b) = result.unwrap();
        
        assert!(approx_eq!(f32, t, 1.0, ulps = 2));

        assert!(approx_eq!(f32, b.x, 0.5, ulps = 2));
        assert!(approx_eq!(f32, b.y, 0.25, ulps = 2));
        assert!(approx_eq!(f32, b.z, 0.25, ulps = 2));
    }
    
    
    #[test]
    fn triangle_intersect_miss_dir_away() {
        let result = triangle_intersect(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
            &Ray {
                origin: Point3::new(0.25, 0.25, 1.0),
                direction: Vector3::new(0.0, 0.0, 1.0),
            }
        );

        assert!(result.is_none());
    }

    
    #[test]
    fn triangle_intersect_miss_origin_behind() {
        let result = triangle_intersect(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
            &Ray {
                origin: Point3::new(0.25, 0.25, -1.0),
                direction: Vector3::new(0.0, 0.0, -1.0),
            }
        );

        assert!(result.is_none());
    }

    #[test]
    fn triangle_intersect_miss_outside() {
        let result = triangle_intersect(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
            &Ray {
                origin: Point3::new(1.0, 1.0, 1.0),
                direction: Vector3::new(0.0, 0.0, -1.0),
            }
        );

        assert!(result.is_none());
    }

    #[test]
    fn surface_point_interpolate_flat() {

        let positions = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let normals = [
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];

        let tangents = [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        ];

        let vertices = [
            Vertex {
                position: positions[0],
                normal: normals[0],
                tangent: tangents[0]
            },
            Vertex {
                position: positions[1],
                normal: normals[1],
                tangent: tangents[1]
            },
            Vertex {
                position: positions[2],
                normal: normals[2],
                tangent: tangents[2]
            },
        ];

        let result = SurfacePoint::interpolate(
            &[&vertices[0], &vertices[1], &vertices[2]],
            &Vector3::new(0.5, 0.25, 0.25),
        );
        
        assert!(approx_eq!(f32, result.position.x, 0.25, ulps = 2));
        assert!(approx_eq!(f32, result.position.y, 0.25, ulps = 2));
        assert!(approx_eq!(f32, result.position.z, 0.0, ulps = 2));
        
        assert!(approx_eq!(f32, result.normal.x, 0.0, ulps = 2));
        assert!(approx_eq!(f32, result.normal.y, 0.0, ulps = 2));
        assert!(approx_eq!(f32, result.normal.z, 1.0, ulps = 2));
        
        assert!(approx_eq!(f32, result.tangent.x, 1.0, ulps = 2));
        assert!(approx_eq!(f32, result.tangent.y, 0.0, ulps = 2));
        assert!(approx_eq!(f32, result.tangent.z, 0.0, ulps = 2));
    }

}
