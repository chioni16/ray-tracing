use crate::float4::*;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colour(pub Float4);

impl Colour {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Float4::new_vector(r, g, b))
    }

    pub const fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn scalar_product(self, f: f64) -> Self {
        self.0.scalar_mul(f).into()
    }

    pub fn hadamard_product(self, rhs: Self) -> Self {
        Self::new(
            self.0 .0[0] * rhs.0 .0[0],
            self.0 .0[1] * rhs.0 .0[1],
            self.0 .0[2] * rhs.0 .0[1],
        )
    }
}

impl From<Float4> for Colour {
    fn from(value: Float4) -> Self {
        Self::new(value.0[0], value.0[1], value.0[2])
    }
}

impl Add for Colour {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.0.add(rhs.0).into()
    }
}

impl Sub for Colour {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.sub(rhs.0).into()
    }
}

impl Mul<Colour> for Colour {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.hadamard_product(rhs)
    }
}
