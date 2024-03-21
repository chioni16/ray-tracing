use crate::{
    colour::Colour,
    float4::Float4,
    matrix::Matrix,
    pattern::Pattern,
    ray::{Intersection, Intersections, Ray},
    util::EPSILON,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Sphere,
    Plane,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub shape: Shape,
    pub transform: Matrix,
    pub material: Material,
}

impl Object {
    pub fn transform(&self) -> &Matrix {
        &self.transform
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn intersect(&self, ray: &Ray) -> Intersections {
        let object_space_ray = ray.transform(self.transform().inverse().unwrap());
        let distances = match self.shape {
            Shape::Sphere => {
                let sphere_to_ray = object_space_ray.origin - Float4::origin();

                let a = object_space_ray.direction.dot(object_space_ray.direction);
                let b = 2.0 * object_space_ray.direction.dot(sphere_to_ray);
                let c = sphere_to_ray.dot(sphere_to_ray) - 1.0;

                let discriminant = b.powi(2) - 4.0 * a * c;

                if discriminant < 0.0 {
                    vec![]
                } else {
                    let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                    let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                    vec![t1, t2]
                }
            }
            Shape::Plane => {
                if object_space_ray.direction.0[1].abs() < EPSILON {
                    vec![]
                } else {
                    let t = -object_space_ray.origin.0[1] / object_space_ray.direction.0[1];
                    vec![t]
                }
            }
        };

        Intersections::new(
            distances
                .iter()
                .map(|distance| Intersection::new(ray, self, *distance))
                .collect(),
        )
    }

    pub fn normal_at(&self, world_point: Float4) -> Float4 {
        let matrix = self.transform();
        let object_point = matrix.inverse().unwrap() * world_point;

        let object_normal = match self.shape {
            Shape::Sphere => object_point - Float4::origin(),
            Shape::Plane => Float4::new_vector(0.0, 1.0, 0.0),
        };

        let mut world_normal = matrix.inverse().unwrap().transpose() * object_normal;
        world_normal.0[3] = 0.0;
        world_normal.normalise()
    }

    pub fn lighting(
        &self,
        light: PointLight,
        point: Float4,
        eyev: Float4,
        normalv: Float4,
        in_shadow: bool,
    ) -> Colour {
        let colour = self
            .material()
            .pattern
            .as_ref()
            .map_or(self.material.colour, |pattern| {
                pattern.at_object(point, self)
            });

        let effective_colour = colour * light.colour;
        let ambient = effective_colour.scalar_product(self.material.ambient);

        if in_shadow {
            return ambient;
        }

        let lightv = (light.position - point).normalise();
        let light_dot_normal = lightv.dot(normalv);
        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Colour::black(), Colour::black())
        } else {
            let diffuse = effective_colour.scalar_product(self.material.diffuse * light_dot_normal);

            let reflectv = (-lightv).reflect(normalv);
            let reflect_dot_eye = reflectv.dot(eyev);
            let specular = if reflect_dot_eye <= 0.0 {
                Colour::black()
            } else {
                let factor = reflect_dot_eye.powf(self.material.shininess);
                light.colour.scalar_product(self.material.specular * factor)
            };

            (diffuse, specular)
        };

        ambient + diffuse + specular
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Float4,
    pub colour: Colour,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub colour: Colour,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
    pub pattern: Option<Pattern>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            colour: Colour::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflective: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
            pattern: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pattern::PatternKind;

    #[test]
    fn material_lighting() {
        let s = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let position = Float4::origin();

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, false),
            Colour::new(1.9, 1.9, 1.9)
        );

        let eyev = Float4::new_vector(0.0, 1.0 / 2_f64.sqrt(), -1.0 / 2_f64.sqrt());
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, false),
            Colour::new(1.0, 1.0, 1.0)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 10.0, -10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, false),
            Colour::new(0.7364, 0.7364, 0.7364)
        );

        let eyev = Float4::new_vector(0.0, -1.0 / 2_f64.sqrt(), -1.0 / 2_f64.sqrt());
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 10.0, -10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, false),
            Colour::new(1.6364, 1.6364, 1.6364)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, 10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, false),
            Colour::new(0.1, 0.1, 0.1)
        );

        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            colour: Colour::new(1.0, 1.0, 1.0),
        };
        assert_eq!(
            s.lighting(light, position, eyev, normalv, true),
            Colour::new(0.1, 0.1, 0.1)
        );
    }

    #[test]
    fn material_lighting_with_pattern() {
        let s = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material {
                ambient: 1.0,
                diffuse: 0.0,
                specular: 0.0,
                pattern: Some(Pattern {
                    kind: PatternKind::Stripe(Colour::white(), Colour::black()),
                    transform: Matrix::identity(4),
                }),
                ..Default::default()
            },
        };
        let eyev = Float4::new_vector(0.0, 0.0, -1.0);
        let normalv = Float4::new_vector(0.0, 0.0, -1.0);
        let light = PointLight {
            position: Float4::new_point(0.0, 0.0, -10.0),
            colour: Colour::white(),
        };

        assert_eq!(
            s.lighting(
                light,
                Float4::new_point(0.9, 0.0, 0.0),
                eyev,
                normalv,
                false
            ),
            Colour::white()
        );
        assert_eq!(
            s.lighting(
                light,
                Float4::new_point(1.1, 0.0, 0.0),
                eyev,
                normalv,
                false
            ),
            Colour::black()
        );
    }
}
