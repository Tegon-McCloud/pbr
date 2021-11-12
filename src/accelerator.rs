use crate::scene::Scene;

pub trait Accelerator {
    fn from_scene(scene: Scene);
}
