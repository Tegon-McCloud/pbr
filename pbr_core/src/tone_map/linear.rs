use rayon::prelude::ParallelIterator;

use crate::{texture::Texture, spectrum::Spectrum};

use super::ToneMap;


pub struct LinearToneMap {
    factor: f32,
}


impl LinearToneMap {
    pub fn new() -> LinearToneMap {
        LinearToneMap { factor: 1.0 }
    }

    pub fn new_scaling(factor: f32) -> LinearToneMap {
        LinearToneMap { factor }
    }
}

impl ToneMap for LinearToneMap {
    fn apply(&self, img: &mut Texture<Spectrum<f32>>) {
        img.par_pixels_mut()
            .for_each(|(_xy, px)| px.apply(|x| *x = (*x * self.factor).clamp(0.0, 1.0)));
    }
}