use crate::{
    colour::Colour,
    float4::Float4,
    matrix::{scale, Matrix},
    object::{Material, Object, PointLight, Shape},
    ray::{Intersection, Intersections, Ray},
    util::float_is_eq,
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
            .flat_map(|object| object.intersect(ray).into_inner())
            .collect::<Vec<_>>();
        is.sort_by(|a, b| a.distance().total_cmp(&b.distance()));
        Intersections::new(is)
    }

    pub fn shade_hit(&self, intersection: &Intersection, remaining: u8) -> Colour {
        let over_point = intersection.over_point();
        let surface = intersection.object().lighting(
            self.light,
            over_point,
            intersection.eyev(),
            intersection.normalv(),
            self.is_shadowed(over_point),
        );

        let reflected = self.reflected_colour(intersection, remaining);
        let refracted = self.refracted_colour(intersection, remaining);

        let material = intersection.object().material();
        if material.reflective > 0.0 && material.transparency > 0.0 {
            let reflectance = intersection.schlick();
            surface + reflected * reflectance + refracted * (1.0 - reflectance)
        } else {
            surface + reflected + refracted
        }
    }

    pub fn colour_at(&self, ray: &Ray, remaining: u8) -> Colour {
        self.intersect(ray)
            .hit()
            .map(|hit| self.shade_hit(&hit, remaining))
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

    pub fn reflected_colour(&self, intersection: &Intersection, remaining: u8) -> Colour {
        if remaining == 0 || float_is_eq(intersection.object().material().reflective, 0.0) {
            return Colour::black();
        }

        let reflect_ray = Ray {
            origin: intersection.over_point(),
            direction: intersection.reflectv(),
        };
        let colour = self.colour_at(&reflect_ray, remaining - 1);
        colour * intersection.object().material().reflective
    }

    pub fn refracted_colour(&self, intersection: &Intersection, remaining: u8) -> Colour {
        if remaining == 0 || float_is_eq(intersection.object().material().transparency, 0.0) {
            return Colour::black();
        }

        let n_ratio = intersection.n1() / intersection.n2();
        let cos_i = intersection.eyev().dot(intersection.normalv());
        let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));
        if sin2_t > 1.0 {
            return Colour::black();
        }

        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = intersection.normalv().scalar_mul(n_ratio * cos_i - cos_t)
            - intersection.eyev().scalar_mul(n_ratio);
        let refract_ray = Ray {
            origin: intersection.under_point(),
            direction,
        };

        self.colour_at(&refract_ray, remaining - 1) * intersection.object().material().transparency
    }
}

impl Default for World {
    fn default() -> Self {
        let light = PointLight {
            position: Float4::new_point(-10.0, 10.0, -10.0),
            colour: Colour::white(),
        };

        let s1 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ..Default::default()
            },
        };

        let s2 = Object {
            shape: Shape::Sphere,
            transform: scale(0.5, 0.5, 0.5),
            material: Material::default(),
        };

        Self {
            light,
            objects: vec![s1, s2],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        float4::Float4,
        matrix::translate,
        pattern::{Pattern, PatternKind},
        ray::Ray,
        util::float_is_eq,
        REF_RECURSION_LIMIT,
    };

    use super::*;

    #[test]
    fn intersect() {
        let w = World::default();
        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let is = w.intersect(&r);
        assert_eq!(is.count(), 4);
        assert!(float_is_eq(is.get_intersection_at(0).distance(), 4.0));
        assert!(float_is_eq(is.get_intersection_at(1).distance(), 4.5));
        assert!(float_is_eq(is.get_intersection_at(2).distance(), 5.5));
        assert!(float_is_eq(is.get_intersection_at(3).distance(), 6.0));
    }

    #[test]
    fn shade_hit() {
        let w1 = World::default();
        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i1 = Intersection::new(&r1, &w1.objects[0], 4.0);
        assert_eq!(
            w1.shade_hit(&i1, REF_RECURSION_LIMIT),
            Colour::new(0.38066, 0.47583, 0.2855)
        );

        let w2 = World {
            light: PointLight {
                position: Float4::new_point(0.0, 0.25, 0.0),
                colour: Colour::new(1.0, 1.0, 1.0),
            },
            ..Default::default()
        };
        let r2 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i2 = Intersection::new(&r2, &w2.objects[1], 0.5);
        assert_eq!(
            w2.shade_hit(&i2, REF_RECURSION_LIMIT),
            Colour::new(0.90498, 0.90498, 0.90498)
        );

        let s3_1 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let s3_2 = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 0.0, 10.0),
            material: Material::default(),
        };
        let w3 = World {
            light: PointLight {
                position: Float4::new_point(0.0, 0.0, -10.0),
                colour: Colour::new(1.0, 1.0, 1.0),
            },
            objects: vec![s3_1, s3_2.clone()],
        };
        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.0, 5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let i3 = Intersection::new(&r3, &s3_2, 4.0);
        assert_eq!(
            w3.shade_hit(&i3, REF_RECURSION_LIMIT),
            Colour::new(0.1, 0.1, 0.1)
        );

        let mut w4 = World::default();
        let plane = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Default::default()
            },
        };
        w4.objects.push(plane.clone());
        let r4 = Ray {
            origin: Float4::new_point(0.0, 0.0, -3.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let i4 = Intersection::new(&r4, &plane, 2f64.sqrt());
        assert_eq!(
            w4.shade_hit(&i4, REF_RECURSION_LIMIT),
            Colour::new(0.87675, 0.92434, 0.82917)
        );
    }

    #[test]
    fn colour_at() {
        let w1 = World::default();
        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        assert_eq!(w1.colour_at(&r1, REF_RECURSION_LIMIT), Colour::black());

        let w2 = World::default();
        let r2 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            w2.colour_at(&r2, REF_RECURSION_LIMIT),
            Colour::new(0.38066, 0.47583, 0.2855)
        );

        let s1 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material {
                colour: Colour::new(0.8, 1.0, 0.6),
                diffuse: 0.7,
                specular: 0.2,
                ambient: 1.0,
                ..Default::default()
            },
        };
        let s2 = Object {
            shape: Shape::Sphere,
            transform: scale(0.5, 0.5, 0.5),
            material: Material {
                ambient: 1.0,
                ..Default::default()
            },
        };
        let w3 = World {
            objects: vec![s1, s2],
            ..Default::default()
        };
        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.0, 0.75),
            direction: Float4::new_vector(0.0, 0.0, -1.0),
        };
        assert_eq!(
            w3.colour_at(&r3, REF_RECURSION_LIMIT),
            Colour::new(1.0, 1.0, 1.0)
        );
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

    #[test]
    fn reflected_colour() {
        let w1 = World::default();
        // w.objects[1].material.ambient = 1.0;
        let r1 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let mut s1 = w1.objects[1].clone();
        s1.material.ambient = 1.0;
        let i1 = Intersection::new(&r1, &s1, 1.0);
        assert_eq!(
            w1.reflected_colour(&i1, REF_RECURSION_LIMIT),
            Colour::black()
        );

        let mut w2 = World::default();
        let plane = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Default::default()
            },
        };
        w2.objects.push(plane.clone());
        let r2 = Ray {
            origin: Float4::new_point(0.0, 0.0, -3.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let i2 = Intersection::new(&r2, &plane, 2f64.sqrt());
        assert_eq!(
            w2.reflected_colour(&i2, REF_RECURSION_LIMIT),
            Colour::new(0.19033, 0.23791, 0.14274)
        );
    }

    #[test]
    fn reflection_recursion() {
        let mut w1 = World::default();
        let plane = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                ..Default::default()
            },
        };
        w1.objects.push(plane.clone());

        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -3.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let i1 = Intersection::new(&r1, &plane, 2f64.sqrt());
        assert_eq!(w1.reflected_colour(&i1, 0), Colour::black());

        let mut w2 = World {
            light: PointLight {
                position: Float4::origin(),
                colour: Colour::white(),
            },
            ..Default::default()
        };
        let lower = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                reflective: 1.0,
                ..Default::default()
            },
        };
        let upper = Object {
            shape: Shape::Plane,
            transform: translate(0.0, 1.0, 0.0),
            material: Material {
                reflective: 1.0,
                ..Default::default()
            },
        };
        w2.objects.extend(vec![lower, upper]);
        let r2 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        w2.colour_at(&r2, REF_RECURSION_LIMIT);
    }

    #[test]
    fn refracted_colour() {
        let w1 = World::default();
        let s1 = &w1.objects[0];
        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let is1 = Intersections::new(vec![
            Intersection::new(&r1, &s1, 4.0),
            Intersection::new(&r1, &s1, 6.0),
        ]);
        assert_eq!(
            w1.refracted_colour(is1.get_intersection_at(0), REF_RECURSION_LIMIT),
            Colour::black()
        );

        let mut w2 = World::default();
        w2.objects[0].material.transparency = 1.0;
        w2.objects[0].material.refractive_index = 1.5;
        let r2 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let is2 = Intersections::new(vec![
            Intersection::new(&r2, &w2.objects[0], 4.0),
            Intersection::new(&r2, &w2.objects[0], 6.0),
        ]);
        assert_eq!(
            w2.refracted_colour(is2.get_intersection_at(0), 0),
            Colour::black()
        );

        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.0, 1.0 / 2f64.sqrt()),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        let is3 = Intersections::new(vec![
            Intersection::new(&r3, &w2.objects[0], -1.0 / 2f64.sqrt()),
            Intersection::new(&r3, &w2.objects[0], 1.0 / 2f64.sqrt()),
        ]);
        assert_eq!(
            w2.refracted_colour(is3.get_intersection_at(1), REF_RECURSION_LIMIT),
            Colour::black()
        );

        let mut w4 = World::default();
        w4.objects[0].material.ambient = 1.0;
        w4.objects[0].material.pattern = Some(Pattern {
            transform: Matrix::identity(4),
            kind: PatternKind::TestLocation,
        });
        w4.objects[1].material.transparency = 1.0;
        w4.objects[1].material.refractive_index = 1.5;
        let r4 = Ray {
            origin: Float4::new_point(0.0, 0.0, 0.1),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        let is4 = Intersections::new(vec![
            Intersection::new(&r4, &w4.objects[0], -0.9899),
            Intersection::new(&r4, &w4.objects[1], -0.4899),
            Intersection::new(&r4, &w4.objects[1], 0.4899),
            Intersection::new(&r4, &w4.objects[0], 0.9899),
        ]);
        assert_eq!(
            w4.refracted_colour(is4.get_intersection_at(2), REF_RECURSION_LIMIT),
            Colour::new(0.0, 0.998874, 0.047218)
        );

        let mut w5 = World::default();
        let floor = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
        };
        w5.objects.push(floor.clone());
        let ball = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, -3.5, -0.5),
            material: Material {
                colour: Colour::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
        };
        w5.objects.push(ball);
        let r5 = Ray {
            origin: Float4::new_point(0.0, 0.0, -3.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let is5 = Intersections::new(vec![Intersection::new(&r5, &floor, 2f64.sqrt())]);
        assert_eq!(
            w5.shade_hit(is5.get_intersection_at(0), REF_RECURSION_LIMIT),
            Colour::new(0.93642, 0.68642, 0.68642)
        );
    }

    #[test]
    fn schlick() {
        let mut w = World::default();
        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -3.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let floor = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -1.0, 0.0),
            material: Material {
                reflective: 0.5,
                transparency: 0.5,
                refractive_index: 1.5,
                ..Default::default()
            },
        };
        w.objects.push(floor.clone());
        let ball = Object {
            shape: Shape::Plane,
            transform: translate(0.0, -3.5, -0.5),
            material: Material {
                colour: Colour::new(1.0, 0.0, 0.0),
                ambient: 0.5,
                ..Default::default()
            },
        };
        w.objects.push(ball);
        let intersections = Intersections::new(vec![Intersection::new(&r, &floor, 2f64.sqrt())]);
        assert_eq!(
            w.shade_hit(intersections.get_intersection_at(0), REF_RECURSION_LIMIT),
            Colour::new(0.93391, 0.69643, 0.69243)
        );
    }
}
