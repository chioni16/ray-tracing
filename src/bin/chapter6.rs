use ray::canvas::Canvas;
use ray::colour::Colour;
use ray::float4::Float4;
use ray::matrix::Matrix;
use ray::object::{Material, Object, PointLight, Shape};
use ray::ray::Ray;

use std::path::Path;
use std::sync::Mutex;

use itertools::Itertools;
use rayon::prelude::*;

const CANVAS_PIXELS: usize = 600;

fn main() {
    let canvas_mutex = Mutex::new(Canvas::new(CANVAS_PIXELS, CANVAS_PIXELS, Colour::black()));

    let sphere = Object {
        shape: Shape::Sphere,
        transform: Matrix::identity(4),
        // scale(1.0, 0.5, 1.0),
        // scale(0.5, 1.0, 1.0),
        // rotate_z(PI/4.0) * scale(0.5, 1.0, 1.0),
        // shear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0) * scale(0.5, 1.0, 1.0),
        material: Material {
            colour: Colour::new(0.0, 0.2, 1.0),
            ..Default::default()
        },
    };

    let light = PointLight {
        position: Float4::new_point(-10.0, 10.0, -10.0),
        colour: Colour::new(1.0, 1.0, 1.0),
    };

    let ray_origin = Float4::new_point(0.0, 0.0, -5.0);
    let wall_z = 10.0;

    let wall_size = 7.0;
    let half = wall_size / 2.0;
    let pixel_size = wall_size / CANVAS_PIXELS as f64;

    (0..CANVAS_PIXELS)
        .cartesian_product(0..CANVAS_PIXELS)
        .par_bridge()
        .for_each(|(y, x)| {
            let world_y = half - pixel_size * y as f64;
            let world_x = -half + pixel_size * x as f64;

            let position = Float4::new_point(world_x, world_y, wall_z);
            let ray = Ray {
                origin: ray_origin,
                direction: (position - ray_origin).normalise(),
            };

            if let Some(hit) = sphere.intersect(&ray).hit() {
                let point = ray.position(hit.distance());
                let normalv = hit.object().normal_at(point);
                let eyev = ray.direction.scalar_mul(-1.0);
                let colour = hit.object().lighting(light, point, eyev, normalv, false);
                let mut canvas = canvas_mutex.lock().unwrap();
                canvas.write_pixel(x, y, colour);
            }
        });

    canvas_mutex
        .lock()
        .unwrap()
        .to_file(Path::new("images/sphere_silhouette_3d.ppm"))
        .unwrap();
}
