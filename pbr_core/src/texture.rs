use std::{io::{BufReader, Result, Error, ErrorKind}, fs::File, path::Path, ops::Mul};

use image::{codecs::hdr, RgbaImage};
use nalgebra::{Point2, Vector3, SVector, Scalar, ClosedMul, ClosedDiv};
use rayon::prelude::{IntoParallelRefIterator, IndexedParallelIterator, ParallelIterator, IntoParallelRefMutIterator};

use crate::spectrum::Spectrum;

pub enum ColorSpace {
    Linear,
    Srgb,
}

#[derive(Clone)]
pub struct Texture<T> {
    size: (u32, u32),
    data: Box<[T]>,
}

impl<T> Texture<T> {
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn set(&mut self, pos: Point2<u32>, val: T) {
        self.data[(pos.y * self.size.0 + pos.x) as usize] = val;
    }
}

impl<T> Texture<T> where
    T: Copy
{
    pub fn new(width: u32, height: u32, fill_col: &T) -> Self {
        let size = (width, height);
        let data = vec![*fill_col; width as usize * height as usize].into_boxed_slice();
        
        Self { size, data, }
    }

    pub fn sample(&self, uv: &Point2<f32>) -> T {
        let u = uv[0];
        let v = uv[1];

        let x = ((u * self.size.0 as f32) as i32).rem_euclid(self.size.0 as i32);
        let y = ((v * self.size.1 as f32) as i32).rem_euclid(self.size.1 as i32);
        
        let index = y as usize * self.size.0 as usize + x as usize;

        self.data[index]
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn pixels(&self) -> impl Iterator<Item = (Point2<u32>, &T)> {
        self.data
            .iter()
            .enumerate()
            .map(|(pos, px)| (
                Point2::new(pos as u32 % self.size.0, pos as u32 / self.size.0),
                px
            ))
    }

    pub fn pixels_mut(&mut self) -> impl Iterator<Item = (Point2<u32>, &mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .map(|(pos, px)| (
                Point2::new(pos as u32 % self.size.0, pos as u32 / self.size.0),
                px
            ))
    }

}

impl<T> Texture<T> where
    T: Sync
{
    pub fn par_pixels(&self) -> impl ParallelIterator<Item = (Point2<u32>, &T)> {
        self.data
            .par_iter()
            .enumerate()
            .map(|(pos, px)| (
                Point2::new(pos as u32 % self.size.0, pos as u32 / self.size.0),
                px
            ))
    }
}

impl<T> Texture<T> where
    T: Send
{
    pub fn par_pixels_mut(&mut self) -> impl ParallelIterator<Item = (Point2<u32>, &mut T)> {
        self.data
            .par_iter_mut()
            .enumerate()
            .map(|(pos, px)| (
                Point2::new(pos as u32 % self.size.0, pos as u32 / self.size.0),
                px
            ))
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

impl<T> Texture<Spectrum<T>> where
    T: PixelComponent,
{
    pub fn from_raw_data<U, const K: usize>(width: u32, height: u32, data: &[u8]) -> Result<Self> where
        U: PixelComponent,
        T: From<U>,
    {
        let vec_texture = Texture::<SVector<T, 3>>::from_raw_data::<U, K>(width, height, data)?;
        let (size, vec_data) = (vec_texture.size, vec_texture.data);

        let data = unsafe { std::mem::transmute(vec_data) };
        
        Ok(Self {
            size,
            data,
        })
    }
}

impl<T, const D: usize> Texture<SVector<T, D>> where
    T: PixelComponent 
{
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
            .collect();

        Ok(Self {
            size: (width, height),
            data: buffer,
        })
    }

}


impl Texture<Spectrum<f32>> {
    pub fn from_hdr_file(path: &str) -> Self {
        let reader = BufReader::new(File::open(path).unwrap());
        let decoder = hdr::HdrDecoder::new(reader).unwrap();
        let meta = decoder.metadata();
        let size = (meta.width, meta.height);
        let data = decoder.read_image_hdr().unwrap();
        
        let data = data.into_iter()
            .map(|px| Spectrum::new(px[0], px[1], px[2]))
            .collect();

        Texture { size, data }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let mut img = RgbaImage::new(self.size.0, self.size.1);

        self.data
            .iter()
            .copied()
            .map(|px| {
                let mut px: Vector3<f32> = px.into();
                px.apply(|x| *x = x.powf(1.0/2.2).mul(255.0).clamp(0.0, 255.0));
                [px.x as u8, px.y as u8, px.z as u8, 255u8]
            })
            .zip(img.pixels_mut())
            .for_each(|(value,target)| *target = value.into());

        img.save(path).unwrap();
    }
}


pub enum MaybeTexture<T> {
    Texture(Texture<T>),
    Value(T),
}

impl<T> MaybeTexture<T> where
    T: Copy,
{
    pub fn sample(&self, uv: &Point2<f32>) -> T {
        match self {
            Self::Texture(texture) => texture.sample(uv),
            Self::Value(value) => *value,
        }
    }
}

pub struct FactoredTexture<T> {
    pub factor: T,
    pub texture: Option<Texture<T>>,
}

impl<T> FactoredTexture<T> where
    T: Copy + ClosedMul
{
    pub fn new(factor: T, texture: Option<Texture<T>>) -> Self {
        Self {
            factor,
            texture,
        }
    }

    pub fn sample(&self, uv: &Point2<f32>) -> T {
        match &self.texture {
            Some(texture) => self.factor * texture.sample(uv),
            None => self.factor,
        }
    }
}