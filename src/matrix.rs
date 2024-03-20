use crate::{float4::Float4, util::float_is_eq};

#[derive(Debug, Clone)]
pub struct Matrix(pub Vec<Vec<f64>>);

impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self(vec![vec![0.0; width]; height])
    }

    pub fn identity(size: usize) -> Self {
        let mut identity = Self::new(size, size);
        for i in 0..size {
            identity.0[i][i] = 1.0;
        }
        identity
    }

    pub fn multiply(&self, rhs: &Self) -> Self {
        assert_eq!(self.0[0].len(), rhs.0.len());

        let ot = rhs.transpose();

        let prod = self
            .0
            .iter()
            .map(|sr| {
                ot.0.iter()
                    .map(|otr| sr.iter().zip(otr.iter()).map(|(a, b)| a * b).sum())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self(prod)
    }

    pub fn transpose(&self) -> Self {
        let transpose = (0..self.0[0].len())
            .map(|i| self.0.iter().map(|inner| inner[i]).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        Self(transpose)
    }

    pub fn submatrix(&self, row: usize, col: usize) -> Self {
        assert!(row < self.0.len());
        assert!(col < self.0[0].len());

        let mut sub = self.clone();
        sub.0.remove(row);
        for row in &mut sub.0 {
            row.remove(col);
        }
        sub
    }

    pub fn determinant(&self) -> f64 {
        assert_eq!(self.0.len(), self.0[0].len());

        if self.0.len() == 2 {
            self.0[0][0] * self.0[1][1] - self.0[1][0] * self.0[0][1]
        } else {
            self.0[0]
                .iter()
                .enumerate()
                .map(|(col, e)| e * self.cofactor(0, col))
                .sum()
        }
    }

    pub fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).determinant()
    }

    pub fn cofactor(&self, row: usize, col: usize) -> f64 {
        let minor = self.minor(row, col);
        if (row + col) % 2 == 1 {
            -1.0 * minor
        } else {
            minor
        }
    }

    pub fn inverse(&self) -> Option<Self> {
        if float_is_eq(self.determinant(), 0.0) {
            return None;
        }

        let mut inverse = Self::new(self.0[0].len(), self.0.len());
        let det = self.determinant();
        for row in 0..self.0.len() {
            for col in 0..self.0[0].len() {
                inverse.0[col][row] = self.cofactor(row, col) / det;
            }
        }
        Some(inverse)
    }
}

impl std::ops::Mul<Matrix> for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(&rhs)
    }
}

impl std::ops::Mul<Float4> for Matrix {
    type Output = Float4;

    fn mul(self, rhs: Float4) -> Self::Output {
        let rhs: Matrix = rhs.into();
        (self * rhs).into()
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        if self.0[0].len() != other.0[0].len() {
            return false;
        }
        self.0
            .iter()
            .flat_map(|row| row.iter())
            .zip(other.0.iter().flat_map(|row| row.iter()))
            .all(|(a, b)| float_is_eq(*a, *b))
    }
}

impl From<Float4> for Matrix {
    fn from(value: Float4) -> Self {
        Matrix(vec![
            vec![value.0[0]],
            vec![value.0[1]],
            vec![value.0[2]],
            vec![value.0[3]],
        ])
    }
}

pub fn translate(x: f64, y: f64, z: f64) -> Matrix {
    Matrix(vec![
        vec![1.0, 0.0, 0.0, x],
        vec![0.0, 1.0, 0.0, y],
        vec![0.0, 0.0, 1.0, z],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn scale(x: f64, y: f64, z: f64) -> Matrix {
    Matrix(vec![
        vec![x, 0.0, 0.0, 0.0],
        vec![0.0, y, 0.0, 0.0],
        vec![0.0, 0.0, z, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn rotate_x(r: f64) -> Matrix {
    Matrix(vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, r.cos(), -r.sin(), 0.0],
        vec![0.0, r.sin(), r.cos(), 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn rotate_y(r: f64) -> Matrix {
    Matrix(vec![
        vec![r.cos(), 0.0, r.sin(), 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![-r.sin(), 0.0, r.cos(), 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn rotate_z(r: f64) -> Matrix {
    Matrix(vec![
        vec![r.cos(), -r.sin(), 0.0, 0.0],
        vec![r.sin(), r.cos(), 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn shear(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix {
    Matrix(vec![
        vec![1.0, xy, xz, 0.0],
        vec![yx, 1.0, yz, 0.0],
        vec![zx, zy, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ])
}

pub fn view_transform(from: Float4, to: Float4, up: Float4) -> Matrix {
    let forward = (to - from).normalise();
    let left = forward.cross(up.normalise());
    let true_up = left.cross(forward);

    let orientation = Matrix(vec![
        vec![left.0[0], left.0[1], left.0[2], 0.0],
        vec![true_up.0[0], true_up.0[1], true_up.0[2], 0.0],
        vec![-forward.0[0], -forward.0[1], -forward.0[2], 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
    ]);

    orientation * translate(-from.0[0], -from.0[1], -from.0[2])
}

mod test {
    use super::*;
    use crate::float4::*;
    use std::f64::consts::PI;

    #[test]
    fn identity_transpose() {
        let a = Matrix::identity(5);
        let b = a.transpose();
        assert_eq!(a, b);
    }

    #[test]
    fn identity_multipty() {
        let a = Matrix::identity(4);
        let b = Matrix(vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![9.0, 10.0, 11.0, 12.0],
            vec![13.0, 14.0, 15.0, 16.0],
        ]);
        assert_eq!(b, a.multiply(&b));
        assert_eq!(b.multiply(&a), a.multiply(&b));
    }

    #[test]
    fn determinant_2() {
        let matrix = Matrix(vec![vec![1.0, 5.0], vec![-3.0, 2.0]]);
        assert!(float_is_eq(matrix.determinant(), 17.0));
    }

    #[test]
    fn determinant_3() {
        let matrix = Matrix(vec![
            vec![1.0, 2.0, 6.0],
            vec![-5.0, 8.0, -4.0],
            vec![2.0, 6.0, 4.0],
        ]);
        assert!(float_is_eq(matrix.cofactor(0, 0), 56.0));
        assert!(float_is_eq(matrix.cofactor(0, 1), 12.0));
        assert!(float_is_eq(matrix.cofactor(0, 2), -46.0));
        assert!(float_is_eq(matrix.determinant(), -196.0));
    }

    #[test]
    fn determinant_4() {
        let matrix = Matrix(vec![
            vec![-2.0, -8.0, 3.0, 5.0],
            vec![-3.0, 1.0, 7.0, 3.0],
            vec![1.0, 2.0, -9.0, 6.0],
            vec![-6.0, 7.0, 7.0, -9.0],
        ]);
        assert!(float_is_eq(matrix.cofactor(0, 0), 690.0));
        assert!(float_is_eq(matrix.cofactor(0, 1), 447.0));
        assert!(float_is_eq(matrix.cofactor(0, 2), 210.0));
        assert!(float_is_eq(matrix.cofactor(0, 3), 51.0));
        assert!(float_is_eq(matrix.determinant(), -4071.0));
    }

    #[test]
    fn inverse_1() {
        let matrix = Matrix(vec![
            vec![8.0, -5.0, 9.0, 2.0],
            vec![7.0, 5.0, 6.0, 1.0],
            vec![-6.0, 0.0, 9.0, 6.0],
            vec![-3.0, 0.0, -9.0, -4.0],
        ]);
        let expected = Matrix(vec![
            vec![-0.15385, -0.15385, -0.28205, -0.53846],
            vec![-0.07692, 0.12308, 0.02564, 0.03077],
            vec![0.35897, 0.35897, 0.43590, 0.92308],
            vec![-0.69231, -0.69231, -0.76923, -1.92308],
        ]);
        assert_eq!(matrix.inverse().unwrap(), expected);
    }

    #[test]
    fn inverse_2() {
        let matrix = Matrix(vec![
            vec![9.0, 3.0, 0.0, 9.0],
            vec![-5.0, -2.0, -6.0, -3.0],
            vec![-4.0, 9.0, 6.0, 4.0],
            vec![-7.0, 6.0, 6.0, 2.0],
        ]);
        let expected = Matrix(vec![
            vec![-0.04074, -0.07778, 0.14444, -0.22222],
            vec![-0.07778, 0.03333, 0.36667, -0.33333],
            vec![-0.02901, -0.14630, -0.10926, 0.12963],
            vec![0.17778, 0.06667, -0.26667, 0.33333],
        ]);
        assert_eq!(matrix.inverse().unwrap(), expected);
    }

    #[test]
    fn inverse_3() {
        let a = Matrix(vec![
            vec![3.0, -9.0, 7.0, 3.0],
            vec![3.0, -8.0, 2.0, -9.0],
            vec![-4.0, 4.0, 4.0, 1.0],
            vec![-6.0, 5.0, -1.0, 1.0],
        ]);
        let b = Matrix(vec![
            vec![8.0, 2.0, 2.0, 2.0],
            vec![3.0, -1.0, 7.0, 0.0],
            vec![7.0, 0.0, 5.0, 4.0],
            vec![6.0, -2.0, 0.0, 5.0],
        ]);
        let c = a.multiply(&b);
        assert_eq!(c.multiply(&b.inverse().unwrap()), a);
    }

    #[test]
    fn translation_1() {
        let t = translate(5.0, -3.0, 2.0);
        let p: Matrix = Float4::new_point(-3.0, 4.0, 5.0).into();
        let expected: Matrix = Float4::new_point(2.0, 1.0, 7.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn translation_2() {
        let t = translate(5.0, -3.0, 2.0).inverse().unwrap();
        let p: Matrix = Float4::new_point(-3.0, 4.0, 5.0).into();
        let expected: Matrix = Float4::new_point(-8.0, 7.0, 3.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn translation_3() {
        let t = translate(5.0, -3.0, 2.0).inverse().unwrap();
        let p: Matrix = Float4::new_vector(-3.0, 4.0, 5.0).into();
        assert_eq!(t.multiply(&p), p);
    }

    #[test]
    fn scaling_1() {
        let t = scale(2.0, 3.0, 4.0);
        let p: Matrix = Float4::new_point(-4.0, 6.0, 8.0).into();
        let expected: Matrix = Float4::new_point(-8.0, 18.0, 32.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn scaling_2() {
        let t = scale(2.0, 3.0, 4.0);
        let p: Matrix = Float4::new_vector(-4.0, 6.0, 8.0).into();
        let expected: Matrix = Float4::new_vector(-8.0, 18.0, 32.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn scaling_3() {
        let t = scale(2.0, 3.0, 4.0).inverse().unwrap();
        let p: Matrix = Float4::new_vector(-4.0, 6.0, 8.0).into();
        let expected: Matrix = Float4::new_vector(-2.0, 2.0, 2.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn reflection_1() {
        let t = scale(-1.0, 1.0, 1.0).inverse().unwrap();
        let p: Matrix = Float4::new_vector(2.0, 3.0, 4.0).into();
        let expected: Matrix = Float4::new_vector(-2.0, 3.0, 4.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn rotation_x() {
        let p: Matrix = Float4::new_point(0.0, 1.0, 0.0).into();

        let t = rotate_x(PI / 4.0);
        let expected: Matrix =
            Float4::new_point(0.0, 1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt()).into();
        assert_eq!(t.multiply(&p), expected);

        let t = rotate_x(PI / 2.0);
        let expected: Matrix = Float4::new_point(0.0, 0.0, 1.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = rotate_x(PI / 4.0).inverse().unwrap();
        let expected: Matrix =
            Float4::new_point(0.0, 1.0 / 2.0_f64.sqrt(), -1.0 / 2.0_f64.sqrt()).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn rotation_y() {
        let p: Matrix = Float4::new_point(0.0, 0.0, 1.0).into();

        let t = rotate_y(PI / 4.0);
        let expected: Matrix =
            Float4::new_point(1.0 / 2.0_f64.sqrt(), 0.0, 1.0 / 2.0_f64.sqrt()).into();
        assert_eq!(t.multiply(&p), expected);

        let t = rotate_y(PI / 2.0);
        let expected: Matrix = Float4::new_point(1.0, 0.0, 0.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn rotation_z() {
        let p: Matrix = Float4::new_point(0.0, 1.0, 0.0).into();

        let t = rotate_z(PI / 4.0);
        let expected: Matrix =
            Float4::new_point(-1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt(), 0.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = rotate_z(PI / 2.0);
        let expected: Matrix = Float4::new_point(-1.0, 0.0, 0.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn shearing() {
        let p: Matrix = Float4::new_point(2.0, 3.0, 4.0).into();

        let t = shear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let expected: Matrix = Float4::new_point(5.0, 3.0, 4.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = shear(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let expected: Matrix = Float4::new_point(6.0, 3.0, 4.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = shear(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let expected: Matrix = Float4::new_point(2.0, 5.0, 4.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = shear(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let expected: Matrix = Float4::new_point(2.0, 7.0, 4.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = shear(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let expected: Matrix = Float4::new_point(2.0, 3.0, 6.0).into();
        assert_eq!(t.multiply(&p), expected);

        let t = shear(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let expected: Matrix = Float4::new_point(2.0, 3.0, 7.0).into();
        assert_eq!(t.multiply(&p), expected);
    }

    #[test]
    fn view_transformation() {
        let from1 = Float4::origin();
        let to1 = Float4::new_point(0.0, 0.0, 1.0);
        let up1 = Float4::new_vector(0.0, 1.0, 0.0);
        assert_eq!(view_transform(from1, to1, up1), scale(-1.0, 1.0, -1.0));

        let from2 = Float4::origin();
        let to2 = Float4::new_point(0.0, 0.0, -1.0);
        let up2 = Float4::new_vector(0.0, 1.0, 0.0);
        assert_eq!(view_transform(from2, to2, up2), Matrix::identity(4));

        let from3 = Float4::new_point(0.0, 0.0, 8.0);
        let to3 = Float4::origin();
        let up3 = Float4::new_vector(0.0, 1.0, 0.0);
        assert_eq!(view_transform(from3, to3, up3), translate(0.0, 0.0, -8.0));

        let from4 = Float4::new_point(1.0, 3.0, 2.0);
        let to4 = Float4::new_point(4.0, -2.0, 8.0);
        let up4 = Float4::new_vector(1.0, 1.0, 0.0);
        assert_eq!(
            view_transform(from4, to4, up4),
            Matrix(vec![
                vec![-0.50709, 0.50709, 0.67612, -2.36643],
                vec![0.76772, 0.60609, 0.12122, -2.82843],
                vec![-0.35857, 0.59761, -0.71714, 0.0],
                vec![0.0, 0.0, 0.0, 1.0],
            ])
        );
    }
}
