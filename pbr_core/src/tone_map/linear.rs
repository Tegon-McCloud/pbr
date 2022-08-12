use rayon::prelude::ParallelIterator;

use crate::{texture::Texture, spectrum::Spectrum};

use super::ToneMap;


pub struct LinearToneMap {}


impl LinearToneMap {
    pub fn new() -> LinearToneMap {
        LinearToneMap {}
    }
}

impl ToneMap for LinearToneMap {
    fn apply(&self, img: &mut Texture<Spectrum<f32>>) {
        img.par_pixels_mut()
            .for_each(|(_xy, px)| px.apply(|x| *x = x.clamp(0.0, 1.0)));
    }
}