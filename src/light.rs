use std::f32::consts::PI;

use enum_dispatch::enum_dispatch;
use nalgebra::{Vector3, Point3, Point2, Matrix3};
use rand::{thread_rng, Rng};

use crate::{geometry::{SurfacePoint, Ray, uniform_hemisphere_map}, accelerator::Accelerator, texture::Texture, scene::Scene, spectrum::Spectrum};

pub enum VisibilityTest {
    PointToPoint {
        p1: Point3<f32>,
        p2: Point3<f32>,
    },
    PointInDirection {
        p: Point3<f32>,
        d: Vector3<f32>,  
    }
}

impl VisibilityTest {
    pub fn eval<A: Accelerator>(self, scene: &Scene<A>) -> bool {
        match self {
            Self::PointToPoint{ p1, p2 } => {
                let ray = Ray {
                    origin: p1,
                    direction: (p2 - p1).normalize(),
                };
        
                let dist = (p2 - p1).norm();
        
                match scene.accelerator.intersect(&ray) {
                    Some(info) => info.t <= dist,
                    None => true,
                }
            },

            Self::PointInDirection{ p, d } => {
                let ray = Ray {
                    origin: p,
                    direction: d.normalize(),
                };

                scene.accelerator.intersect(&ray).is_none()
            }
        }

    }
}

pub struct RadianceSample {
    pub radiance: Spectrum<f32>,
    pub direction: Vector3<f32>,
    pub pdf: f32,
    pub visibility_test: VisibilityTest,
}

#[enum_dispatch]
pub trait Emitter {
    fn emission(&self, dir: &Vector3<f32>) -> Spectrum<f32>;
    fn sample(&self, p: &SurfacePoint) -> RadianceSample;
    fn is_delta(&self) -> bool;
    fn is_background(&self) -> bool;
}

#[enum_dispatch(Emitter)]
pub enum LightSource {
    Test(TestLight),
    Directional(DirectionalLight),
    SkySphere(SkySphere),
}

pub struct DirectionalLight {
    pub neg_direction: Vector3<f32>,
    pub irradiance: Spectrum<f32>,
}

impl Emitter for DirectionalLight {
    fn emission(&self, _dir: &Vector3<f32>) -> Spectrum<f32> {
        Spectrum::black()
    }

    fn sample(&self, p: &SurfacePoint) -> RadianceSample {
        RadianceSample {
            radiance: self.irradiance,
            direction: self.neg_direction,
            pdf: 1.0,
            visibility_test: VisibilityTest::PointInDirection { p: p.position + 0.0001 * p.normal, d: self.neg_direction },
        }
    }

    fn is_delta(&self) ->bool {
        true
    }

    fn is_background(&self) -> bool {
        true
    }
}

pub struct SkySphere {
    texture: Texture<Spectrum<f32>>,
}

impl SkySphere {
    pub fn new(texture: Texture<Spectrum<f32>>) -> Self {
        Self { texture }
    }
}

impl Emitter for SkySphere {
    fn emission(&self, dir: &Vector3<f32>) -> Spectrum<f32> {
        let uv = Point2::new(
            1.0 / (2.0 * PI) * f32::atan2(dir[2], dir[0]),
            1.0 / PI * f32::acos(dir[1]),
        );
        
        self.texture.sample(&uv)
    }

    fn sample(&self, p: &SurfacePoint) -> RadianceSample {
        let t2w = p.tangent_to_world();
        let mut rng = thread_rng();
        let u = Point2::new(rng.gen(), rng.gen());
        let direction = t2w * uniform_hemisphere_map(&u);
        let pdf = 1.0 / (2.0 * PI);
        
        let radiance = self.emission(&direction);
        
        let visibility_test = VisibilityTest::PointInDirection {
            p: p.position + 0.0001 * p.normal,
            d: direction
        };

        RadianceSample {
            radiance,
            direction,
            pdf,
            visibility_test
        }
    }

    fn is_delta(&self) ->bool {
        false
    }

    fn is_background(&self) ->bool {
        true
    }
}

pub struct TestLight {}

impl Emitter for TestLight {
    fn emission(&self, dir: &Vector3<f32>) -> Spectrum<f32> {
        let light_dir = Vector3::new(1.0, 1.0, 1.0).normalize();
        let light_col = Spectrum::new(1.0, 1.0, 1.0);
        light_col * light_dir.dot(dir).max(0.0)
    }

    fn sample(&self, p: &SurfacePoint) -> RadianceSample {
        let t = p.tangent;
        let n = p.normal;
        let b = n.cross(&t);
        
        let t2w = Matrix3::from_columns(&[t, b, n]);
        
        let mut rng = thread_rng();
        let u = Point2::new(rng.gen(), rng.gen());

        let sample_dir = t2w * uniform_hemisphere_map(&u);

        let light_dir = Vector3::new(1.0, 1.0, 1.0).normalize();
        let light_col = Spectrum::new(1.0, 1.0, 1.0);
        let irradiance = light_col * light_dir.dot(&sample_dir).max(0.0);
        let vis_test = VisibilityTest::PointInDirection { p: p.position + 0.0001 * p.normal, d: sample_dir, };
        
        RadianceSample {
            radiance: irradiance,
            direction: sample_dir,
            pdf: 1.0 / (2.0 * PI),
            visibility_test: vis_test,
        }
    }

    fn is_delta(&self) ->bool {
        false
    }

    fn is_background(&self) ->bool {
        true
    }
}