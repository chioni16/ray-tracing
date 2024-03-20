use crate::{
    colour::Colour,
    float4::Float4,
    matrix::Matrix,
    ray::{Intersection, Intersections, Ray},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Sphere(Matrix, Material),
}

impl Object {
    pub fn transform(&self) -> &Matrix {
        match self {
            Self::Sphere(matrix, ..) => matrix,
        }
    }

    pub fn material(&self) -> &Material {
        match self {
            Self::Sphere(_, material) => material,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Intersections {
        let object_space_ray = ray.transform(self.transform().inverse().unwrap());

        let sphere_to_ray = object_space_ray.origin - Float4::origin();

        let a = object_space_ray.direction.dot(object_space_ray.direction);
        let b = 2.0 * object_space_ray.direction.dot(sphere_to_ray);
        let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;

        let discriminant = b.powi(2) - 4.0 * a * c;

        if discriminant < 0.0 {
            Intersections(vec![])
        } else {
            let t1 = Intersection::new(ray, self, (-b - discriminant.sqrt()) / (2.0 * a));
            let t2 = Intersection::new(ray, self, (-b + discriminant.sqrt()) / (2.0 * a));
            Intersections(vec![t1, t2])
        }
    }

    pub fn normal_at(&self, world_point: Float4) -> Float4 {
        match self {
            Self::Sphere(matrix, ..) => {
                let object_point = matrix.inverse().unwrap() * world_point;
                let object_normal = object_point - Float4::origin();
                let mut world_normal = matrix.inverse().unwrap().transpose() * object_normal;
                world_normal.0[3] = 0.0;
                world_normal.normalise()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Float4,
    pub intensity: Colour,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    pub colour: Colour,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
}

impl Material {
    pub fn lighting(
        &self,
        light: PointLight,
        point: Float4,
        eyev: Float4,
        normalv: Float4,
        in_shadow: bool,
    ) -> Colour {
        let effective_colour = self.colour * light.intensity;
        let ambient = effective_colour.scalar_product(self.ambient);

        if in_shadow {
            return ambient;
        }

        let lightv = (light.position - point).normalise();
        let light_dot_normal = lightv.dot(normalv);
        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Colour::black(), Colour::black())
        } else {
            let diffuse = effective_colour.scalar_product(self.diffuse * light_dot_normal);

            let reflectv = (-lightv).reflect(normalv);
            let reflect_dot_eye = reflectv.dot(eyev);
            let specular = if reflect_dot_eye <= 0.0 {
                Colour::black()
            } else {
                let factor = reflect_dot_eye.powf(self.shininess);
                light.intensity.scalar_product(self.specular * factor)
            };

            (diffuse, specular)
        };

        ambient + diffuse + specular
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            colour: Colour::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
        }
    }
}

mod test {
    use super::*;
    use crate::canvas;
    use crate::ray::*;
    use std::path::Path;

    #[test]
    fn material_lighting() {
        let material = Material::default();
        let position = Float4::origin();

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, false),
            Colour::new(1.9, 1.9, 1.9)
        );

        let eyev = Float4::new_vector(0.0, 1.0 / 2_f64.sqrt(), -1.0 / 2_f64.sqrt());
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, false),
            Colour::new(1.0, 1.0, 1.0)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 10.0, -10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, false),
            Colour::new(0.7364, 0.7364, 0.7364)
        );

        let eyev = Float4::new_vector(0.0, -1.0 / 2_f64.sqrt(), -1.0 / 2_f64.sqrt());
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 10.0, -10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, false),
            Colour::new(1.6364, 1.6364, 1.6364)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, 10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, false),
            Colour::new(0.1, 0.1, 0.1)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            intensity: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            material.lighting(light, position, eyev, normalv, true),
            Colour::new(0.1, 0.1, 0.1)
        );
    }
}
