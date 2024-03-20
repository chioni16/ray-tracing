use std::ops::{Add, Neg, Sub};

use crate::{matrix::Matrix, util::float_is_eq};

#[derive(Debug, Clone, Copy)]
pub struct Float4(pub [f64; 4]);

impl Float4 {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self([x, y, z, w])
    }

    pub fn new_point(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z, 1.0])
    }

    pub fn origin() -> Self {
        Self::new_point(0.0, 0.0, 0.0)
    }

    pub const fn new_vector(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z, 0.0])
    }

    pub fn is_point(self) -> bool {
        float_is_eq(self.0[3], 1.0)
    }

    pub fn is_vector(self) -> bool {
        float_is_eq(self.0[3], 0.0)
    }

    pub fn scalar_mul(self, rhs: f64) -> Self {
        Self::new(
            self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs,
        )
    }

    // pub fn scalar_div(self, rhs: f64) -> Self {
    //     self.scalar_mul(-1.0 / rhs) // @TODO
    // }

    pub fn mag(&self) -> f64 {
        (self.0[0] * self.0[0]
            + self.0[1] * self.0[1]
            + self.0[2] * self.0[2]
            + self.0[3] * self.0[3])
            .sqrt()
    }

    pub fn normalise(self) -> Self {
        let mag = self.mag();
        Self::new(
            self.0[0] / mag,
            self.0[1] / mag,
            self.0[2] / mag,
            self.0[3] / mag,
        )
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.0[0] * rhs.0[0] + self.0[1] * rhs.0[1] + self.0[2] * rhs.0[2] + self.0[3] * rhs.0[3]
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new_vector(
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        )
    }

    pub fn reflect(&self, normal: Self) -> Self {
        assert!(self.is_vector() && normal.is_vector());

        self.sub(normal.scalar_mul(2.0 * self.dot(normal)))
    }
}

impl Add for Float4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        )
    }
}

impl Sub for Float4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.add(rhs.neg())
    }
}

impl Neg for Float4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.scalar_mul(-1.0)
    }
}

impl PartialEq for Float4 {
    fn eq(&self, other: &Self) -> bool {
        float_is_eq(self.0[0], other.0[0])
            && float_is_eq(self.0[1], other.0[1])
            && float_is_eq(self.0[2], other.0[2])
            && float_is_eq(self.0[3], other.0[3])
    }
}

impl From<Matrix> for Float4 {
    fn from(value: Matrix) -> Self {
        assert!(value.0.len() == 4 && value.0[0].len() == 1);
        Self([value.0[0][0], value.0[1][0], value.0[2][0], value.0[3][0]])
    }
}

mod test {
    use super::*;

    #[test]
    fn reflect() {
        let v = Float4::new_vector(1.0, -1.0, 0.0);
        let n = Float4::new_vector(0.0, 1.0, 0.0);
        assert_eq!(v.reflect(n), Float4::new_vector(1.0, 1.0, 0.0));

        let v = Float4::new_vector(0.0, -1.0, 0.0);
        let n = Float4::new_vector(1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt(), 0.0);
        assert_eq!(v.reflect(n), Float4::new_vector(1.0, 0.0, 0.0));
    }
}
