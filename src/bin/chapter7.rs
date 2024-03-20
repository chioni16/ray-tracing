use ray::camera::Camera;
use ray::colour::Colour;
use ray::float4::Float4;
use ray::matrix::{rotate_x, rotate_y, scale, translate, view_transform};
use ray::object::{Material, Object, PointLight};
use ray::world::World;

use std::f64::consts::PI;
use std::path::Path;

const CANVAS_WIDTH: usize = 3840 / 4;
const CANVAS_HEIGHT: usize = 2160 / 4;

fn main() {
    let background_material = Material {
        colour: Colour::new(0.5, 0.45, 0.45),
        specular: 0.0,
        ..Default::default()
    };
    let floor = Object::Sphere(scale(10.0, 0.01, 10.0), background_material);

    let left_wall = Object::Sphere(
        translate(0.0, 0.0, 5.0)
            * rotate_y(-PI / 4.0)
            * rotate_x(PI / 2.0)
            * scale(10.0, 0.01, 10.0),
        background_material,
    );

    let right_wall = Object::Sphere(
        translate(0.0, 0.0, 5.0)
            * rotate_y(PI / 4.0)
            * rotate_x(PI / 2.0)
            * scale(10.0, 0.01, 10.0),
        background_material,
    );

    let middle = Object::Sphere(
        translate(-0.5, 1.0, 0.5),
        Material {
            colour: Colour::new(1.0, 0.49, 0.0),
            diffuse: 0.7,
            specular: 0.1,
            shininess: 50.0,
            ..Default::default()
        },
    );
    let right = Object::Sphere(
        translate(1.5, 0.5, -0.5) * scale(0.5, 0.5, 0.5),
        Material {
            colour: Colour::new(0.51, 0.75, 0.06),
            // diffuse: 0.7,
            // specular: 0.3,
            ..Default::default()
        },
    );
    let left = Object::Sphere(
        translate(-1.5, 0.33, -0.75) * scale(0.33, 0.33, 0.33),
        Material {
            colour: Colour::new(0.78, 0.28, 0.96),
            // diffuse: 0.7,
            // specular: 0.3,
            ..Default::default()
        },
    );

    let world = World {
        light: PointLight {
            position: Float4::new_point(-10.0, 10.0, -10.0),
            intensity: Colour::white(),
        },
        objects: vec![floor, left_wall, right_wall, middle, left, right],
    };

    let camera = Camera::new(
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        PI / 3.0,
        view_transform(
            Float4::new_point(0.0, 3.5, -5.0),
            Float4::new_point(0.0, 1.0, 0.0),
            Float4::new_vector(0.0, 1.0, 0.0),
        ),
    );

    let image = camera.render(world);
    image.to_file(Path::new("images/chapter8.ppm")).unwrap();
}
