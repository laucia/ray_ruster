extern crate nalgebra as na;

use crate::geometry::types::{Direction, Position};

#[derive(Debug)]
pub struct Ray {
    pub position: Position,
    pub direction: Direction,
}

impl Ray {
    pub fn new(position: [f64; 3], direction: [f64; 3]) -> Ray {
        Ray {
            position: Position::new(position[0], position[1], position[2]),
            direction: Direction::new(direction[0], direction[1], direction[2]),
        }
    }

    pub fn intersect(
        &self,
        t1: &Position,
        t2: &Position,
        t3: &Position,
    ) -> Option<(Position, [f64; 2])> {
        let u = *t2 - *t1;
        let v = *t3 - *t1;

        let p = self.direction.cross(&v);
        let determinant = u.dot(&p);

        // Triangle normal and direction are parallel
        // or if negative triangle is backfacing
        if determinant < na::zero() {
            return None;
        }
        let inv_determinant = 1.0 / determinant;

        let w = self.position - *t1;
        let dist_u = w.dot(&p) * inv_determinant;
        if dist_u < na::zero() || dist_u > 1.0 {
            return None;
        }

        let q = w.cross(&u);

        let dist_v = self.direction.dot(&q) * inv_determinant;
        if dist_v < na::zero() || dist_u + dist_v > 1.0 {
            return None;
        }

        let dist_w = v.dot(&q) * inv_determinant;
        if dist_w < na::zero() {
            return None;
        }

        return Some((self.position + dist_w * self.direction, [dist_u, dist_v]));
    }
}
