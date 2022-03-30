use std::{ops::{Mul, Add, Sub, Div, MulAssign, AddAssign, SubAssign, DivAssign}};

use nalgebra::{Scalar, ClosedMul, ClosedAdd, ClosedSub, ClosedDiv, Vector3};
use num_traits::{Zero};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
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
}

impl<T: Scalar> Spectrum<T> where
    T: Zero,
{
    pub fn black() -> Self {
        Self::constant(T::zero())
    }
}

impl<T: Scalar> Copy for Spectrum<T> where
    T: Copy {}


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
