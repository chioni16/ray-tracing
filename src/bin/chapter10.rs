use ray::camera::Camera;
use ray::colour::Colour;
use ray::float4::Float4;
use ray::matrix::{rotate_x, rotate_y, rotate_z, scale, shear, translate, view_transform, Matrix};
use ray::object::{Material, Object, PointLight, Shape};
use ray::pattern::{Pattern, PatternKind};
use ray::world::World;

use std::f64::consts::PI;
use std::path::Path;

const CANVAS_WIDTH: usize = 3840 / 2;
const CANVAS_HEIGHT: usize = 2160 / 2;

fn main() {
    let background_material = Material {
        colour: Colour::new(0.5, 0.45, 0.45),
        specular: 0.0,
        pattern: Some(Pattern {
            kind: PatternKind::Stripe(Colour::white(), Colour::black()),
            // transform: Matrix::identity(4),
            transform: rotate_z(PI / 4.0),
        }),
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
            pattern: Some(Pattern {
                kind: PatternKind::Ring(Colour::new(1.0, 0.0, 0.0), Colour::new(0.0, 0.0, 1.0)),
                transform: rotate_x(PI / 3.0) * scale(0.25, 0.75, 0.8),
            }),
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
            pattern: Some(Pattern {
                kind: PatternKind::Gradient(Colour::new(1.0, 1.0, 0.0), Colour::new(1.0, 0.0, 1.0)),
                transform: scale(1.0, 2.0, 3.0),
            }),
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
            pattern: Some(Pattern {
                kind: PatternKind::Checkers(Colour::new(0.0, 1.0, 0.0), Colour::new(0.0, 1.0, 1.0)),
                transform: translate(1.0, 2.0, 3.0),
            }),
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
    image.to_file(Path::new("images/chapter10.ppm")).unwrap();
}
