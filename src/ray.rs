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
    inside: bool,
    ray: Ray,
    object: Object,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            distance: 0.0,
            point: Float4::origin(),
            eyev: Float4::new_vector(0.0, 0.0, 0.0),
            normalv: Float4::new_vector(0.0, 0.0, 0.0),
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

        Self {
            distance,
            point,
            eyev,
            normalv,
            inside,
            ray: *ray,
            object: object.clone(),
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
    pub fn inside(&self) -> bool {
        self.inside
    }
    pub fn object(&self) -> &Object {
        &self.object
    }
    pub fn over_point(&self) -> Float4 {
        self.point + self.normalv.scalar_mul(EPSILON)
    }
}

#[derive(Debug, Clone)]
pub struct Intersections(pub Vec<Intersection>);

impl Intersections {
    pub fn hit(&self) -> Option<Intersection> {
        let mut min_pos_distance = f64::MAX;
        let mut hi = None;

        for i in 0..self.0.len() {
            if self.0[i].distance > 0.0 && self.0[i].distance < min_pos_distance {
                min_pos_distance = self.0[i].distance;
                hi = Some(i);
            }
        }

        hi.map(|hi| self.0[hi].clone())
    }
}

mod test {
    use super::*;
    use crate::{canvas, colour::Colour, matrix::*, object::Material};
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
}
