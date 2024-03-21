use crate::{colour::Colour, float4::Float4, matrix::Matrix, object::Object, util::float_is_eq};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternKind {
    Stripe(Colour, Colour),
    Gradient(Colour, Colour),
    Ring(Colour, Colour),
    Checkers(Colour, Colour),
    TestLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub transform: Matrix,
}

impl Pattern {
    pub fn at(&self, point: Float4) -> Colour {
        match self.kind {
            PatternKind::Stripe(colour1, colour2) => {
                if float_is_eq(point.0[0].floor() % 2.0, 0.0) {
                    colour1
                } else {
                    colour2
                }
            }
            PatternKind::Gradient(colour1, colour2) => {
                let x = point.0[0];
                colour1 + (colour2 - colour1) * (x - x.floor())
            }
            PatternKind::Ring(colour1, colour2) => {
                if float_is_eq(
                    (point.0[0].powi(2) + point.0[2].powi(2)).sqrt().floor() % 2.0,
                    0.0,
                ) {
                    colour1
                } else {
                    colour2
                }
            }
            PatternKind::Checkers(colour1, colour2) => {
                if float_is_eq(
                    (point.0[0].floor() + point.0[1].floor() + point.0[2].floor()) % 2.0,
                    0.0,
                ) {
                    colour1
                } else {
                    colour2
                }
            }
            PatternKind::TestLocation => Colour::new(point.0[0], point.0[1], point.0[2]),
        }
    }

    pub fn at_object(&self, point: Float4, object: &Object) -> Colour {
        let object_point = object.transform().inverse().unwrap() * point;
        let pattern_point = self.transform.inverse().unwrap() * object_point;
        self.at(pattern_point)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        matrix::{scale, translate},
        object::{Material, Shape},
    };

    use super::*;

    #[test]
    fn stripe_at() {
        let s = Pattern {
            kind: PatternKind::Stripe(Colour::white(), Colour::black()),
            transform: Matrix::identity(4),
        };

        assert_eq!(s.at(Float4::origin()), Colour::white());

        assert_eq!(s.at(Float4::new_point(0.0, 1.0, 0.0)), Colour::white());
        assert_eq!(s.at(Float4::new_point(0.0, 2.0, 0.0)), Colour::white());

        assert_eq!(s.at(Float4::new_point(0.0, 0.0, 1.0)), Colour::white());
        assert_eq!(s.at(Float4::new_point(0.0, 0.0, 2.0)), Colour::white());

        assert_eq!(s.at(Float4::new_point(0.9, 0.0, 0.0)), Colour::white());
        assert_eq!(s.at(Float4::new_point(1.0, 0.0, 0.0)), Colour::black());
        assert_eq!(s.at(Float4::new_point(-0.1, 0.0, 0.0)), Colour::black());
        assert_eq!(s.at(Float4::new_point(-1.0, 0.0, 0.0)), Colour::black());
        assert_eq!(s.at(Float4::new_point(-1.1, 0.0, 0.0)), Colour::white());
    }

    #[test]
    fn stripe_at_object() {
        let s1 = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material::default(),
        };
        let p1 = Pattern {
            kind: PatternKind::Stripe(Colour::white(), Colour::black()),
            transform: Matrix::identity(4),
        };
        assert_eq!(
            p1.at_object(Float4::new_point(1.5, 0.0, 0.0), &s1),
            Colour::white()
        );

        let s2 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let p2 = Pattern {
            kind: PatternKind::Stripe(Colour::white(), Colour::black()),
            transform: scale(2.0, 2.0, 2.0),
        };
        assert_eq!(
            p2.at_object(Float4::new_point(1.5, 0.0, 0.0), &s2),
            Colour::white()
        );

        let s3 = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material::default(),
        };
        let p3 = Pattern {
            kind: PatternKind::Stripe(Colour::white(), Colour::black()),
            transform: scale(0.5, 0.5, 0.5),
        };
        assert_eq!(
            p3.at_object(Float4::new_point(2.5, 0.0, 0.0), &s3),
            Colour::white()
        );

        let s4 = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material::default(),
        };
        let p4 = Pattern {
            kind: PatternKind::TestLocation,
            transform: Matrix::identity(4),
        };
        assert_eq!(
            p4.at_object(Float4::new_point(2.0, 3.0, 4.0), &s4),
            Colour::new(1.0, 1.5, 2.0)
        );

        let s5 = Object {
            shape: Shape::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
        };
        let p5 = Pattern {
            kind: PatternKind::TestLocation,
            transform: scale(2.0, 2.0, 2.0),
        };
        assert_eq!(
            p5.at_object(Float4::new_point(2.0, 3.0, 4.0), &s5),
            Colour::new(1.0, 1.5, 2.0)
        );

        let s6 = Object {
            shape: Shape::Sphere,
            transform: scale(2.0, 2.0, 2.0),
            material: Material::default(),
        };
        let p6 = Pattern {
            kind: PatternKind::TestLocation,
            transform: translate(0.5, 1.0, 1.5),
        };
        assert_eq!(
            p6.at_object(Float4::new_point(2.5, 3.0, 3.5), &s6),
            Colour::new(0.75, 0.5, 0.25)
        );
    }

    #[test]
    fn gradient() {
        let p = Pattern {
            kind: PatternKind::Gradient(Colour::white(), Colour::black()),
            transform: Matrix::identity(4),
        };
        assert_eq!(p.at(Float4::origin()), Colour::white());
        assert_eq!(
            p.at(Float4::new_point(0.25, 0.0, 0.0)),
            Colour::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            p.at(Float4::new_point(0.5, 0.0, 0.0)),
            Colour::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            p.at(Float4::new_point(0.75, 0.0, 0.0)),
            Colour::new(0.25, 0.25, 0.25)
        );
    }

    #[test]
    fn ring() {
        let p = Pattern {
            kind: PatternKind::Ring(Colour::white(), Colour::black()),
            transform: Matrix::identity(4),
        };
        assert_eq!(p.at(Float4::origin()), Colour::white());
        assert_eq!(p.at(Float4::new_point(1.0, 0.0, 0.0)), Colour::black());
        assert_eq!(p.at(Float4::new_point(0.0, 0.0, 1.0)), Colour::black());
        assert_eq!(p.at(Float4::new_point(0.708, 0.0, 0.708)), Colour::black());
    }

    #[test]
    fn checkers() {
        let p = Pattern {
            kind: PatternKind::Checkers(Colour::white(), Colour::black()),
            transform: Matrix::identity(4),
        };

        assert_eq!(p.at(Float4::origin()), Colour::white());

        assert_eq!(p.at(Float4::new_point(0.99, 0.0, 0.0)), Colour::white());
        assert_eq!(p.at(Float4::new_point(1.01, 0.0, 0.0)), Colour::black());

        assert_eq!(p.at(Float4::new_point(0.0, 0.99, 0.0)), Colour::white());
        assert_eq!(p.at(Float4::new_point(0.0, 1.01, 0.0)), Colour::black());

        assert_eq!(p.at(Float4::new_point(0.0, 0.0, 0.99)), Colour::white());
        assert_eq!(p.at(Float4::new_point(0.0, 0.0, 1.01)), Colour::black());
    }
}
