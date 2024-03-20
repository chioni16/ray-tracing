pub const EPSILON: f64 = 1e-5;

pub fn float_is_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}
