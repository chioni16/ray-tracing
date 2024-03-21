#![feature(iter_intersperse)]

pub mod camera;
pub mod canvas;
pub mod colour;
pub mod float4;
pub mod matrix;
pub mod object;
pub mod pattern;
pub mod ray;
pub mod util;
pub mod world;

const REF_RECURSION_LIMIT: u8 = 5;
