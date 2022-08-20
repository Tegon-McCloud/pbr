use std::{ops::{Mul, Add, Sub, Div, MulAssign, AddAssign, SubAssign, DivAssign}};

use nalgebra::{Scalar, ClosedMul, ClosedAdd, ClosedSub, ClosedDiv, Vector3};
use num_traits::{Zero, One};

pub struct SpectrumIter<'a, T: Scalar> {
    next_wavelength: usize,
    spectrum: &'a Spectrum<T>
}

impl<'a, T: Scalar> Iterator for SpectrumIter<'a, T> {

    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.next_wavelength {
            0 => Some(&self.spectrum.r),
            1 => Some(&self.spectrum.g),
            2 => Some(&self.spectrum.b),
            _ => None,
        };

        self.next_wavelength += 1;

        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = 3 - self.next_wavelength;
        (len, Some(len))
    }

}

impl<T: Scalar> ExactSizeIterator for SpectrumIter<'_, T> {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spectrum<T: Scalar> {
    pub r: T,
    pub g: T,
    pub b: T,
    _private: (),
}

impl<T: Scalar> Spectrum<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b, _private: () }
    }

    pub fn constant(c: T) -> Self {
        Self::new(c.clone(), c.clone(), c.clone())
    }

    pub fn apply(&mut self, f: impl Fn(&mut T)) {
        f(&mut self.r);
        f(&mut self.g);
        f(&mut self.b);
    }

    pub fn apply_into(mut self, f: impl Fn(T) -> T) -> Spectrum<T> {
        self.r = f(self.r);
        self.g = f(self.g);
        self.b = f(self.b);
        self
    }

    pub fn iter<'s>(&'s self) -> impl Iterator<Item = &T> + 's {
        SpectrumIter { next_wavelength: 0, spectrum: self }
    }
}

impl<T> Spectrum<T> where
    T: Scalar + Copy + One + ClosedAdd<T> + ClosedSub<T> + ClosedMul<T>
{ 
    pub fn lerp(a: &Spectrum<T>, b: &Spectrum<T>, t: T) -> Spectrum<T> {
        a * (T::one() - t) + b * t
    }
}

impl Spectrum<f32> {
    
    pub fn any_nan(&self) -> bool {
        self.r.is_nan() ||
        self.g.is_nan() ||
        self.b.is_nan()
    } 
}

impl<T: Scalar> Spectrum<T> where
    T: Zero,
{
    pub fn black() -> Self {
        Self::constant(T::zero())
    }
}

impl<T: Scalar> From<&[T]> for Spectrum<T> where
    T: Copy
{
    fn from(s: &[T]) -> Self {
        Self::new(s[0], s[1], s[2])
    }
}

impl<T: Scalar> From<Spectrum<T>> for Vector3<T> {
    fn from(s: Spectrum<T>) -> Self {
        Vector3::new(s.r, s.g, s.b)
    }
}

macro_rules! componentwise_binop_impl {
    ($Trait: ident, $method: ident, $bound: ident) => {

        impl<T: Scalar> $Trait<Spectrum<T>> for Spectrum<T> where
            T: $bound
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: Spectrum<T>) -> Self::Output {
                Spectrum::new(self.r.$method(rhs.r), self.g.$method(rhs.g), self.b.$method(rhs.b))
            }
        }
        
        impl<T: Scalar> $Trait<Spectrum<T>> for &Spectrum<T> where
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: Spectrum<T>) -> Self::Output {
                Spectrum::new(self.r.$method(rhs.r), self.g.$method(rhs.g), self.b.$method(rhs.b))
            }
        }

        impl<T: Scalar> $Trait<&Spectrum<T>> for &Spectrum<T> where
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: &Spectrum<T>) -> Self::Output {
                Spectrum::new(self.r.$method(rhs.r), self.g.$method(rhs.g), self.b.$method(rhs.b))
            }
        }

        impl<T: Scalar> $Trait<&Spectrum<T>> for Spectrum<T> where
        T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: &Spectrum<T>) -> Self::Output {
                Spectrum::new(self.r.$method(rhs.r), self.g.$method(rhs.g), self.b.$method(rhs.b))
            }
        }
    };
}

componentwise_binop_impl!(Add, add, ClosedAdd);
componentwise_binop_impl!(Sub, sub, ClosedSub);
componentwise_binop_impl!(Mul, mul, ClosedMul);
componentwise_binop_impl!(Div, div, ClosedDiv);

macro_rules! componentwise_binop_assign_impl {
    ($Trait: ident, $method: ident, $bound: ident) => {
        impl<T: Scalar> $Trait<Spectrum<T>> for Spectrum<T> where
            T: $bound 
        {
            fn $method(&mut self, rhs: Spectrum<T>) {
                self.r.$method(rhs.r);
                self.g.$method(rhs.g);
                self.b.$method(rhs.b);
            }
        }

        impl<T: Scalar> $Trait<&Spectrum<T>> for Spectrum<T> where
            T: $bound + Copy
        {
            fn $method(&mut self, rhs: &Spectrum<T>) {
                self.r.$method(rhs.r);
                self.g.$method(rhs.g);
                self.b.$method(rhs.b);
            }
        }
    };
}

componentwise_binop_assign_impl!(AddAssign, add_assign, ClosedAdd);
componentwise_binop_assign_impl!(SubAssign, sub_assign, ClosedSub);
componentwise_binop_assign_impl!(MulAssign, mul_assign, ClosedMul);
componentwise_binop_assign_impl!(DivAssign, div_assign, ClosedDiv);

macro_rules! scalar_binop_impl {
    ($Trait: ident, $method: ident, $bound: ident) => {
        impl<T: Scalar> $Trait<T> for Spectrum<T> where 
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: T) -> Self::Output {
                Spectrum::new(self.r.$method(rhs), self.g.$method(rhs), self.b.$method(rhs))
            }
        }

        impl<T: Scalar> $Trait<T> for &Spectrum<T> where 
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: T) -> Self::Output {
                Spectrum::new(self.r.$method(rhs), self.g.$method(rhs), self.b.$method(rhs))
            }
        }

        impl<T: Scalar> $Trait<&T> for &Spectrum<T> where 
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: &T) -> Self::Output {
                Spectrum::new(self.r.$method(*rhs), self.g.$method(*rhs), self.b.$method(*rhs))
            }
        }

        impl<T: Scalar> $Trait<&T> for Spectrum<T> where 
            T: $bound + Copy
        {
            type Output = Spectrum<T>;

            fn $method(self, rhs: &T) -> Self::Output {
                Spectrum::new(self.r.$method(*rhs), self.g.$method(*rhs), self.b.$method(*rhs))
            }
        }
    };
}

scalar_binop_impl!(Mul, mul, ClosedMul);
scalar_binop_impl!(Div, div, ClosedDiv);

macro_rules! scalar_binop_assign_impl {
    ($Trait: ident, $method: ident, $bound: ident) => {
        impl<T: Scalar> $Trait<T> for Spectrum<T> where
            T: $bound + Copy
        {
            fn $method(&mut self, rhs: T) {
                self.r.$method(rhs);
                self.g.$method(rhs);
                self.b.$method(rhs);
            }
        }

        impl<T: Scalar> $Trait<&T> for Spectrum<T> where
            T: $bound + Copy
        {
            fn $method(&mut self, rhs: &T) {
                self.r.$method(*rhs);
                self.g.$method(*rhs);
                self.b.$method(*rhs);
            }
        }
    };
}

scalar_binop_assign_impl!(MulAssign, mul_assign, ClosedMul);
scalar_binop_assign_impl!(DivAssign, div_assign, ClosedDiv);
