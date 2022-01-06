use nalgebra::{Vector3, Point3};

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
}

pub fn triangle_intersect(p1: &Point3<f32>, p2: &Point3<f32>, p3: &Point3<f32>, ray: &Ray) -> Option<(f32, Point3<f32>, Vector3<f32>)> {
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
        return Some((t, p, Vector3::zeros()));
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
        let (t, p, _bary) = result.unwrap();
        assert!(approx_eq!(f32, t, 1.0, ulps = 2));
        assert!(approx_eq!(f32, p.x, 0.25, ulps = 2));
        assert!(approx_eq!(f32, p.y, 0.25, ulps = 2));
        assert!(approx_eq!(f32, p.z, 0.0, ulps = 2));
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

}
