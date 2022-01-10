use std::path::Path;
use std::io::Cursor;

use nalgebra::{Vector4, Point2, clamp};

use rayon::prelude::*;
use image::{io::Reader as ImageReader, RgbaImage};

pub struct RenderTarget {
    dimensions: (u32, u32),
    buffer: Vec<Vector4<f32>>,
}

impl RenderTarget {

    pub fn new(width: u32, height: u32, fill_col: &Vector4<f32>) -> RenderTarget {
        let dimensions = (width, height);
        let buffer = vec![*fill_col; width as usize * height as usize];

        RenderTarget { dimensions, buffer, }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.dimensions.0 as f32 / self.dimensions.1 as f32
    }
    
    pub fn pixels_par_mut<'a>(&'a mut self) -> impl ParallelIterator<Item=(Point2<f32>, &mut Vector4<f32>)> + 'a {
        self.buffer.par_iter_mut()
            .enumerate()
            .map(|(pos, px)| {
                let x = pos as u32 % self.dimensions.0;
                let y = pos as u32 / self.dimensions.0;
                let uv = Point2::new(
                    (x as f32 + 0.5)/self.dimensions.0 as f32,
                    1.0 - (y as f32 + 0.5)/self.dimensions.1 as f32,
                );
                (uv, px)
            })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {

        let mut img = RgbaImage::new(self.dimensions.0, self.dimensions.1);

        self.buffer
            .iter()
            .map(|px| { 
                let mut px = 256.0 * px;
                px.apply(|x| *x = clamp(*x, 0.0, 255.0));
                [px.x as u8, px.y as u8, px.z as u8, px.w as u8]
            })
            .zip(img.pixels_mut())
            .for_each(|(value,target)| *target = value.into());

        img.save(path).unwrap();
    }

}   

