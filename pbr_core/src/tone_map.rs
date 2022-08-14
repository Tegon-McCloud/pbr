use crate::{texture::Texture, spectrum::Spectrum};


mod linear;
mod reinhard;


pub use reinhard::ReinhardToneMap;
pub use linear::LinearToneMap;

pub trait ToneMap {
    fn apply(&self, img: &mut Texture<Spectrum<f32>>);

}
