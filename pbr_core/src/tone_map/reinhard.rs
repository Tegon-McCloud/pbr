use rayon::prelude::ParallelIterator;

use crate::{texture::Texture, spectrum::Spectrum};

use super::ToneMap;




pub struct ReinhardToneMap {
    white_point: Option<f32>,
}

impl ReinhardToneMap {
    pub fn new() -> ReinhardToneMap {
        ReinhardToneMap { white_point: None }
    }

    pub fn with_whitepoint(white_point: f32) -> ReinhardToneMap {
        ReinhardToneMap { white_point: Some(white_point) }
    }



}
fn compute_white_point(img: &Texture<Spectrum<f32>>) -> f32 {
    img.par_pixels()
        .map(|(_xy, px)| px.r.max(px.g).max(px.b))
        .max_by(|x, y| x.total_cmp(&y))
        .unwrap()
}

impl ToneMap for ReinhardToneMap {
    fn apply(&self, img: &mut Texture<Spectrum<f32>>) {
        
        let white_point = self.white_point.unwrap_or_else(|| compute_white_point(img));
        let white_point2 = white_point * white_point;

        img.par_pixels_mut()
            .for_each(|(_xy, px)| px.apply(|x| *x = *x * (1.0 + *x / white_point2) / (1.0 + *x)));
    }
}