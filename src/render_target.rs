use nalgebra::Vector4;

use rayon::prelude::*;

pub struct RenderTarget {
    dimensions: (usize, usize),
    buffer: Vec<Vector4<f32>>,
}

impl RenderTarget {

    pub fn new(width: usize, height: usize, fill_col: &Vector4<f32>) -> RenderTarget {
        let dimensions = (width, height);
        let buffer = vec![*fill_col; width * height];

        RenderTarget { dimensions, buffer, }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.dimensions.0 as f32 / self.dimensions.1 as f32
    }
    
    pub fn pixels_mut(&mut self) -> <Vec<Vector4<f32>> as IntoParallelRefMutIterator>::Iter {
        self.buffer.par_iter_mut()
    }

}

