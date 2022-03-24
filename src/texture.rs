use std::{io::{BufReader, Result, Error, ErrorKind}, fs::File, path::Path, ops::Mul};

use image::{codecs::hdr, RgbaImage};
use itertools::Itertools;
use rayon::prelude::*;
use nalgebra::{Point2, Vector3, SVector, Scalar, ClosedMul, ClosedDiv};

pub enum ColorSpace {
    Linear,
    Srgb,
}

pub struct Texture<T> {
    size: (u32, u32),
    data: Vec<T>,
}

pub type RenderTarget = Texture<Vector3<f32>>;

impl<T> Texture<T> where
    T: Copy
{
    pub fn new(width: u32, height: u32, fill_col: &T) -> Self {
        let size = (width, height);
        let data = vec![*fill_col; width as usize * height as usize];

        Self { size, data, }
    }

    pub fn sample<U>(&self, uv: &Point2<f32>) -> U where
        U: From<T>
    {
        let u = uv[0];
        let v = uv[1];

        let x = ((u * self.size.0 as f32) as i32).rem_euclid(self.size.0 as i32);
        let y = ((v * self.size.1 as f32) as i32).rem_euclid(self.size.1 as i32);
        
        let index = y as usize * self.size.0 as usize + x as usize;

        self.data[index].into()
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn pixels_mut<'a>(&'a mut self) -> impl Iterator<Item=(Point2<f32>, &mut T)> + 'a {
        self.data
            .iter_mut()
            .enumerate()
            .map(|(pos, px)| {
                let pos = pos as u32;
                let x = pos % self.size.0;
                let y = self.size.1 - pos / self.size.0;
                let uv = Point2::new(
                    (x as f32 + 0.5)/self.size.0 as f32,
                    (y as f32 + 0.5)/self.size.1 as f32,
                );
                (uv, px)
            })
    }
}

impl<T> Texture<T> where
    T: Send + Sync 
{
    pub fn pixels_par_mut<'a>(&'a mut self) -> impl ParallelIterator<Item=(Point2<f32>, &mut T)> + 'a {
        self.data
            .par_iter_mut()
            .enumerate()
            .map(|(pos, px)| {
                let x = pos as u32 % self.size.0;
                let y = pos as u32 / self.size.0;
                let uv = Point2::new(
                    (x as f32 + 0.5)/self.size.0 as f32,
                    1.0 - (y as f32 + 0.5)/self.size.1 as f32,
                );
                (uv, px)
            })
    }
}

pub trait PixelComponent: Copy + Scalar + ClosedMul + ClosedDiv {
    const MAX: Self;

    fn map<T>(u: T) -> Self where
        T: PixelComponent,
        Self: From<T>
    {
        Self::from(u) * Self::MAX / Self::from(T::MAX)
    }
}

macro_rules! impl_pixel_component {
    ($ty:ty, $min:expr, $max:expr) => {
        impl PixelComponent for $ty {
            const MAX: Self = $max;
        }
    };
}

impl_pixel_component!(u8, 0, 255);
impl_pixel_component!(f32, 0.0, 1.0);

impl<T, const D: usize> Texture<SVector<T, D>> where
    T: PixelComponent {
    pub fn from_raw_data<U, const K: usize>(width: u32, height: u32, data: &[u8]) -> Result<Self> where
        U: PixelComponent,
        T: From<U>,
    {
        assert!(D <= K);

        let component_count = K;
        let component_size = std::mem::size_of::<U>();
        let pixel_count = width as usize * height as usize;
        let pixel_size = component_count * component_size;

        if pixel_count * pixel_size != data.len() {
            return Err(Error::from(ErrorKind::InvalidData));
        }

        let buffer = (0..pixel_count)
            .map(|i| {
                let pixel_data = &data[i*pixel_size..(i+1)*pixel_size];
                let pixel_data = unsafe { std::slice::from_raw_parts(pixel_data.as_ptr() as *const U, D) };
                SVector::<T, D>::from_fn(|j, _| T::map(pixel_data[j]))
            })
            .collect_vec();

        Ok(Self {
            size: (width, height),
            data: buffer,
        })
    }

}


impl Texture<Vector3<f32>> {
    pub fn from_hdr_file(path: &str) -> Self {
        let reader = BufReader::new(File::open(path).unwrap());
        let decoder = hdr::HdrDecoder::new(reader).unwrap();
        let meta = decoder.metadata();
        let size = (meta.width, meta.height);
        let data = decoder.read_image_hdr().unwrap();
        
        let data = data.into_iter()
            .map(|px| Vector3::new(px[0], px[1], px[2]))
            .collect_vec();
        
        Texture { size, data }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let mut img = RgbaImage::new(self.size.0, self.size.1);

        self.data
            .iter()
            .copied()
            .map(|mut px| { 
                px.apply(|x| *x = x.powf(1.0/2.2).mul(255.0).clamp(0.0, 255.0));
                [px.x as u8, px.y as u8, px.z as u8, 255u8]
            })
            .zip(img.pixels_mut())
            .for_each(|(value,target)| *target = value.into());

        img.save(path).unwrap();
    }

}



