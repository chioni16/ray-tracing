use ray::camera::Camera;
use ray::colour::Colour;
use ray::float4::Float4;
use ray::matrix::{rotate_x, rotate_y, rotate_z, scale, translate, view_transform, Matrix};
use ray::object::{Material, Object, PointLight, Shape};
use ray::world::World;

use std::f64::consts::PI;
use std::path::Path;

const CANVAS_WIDTH: usize = 3840 / 8;
const CANVAS_HEIGHT: usize = 2160 / 8;

fn main() {
    let background_material = Material {
        colour: Colour::new(0.5, 0.45, 0.45),
        specular: 0.0,
        ..Default::default()
    };
    let floor = Object {
        shape: Shape::Plane,
        // transform: Matrix::identity(4),
        // transform: rotate_z(PI / 4.0),
        transform: Matrix::identity(4),
        material: background_material,
    };

    let middle = Object {
        shape: Shape::Sphere,
        transform: translate(-0.5, 1.0, 0.5),
        material: Material {
            colour: Colour::new(1.0, 0.49, 0.0),
            diffuse: 0.7,
            specular: 0.1,
            shininess: 50.0,
            ..Default::default()
        },
    };
    let right = Object {
        shape: Shape::Sphere,
        transform: translate(1.5, 0.5, -0.5) * scale(0.5, 0.5, 0.5),
        material: Material {
            colour: Colour::new(0.51, 0.75, 0.06),
            // diffuse: 0.7,
            // specular: 0.3,
            ..Default::default()
        },
    };
    let left = Object {
        shape: Shape::Sphere,
        transform: translate(-1.5, 0.33, -0.75) * scale(0.33, 0.33, 0.33),
        material: Material {
            colour: Colour::new(0.78, 0.28, 0.96),
            // diffuse: 0.7,
            // specular: 0.3,
            ..Default::default()
        },
    };

    let world = World {
        light: PointLight {
            position: Float4::new_point(-10.0, 10.0, -10.0),
            colour: Colour::white(),
        },
        objects: vec![floor, middle, left, right],
    };

    let camera = Camera::new(
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        PI / 3.0,
        view_transform(
            Float4::new_point(0.0, 1.5, -5.0),
            Float4::new_point(0.0, 1.0, 0.0),
            Float4::new_vector(0.0, 1.0, 0.0),
        ),
    );

    let image = camera.render(world);
    image.to_file(Path::new("images/chapter9_5.ppm")).unwrap();
}
