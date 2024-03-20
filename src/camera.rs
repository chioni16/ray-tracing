use std::sync::Mutex;

use crate::{
    canvas::Canvas, colour::Colour, float4::Float4, matrix::Matrix, ray::Ray, world::World,
};

use itertools::Itertools;
use rayon::prelude::*;

pub struct Camera {
    hsize: usize,
    vsize: usize,
    half_width: f64,
    half_height: f64,
    field_of_view: f64,
    pixel_size: f64,
    transform: Matrix,
}

impl Camera {
    pub fn new(hsize: usize, vsize: usize, field_of_view: f64, transform: Matrix) -> Self {
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize as f64 / vsize as f64;

        let (half_width, half_height) = if aspect >= 1.0 {
            (half_view, half_view / aspect)
        } else {
            (half_view * aspect, half_view)
        };

        let pixel_size = (2.0 * half_width) / hsize as f64;

        Self {
            hsize,
            vsize,
            half_width,
            half_height,
            field_of_view,
            pixel_size,
            transform,
        }
    }

    pub fn ray_for_pixel(&self, px: usize, py: usize) -> Ray {
        let xoffset = (px as f64 + 0.5) * self.pixel_size;
        let yoffset = (py as f64 + 0.5) * self.pixel_size;

        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        let pixel: Float4 =
            self.transform.inverse().unwrap() * Float4::new_point(world_x, world_y, -1.0);

        let origin: Float4 = self.transform.inverse().unwrap() * Float4::origin();
        let direction = (pixel - origin).normalise();

        Ray { origin, direction }
    }

    pub fn render(&self, world: World) -> Canvas {
        use indicatif::ProgressBar;
        let progress = ProgressBar::new((self.hsize * self.vsize) as u64);

        let image_mutex = Mutex::new(Canvas::new(self.hsize, self.vsize, Colour::white()));

        (0..self.vsize)
            .cartesian_product(0..self.hsize)
            .par_bridge()
            .for_each(|(y, x)| {
                let ray = self.ray_for_pixel(x, y);
                let colour = world.colour_at(&ray);
                let mut image = image_mutex.lock().unwrap();
                image.write_pixel(x, y, colour);

                progress.inc(1);
            });

        progress.finish();

        image_mutex.into_inner().unwrap()
    }
}

mod test {
    use std::f64::consts::PI;

    use crate::{
        camera::Camera,
        colour::Colour,
        float4::Float4,
        matrix::{rotate_y, translate, view_transform, Matrix},
        util::float_is_eq,
        world::World,
    };

    #[test]
    fn pixel_size() {
        assert!(float_is_eq(
            Camera::new(200, 125, PI / 2.0, Matrix::identity(4)).pixel_size,
            0.01
        ));
        assert!(float_is_eq(
            Camera::new(125, 200, PI / 2.0, Matrix::identity(4)).pixel_size,
            0.01
        ));
    }

    #[test]
    fn ray_for_pixel() {
        let c1 = Camera::new(201, 101, PI / 2.0, Matrix::identity(4));
        let r1 = c1.ray_for_pixel(100, 50);
        assert_eq!(r1.origin, Float4::origin());
        assert_eq!(r1.direction, Float4::new_vector(0.0, 0.0, -1.0));

        let c2 = Camera::new(201, 101, PI / 2.0, Matrix::identity(4));
        let r2 = c2.ray_for_pixel(0, 0);
        assert_eq!(r2.origin, Float4::origin());
        assert_eq!(r2.direction, Float4::new_vector(0.66519, 0.33259, -0.66851));

        let c3 = Camera::new(
            201,
            101,
            PI / 2.0,
            rotate_y(PI / 4.0) * translate(0.0, -2.0, 5.0),
        );
        let r3 = c3.ray_for_pixel(100, 50);
        assert_eq!(r3.origin, Float4::new_point(0.0, 2.0, -5.0));
        assert_eq!(
            r3.direction,
            Float4::new_vector(1.0 / 2_f64.sqrt(), 0.0, -1.0 / 2_f64.sqrt())
        );
    }

    #[test]
    fn render() {
        let w = World::default();
        let transform = view_transform(
            Float4::new_point(0.0, 0.0, -5.0),
            Float4::origin(),
            Float4::new_vector(0.0, 1.0, 0.0),
        );
        let c = Camera::new(11, 11, PI / 2.0, transform);
        let i = c.render(w);
        assert_eq!(i.pixels[5][5], Colour::new(0.38066, 0.47583, 0.2855));
    }
}
