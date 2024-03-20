use crate::{
    colour::Colour,
    float4::Float4,
    matrix::{scale, Matrix},
    object::{Material, Object, PointLight},
    ray::{Intersection, Intersections, Ray},
};

pub struct World {
    pub light: PointLight,
    pub objects: Vec<Object>,
}

impl World {
    pub fn intersect(&self, ray: &Ray) -> Intersections {
        let mut is = self
            .objects
            .iter()
            .flat_map(|object| object.intersect(ray).0)
            .collect::<Vec<_>>();
        is.sort_by(|a, b| a.distance().total_cmp(&b.distance()));
        Intersections(is)
    }

    pub fn shade_hit(&self, intersection: &Intersection) -> Colour {
        let over_point = intersection.over_point();
        intersection.object().material().lighting(
            self.light,
            over_point,
            intersection.eyev(),
            intersection.normalv(),
            self.is_shadowed(over_point),
        )
    }

    pub fn colour_at(&self, ray: &Ray) -> Colour {
        self.intersect(ray)
            .hit()
            .map(|hit| self.shade_hit(&hit))
            .unwrap_or(Colour::black())
    }

    pub fn is_shadowed(&self, point: Float4) -> bool {
        let v = self.light.position - point;
        let distance = v.mag();
        let direction = v.normalise();

        let shadow_ray = Ray {
            origin: point,
            direction,
        };
        let intersections = self.intersect(&shadow_ray);

        matches!(intersections.hit(), Some(hit) if hit.distance() < distance)
    }
}

impl Default for World {
    fn default() -> Self {
        let light = PointLight {
            position: Float4::new_point(-10.0, 10.0, -10.0),
            intensity: Colour::white(),
        };

        let s1 = Object::Sphere(
            Matrix::identity(4),
            Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ..Default::default()
            },
        );

        let s2 = Object::Sphere(scale(0.5, 0.5, 0.5), Material::default());

        Self {
            light,
            objects: vec![s1, s2],
        }
    }
}

mod test {
    use crate::{float4::Float4, matrix::translate, ray::Ray, util::float_is_eq};

    use super::*;

    #[test]
    fn intersect() {
        let w = World::default();
        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let is = w.intersect(&r);
        assert_eq!(is.0.len(), 4);
        assert!(float_is_eq(is.0[0].distance(), 4.0));
        assert!(float_is_eq(is.0[1].distance(), 4.5));
        assert!(float_is_eq(is.0[2].distance(), 5.5));
        assert!(float_is_eq(is.0[3].distance(), 6.0));
    }

    #[test]
    fn shade_hit() {
        let w1 = World::default();
        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i1 = Intersection::new(&r1, &w1.objects[0], 4.0);
        assert_eq!(w1.shade_hit(&i1), Colour::new(0.38066, 0.47583, 0.2855));

        let w2 = World {
            light: PointLight {
                position: Float4::new_point(0.0, 0.25, 0.0),
                intensity: Colour::new(1.0, 1.0, 1.0),
            },
            ..Default::default()
        };
        let r2 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i2 = Intersection::new(&r2, &w2.objects[1], 0.5);
        assert_eq!(w2.shade_hit(&i2), Colour::new(0.90498, 0.90498, 0.90498));

        let s3_1 = Object::Sphere(Matrix::identity(4), Material::default());
        let s3_2 = Object::Sphere(translate(0.0, 0.0, 10.0), Material::default());
        let w3 = World {
            light: PointLight {
                position: Float4::new_point(0.0, 0.0, -10.0),
                intensity: Colour::new(1.0, 1.0, 1.0),
            },
            objects: vec![s3_1, s3_2.clone()],
        };
        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.0, 5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i3 = Intersection::new(&r3, &s3_2, 4.0);
        assert_eq!(w3.shade_hit(&i3), Colour::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn colour_at() {
        let w1 = World::default();
        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        assert_eq!(w1.colour_at(&r1), Colour::black());

        let w2 = World::default();
        let r2 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(w2.colour_at(&r2), Colour::new(0.38066, 0.47583, 0.2855));

        let s1 = Object::Sphere(
            Matrix::identity(4),
            Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ambient: 1.0,
                ..Default::default()
            },
        );
        let s2 = Object::Sphere(
            scale(0.5, 0.5, 0.5),
            Material {
                ambient: 1.0,
                ..Default::default()
            },
        );
        let w3 = World {
            objects: vec![s1, s2],
            ..Default::default()
        };
        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.0, 0.75),
            direction: Float4::new_vector(0.0, 0.0, -1.0),
        };
        assert_eq!(w3.colour_at(&r3), Colour::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn is_shadowed() {
        let w1 = World::default();
        let p1 = Float4::new_point(0.0, 10.0, 0.0);
        assert!(!w1.is_shadowed(p1));

        let w2 = World::default();
        let p2 = Float4::new_point(10.0, -10.0, 10.0);
        assert!(w2.is_shadowed(p2));

        let w3 = World::default();
        let p3 = Float4::new_point(-20.0, 20.0, -20.0);
        assert!(!w3.is_shadowed(p3));

        let w4 = World::default();
        let p4 = Float4::new_point(-2.0, 2.0, -2.0);
        assert!(!w4.is_shadowed(p4));
    }
}
