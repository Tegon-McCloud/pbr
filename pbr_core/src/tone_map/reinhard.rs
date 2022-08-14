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

fn luminance(col: &Spectrum<f32>) -> f32 {
    col.r * 0.2126 + col.g * 0.7152 + col.b * 0.0722
}

fn compute_white_point(img: &Texture<Spectrum<f32>>) -> f32 {
    img.par_pixels()
        .map(|(_xy, px)| luminance(px))
        .max_by(|a, b| a.total_cmp(&b))
        .unwrap()
}

impl ToneMap for ReinhardToneMap {
    fn apply(&self, img: &mut Texture<Spectrum<f32>>) {
        
        let l_wp = self.white_point.unwrap_or_else(|| compute_white_point(img));
        let l_wp2 = l_wp * l_wp;

        img.par_pixels_mut()
            .for_each(|(_xy, px)| {
                let l_old = luminance(px);
                let l_new = l_old * (1.0 + l_old / l_wp2) / (1.0 + l_old);
                *px = *px * l_new / l_old
            });
    }
}