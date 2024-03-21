use crate::{
    float4::Float4,
    matrix::Matrix,
    object::{Object, Shape},
    util::EPSILON,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Float4,
    pub direction: Float4,
}

impl Ray {
    pub fn position(&self, t: f64) -> Float4 {
        self.origin + self.direction.scalar_mul(t)
    }

    pub fn transform(&self, matrix: Matrix) -> Self {
        let new_origin = matrix.multiply(&self.origin.into());
        let new_direction = matrix.multiply(&self.direction.into());
        Self {
            origin: new_origin.into(),
            direction: new_direction.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Intersection {
    distance: f64,
    point: Float4,
    eyev: Float4,
    normalv: Float4,
    reflectv: Float4,
    inside: bool,
    ray: Ray,
    object: Object,

    n1: Option<f64>,
    n2: Option<f64>,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            distance: 0.0,
            point: Float4::origin(),
            eyev: Float4::new_vector(0.0, 0.0, 0.0),
            normalv: Float4::new_vector(0.0, 0.0, 0.0),
            reflectv: Float4::new_vector(0.0, 0.0, 0.0),
            inside: false,
            ray: Ray {
                origin: Float4::origin(),
                direction: Float4::new_vector(0.0, 0.0, 0.0),
            },
            object: Object {
                shape: Shape::Sphere,
                transform: Matrix::identity(4),
                material: Default::default(),
            },
            n1: None,
            n2: None,
        }
    }
}

impl Intersection {
    pub fn new(ray: &Ray, object: &Object, distance: f64) -> Self {
        let point = ray.position(distance);
        let eyev = -ray.direction;
        let mut normalv = object.normal_at(point);
        let inside = normalv.dot(eyev) < 0.0;
        if inside {
            normalv = -normalv;
        }
        let reflectv = ray.direction.reflect(normalv);

        Self {
            distance,
            point,
            eyev,
            normalv,
            reflectv,
            inside,
            ray: *ray,
            object: object.clone(),
            n1: None,
            n2: None,
        }
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }
    fn point(&self) -> Float4 {
        self.point
    }
    pub fn eyev(&self) -> Float4 {
        self.eyev
    }
    pub fn normalv(&self) -> Float4 {
        self.normalv
    }
    pub fn reflectv(&self) -> Float4 {
        self.reflectv
    }
    pub fn inside(&self) -> bool {
        self.inside
    }
    pub fn object(&self) -> &Object {
        &self.object
    }

    pub fn over_point(&self) -> Float4 {
        self.point + self.normalv.scalar_mul(EPSILON)
    }
    pub fn under_point(&self) -> Float4 {
        self.point - self.normalv.scalar_mul(EPSILON)
    }

    pub fn n1(&self) -> f64 {
        self.n1.unwrap()
    }
    pub fn n2(&self) -> f64 {
        self.n2.unwrap()
    }

    pub fn schlick(&self) -> f64 {
        let mut cos = self.eyev.dot(self.normalv);

        if self.n1() > self.n2() {
            let n = self.n1() / self.n2();
            let sin2_t = n.powi(2) * (1.0 - cos.powi(2));
            if sin2_t > 1.0 {
                return 1.0;
            }

            let cos_t = (1.0 - sin2_t).sqrt();
            cos = cos_t;
        }

        let r0 = ((self.n1() - self.n2()) / (self.n1() + self.n2())).powi(2);
        r0 + (1.0 - r0) * (1.0 - cos).powi(5)
    }
}

#[derive(Debug, Clone)]
pub struct Intersections(Vec<Intersection>);

impl Intersections {
    pub fn new(is: Vec<Intersection>) -> Self {
        let mut is = Self(is);
        is.compute_refractive_indices();
        is
    }

    fn hit_index(&self) -> Option<usize> {
        let mut min_pos_distance = f64::MAX;
        let mut hi = None;

        for i in 0..self.0.len() {
            if self.0[i].distance > 0.0 && self.0[i].distance < min_pos_distance {
                min_pos_distance = self.0[i].distance;
                hi = Some(i);
            }
        }

        hi
    }

    pub fn get_intersection_at(&self, index: usize) -> &Intersection {
        &self.0[index]
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> Vec<Intersection> {
        self.0
    }

    pub fn hit(&self) -> Option<Intersection> {
        let hi = self.hit_index();
        hi.map(|hi| self.0[hi].clone())
    }

    fn compute_refractive_indices(&mut self) {
        // let Some(hi) = self.hit_index() else {
        //     return;
        // };

        for hi in 0..self.count() {
            let mut containers: Vec<&Object> = vec![];
            for (i, ix) in self.0.iter_mut().enumerate() {
                if i == hi {
                    ix.n1 = Some(
                        containers
                            .last()
                            .map_or(1.0, |o| o.material.refractive_index),
                    );
                }

                let cur_obj = &ix.object;
                if let Some(pos) = containers.iter().position(|x| *x == cur_obj) {
                    containers.remove(pos);
                } else {
                    containers.push(cur_obj);
                }

                if i == hi {
                    ix.n2 = Some(
                        containers
                            .last()
                            .map_or(1.0, |o| o.material.refractive_index),
                    );
                    break;
                }
            }
        }
    }
}

mod test {
    use super::*;
    use crate::{canvas, colour::Colour, matrix::*, object::Material, util::float_is_eq};
    use std::f64::consts::PI;

    #[test]
    fn point_at_distance() {
        let origin = Float4::new_point(2.0, 3.0, 4.0);
        let direction = Float4::new_vector(1.0, 0.0, 0.0);
        let ray = Ray { origin, direction };

        assert_eq!(ray.position(0.0), origin);
        assert_eq!(ray.position(1.0), Float4::new_point(3.0, 3.0, 4.0));
        assert_eq!(ray.position(-1.0), Float4::new_point(1.0, 3.0, 4.0));
        assert_eq!(ray.position(2.5), Float4::new_point(4.5, 3.0, 4.0));
    }

    #[test]
    fn intersection_sphere() {
        let sphere1 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let ray = Ray {
            origin: Float4::new_point(0.0, 1.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere1
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![5.0, 5.0]
        );

        let ray = Ray {
            origin: Float4::new_point(0.0, 2.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere1
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![]
        );

        let ray = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere1
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![-1.0, 1.0]
        );

        let ray = Ray {
            origin: Float4::new_point(0.0, 0.0, 5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere1
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![-6.0, -4.0]
        );

        let sphere2 = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material::default(),
        };
        let ray = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere2
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![3.0, 7.0]
        );

        let sphere3 = Object {
            shape: Shape::Sphere,
            transform: translate(5.0, 0.0, 0.0),
            material: Material::default(),
        };
        let ray = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        assert_eq!(
            sphere3
                .intersect(&ray)
                .0
                .iter()
                .map(|i| i.distance)
                .collect::<Vec<_>>(),
            vec![]
        )
    }

    #[test]
    fn hit_sphere() {
        let i1 = Intersection {
            distance: 5.0,
            object: Object {
                shape: Shape::Sphere,
                transform: Matrix::identity(4),
                material: Material::default(),
            },
            ..Default::default()
        };
        let i2 = Intersection {
            distance: 7.0,
            object: Object {
                shape: Shape::Sphere,
                transform: Matrix::identity(4),
                material: Material::default(),
            },
            ..Default::default()
        };
        let i3 = Intersection {
            distance: -3.0,
            object: Object {
                shape: Shape::Sphere,
                transform: Matrix::identity(4),
                material: Material::default(),
            },
            ..Default::default()
        };
        let i4 = Intersection {
            distance: 2.0,
            object: Object {
                shape: Shape::Sphere,
                transform: Matrix::identity(4),
                material: Material::default(),
            },
            ..Default::default()
        };
        let intersections = Intersections(vec![i1.clone(), i2.clone(), i3.clone(), i4.clone()]);
        assert_eq!(intersections.hit(), Some(i4));
    }

    #[test]
    fn ray_transform() {
        let r = Ray {
            origin: Float4::new_point(1.0, 2.0, 3.0),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };

        let m1 = translate(3.0, 4.0, 5.0);
        let expected1 = Ray {
            origin: Float4::new_point(4.0, 6.0, 8.0),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        assert_eq!(r.transform(m1), expected1);

        let m2 = scale(2.0, 3.0, 4.0);
        let expected2 = Ray {
            origin: Float4::new_point(2.0, 6.0, 12.0),
            direction: Float4::new_vector(0.0, 3.0, 0.0),
        };
        assert_eq!(r.transform(m2), expected2);
    }

    #[test]
    fn normal_at_sphere() {
        let sphere1 = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 1.0, 0.0),
            material: Material::default(),
        };
        let normal = sphere1.normal_at(Float4::new_point(0.0, 1.70711, -0.70711));
        let expected = Float4::new_vector(0.0, 0.70711, -0.70711);
        assert_eq!(normal, expected);

        let sphere2 = Object {
            shape: Shape::Sphere,
            transform: scale(1.0, 0.5, 1.0) * rotate_z(PI / 5.0),
            material: Material::default(),
        };
        let normal2 = sphere2.normal_at(Float4::new_point(
            0.0,
            1.0 / 2.0_f64.sqrt(),
            -1.0 / 2.0_f64.sqrt(),
        ));
        let expected2 = Float4::new_vector(0.0, 0.97014, -0.24254);
        assert_eq!(normal2, expected2);
    }

    #[test]
    fn intersection_in_out() {
        let sphere1 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let ray1 = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let distance1 = 4.0;
        let intersection1 = Intersection::new(&ray1, &sphere1, distance1);
        assert!(!intersection1.inside);

        let sphere2 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let ray2 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };
        let distance2 = 1.0;
        let intersection2 = Intersection::new(&ray2, &sphere2, distance2);
        assert!(intersection2.inside);
        assert_eq!(intersection2.point, Float4::new_point(0.0, 0.0, 1.0));
        assert_eq!(intersection2.eyev, Float4::new_vector(0.0, 0.0, -1.0));
        assert_eq!(intersection2.normalv, Float4::new_vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn over_point() {
        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 0.1),
        };

        let s = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 0.0, 1.0),
            material: Material::default(),
        };
        let i = Intersection::new(&r, &s, 5.0);
        assert!(i.over_point().0[2] < -EPSILON / 2.0);
        assert!(i.point().0[2] > i.over_point().0[2]);
    }

    #[test]
    fn reflectv() {
        let o = Object {
            shape: Shape::Plane,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let r = Ray {
            origin: Float4::new_point(0.0, 1.0, -1.0),
            direction: Float4::new_vector(0.0, -1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt()),
        };
        let i = Intersection::new(&r, &o, 1.0 / 2f64.sqrt());
        assert_eq!(
            i.reflectv,
            Float4::new_vector(0.0, 1.0 / 2f64.sqrt(), 1.0 / 2f64.sqrt())
        );
    }

    #[test]
    fn refractive_index() {
        let a = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Default::default()
            },
        };
        let b = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 0.0, -0.25),
            material: Material {
                transparency: 1.0,
                refractive_index: 2.0,
                ..Default::default()
            },
        };
        let c = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 0.0, 0.25),
            material: Material {
                transparency: 1.0,
                refractive_index: 2.5,
                ..Default::default()
            },
        };

        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -4.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };

        let intersections = Intersections::new(vec![
            Intersection::new(&r, &a, 2.0),
            Intersection::new(&r, &b, 2.75),
            Intersection::new(&r, &c, 3.25),
            Intersection::new(&r, &b, 4.75),
            Intersection::new(&r, &c, 5.25),
            Intersection::new(&r, &a, 6.0),
        ]);

        assert_eq!(intersections.get_intersection_at(0).n1, Some(1.0));
        assert_eq!(intersections.get_intersection_at(0).n2, Some(1.5));
        assert_eq!(intersections.get_intersection_at(1).n1, Some(1.5));
        assert_eq!(intersections.get_intersection_at(1).n2, Some(2.0));
        assert_eq!(intersections.get_intersection_at(2).n1, Some(2.0));
        assert_eq!(intersections.get_intersection_at(2).n2, Some(2.5));
        assert_eq!(intersections.get_intersection_at(3).n1, Some(2.5));
        assert_eq!(intersections.get_intersection_at(3).n2, Some(2.5));
        assert_eq!(intersections.get_intersection_at(4).n1, Some(2.5));
        assert_eq!(intersections.get_intersection_at(4).n2, Some(1.5));
        assert_eq!(intersections.get_intersection_at(5).n1, Some(1.5));
        assert_eq!(intersections.get_intersection_at(5).n2, Some(1.0));
    }

    #[test]
    fn under_point() {
        let r = Ray {
            origin: Float4::new_point(0.0, 0.0, -5.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };

        let s = Object {
            shape: Shape::Sphere,
            transform: translate(0.0, 0.0, 1.0),
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Default::default()
            },
        };
        let i = Intersection::new(&r, &s, 5.0);

        assert!(i.under_point().0[2] > EPSILON / 2.0);
        assert!(i.point().0[2] < i.under_point().0[2]);
    }

    #[test]
    fn schlick() {
        let s = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material {
                transparency: 1.0,
                refractive_index: 1.5,
                ..Default::default()
            },
        };

        let r1 = Ray {
            origin: Float4::new_point(0.0, 0.0, 1.0 / 2f64.sqrt()),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };
        let intersections1 = Intersections::new(vec![
            Intersection::new(&r1, &s, -1.0 / 2f64.sqrt()),
            Intersection::new(&r1, &s, 1.0 / 2f64.sqrt()),
        ]);
        assert!(float_is_eq(
            intersections1.get_intersection_at(1).schlick(),
            1.0
        ));

        let r2 = Ray {
            origin: Float4::origin(),
            direction: Float4::new_vector(0.0, 1.0, 0.0),
        };

        let intersections2 = Intersections::new(vec![
            Intersection::new(&r2, &s, -1.0),
            Intersection::new(&r2, &s, 1.0),
        ]);
        assert!(float_is_eq(
            intersections2.get_intersection_at(1).schlick(),
            0.04
        ));

        let r3 = Ray {
            origin: Float4::new_point(0.0, 0.99, -2.0),
            direction: Float4::new_vector(0.0, 0.0, 1.0),
        };

        let intersections3 = Intersections::new(vec![Intersection::new(&r3, &s, 1.8589)]);
        assert!(float_is_eq(
            intersections3.get_intersection_at(0).schlick(),
            0.48873
        ));
    }
}
