extern crate nalgebra as na;

use crate::geometry::types::{Direction, Position};

#[derive(Debug)]
pub struct Ray {
    pub position: Position,
    pub direction: Direction,
    inv_direction: Direction,
    direction_sign: [usize; 3],
}

impl Ray {
    pub fn new(position: Position, direction: Direction) -> Ray {
        let i_d = Direction::new(1.0 / direction[0], 1.0 / direction[1], 1.0 / direction[2]);

        Ray {
            position: position,
            direction: direction,
            inv_direction: i_d,
            direction_sign: [
                (i_d[0] < 0.0) as usize,
                (i_d[1] < 0.0) as usize,
                (i_d[2] < 0.0) as usize,
            ],
        }
    }

    pub fn intersect_triangle(
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

    fn min_max_intersection(&self, bounds: &[Position; 2], i: usize) -> (f64, f64) {
        return (
            (bounds[self.direction_sign[i]][i] - self.position[i]) * self.inv_direction[i],
            (bounds[1 - self.direction_sign[i]][i] - self.position[i]) * self.inv_direction[i],
        );
    }

    /// Perform intersection testing with box as per
    /// An efficient and robust ray-box intersection algorithm - Williams & All
    /// http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.64.7663
    pub fn intersect_box(&self, bounds: &[Position; 2]) -> bool {
        let (mut tmin, mut tmax) = self.min_max_intersection(bounds, 0);
        let (tymin, tymax) = self.min_max_intersection(bounds, 1);

        if (tmin > tymax) || (tymin > tmax) {
            return false;
        };
        if tymin > tmin {
            tmin = tymin
        };
        if tymax < tmax {
            tmax = tymax
        };
        let (tzmin, tzmax) = self.min_max_intersection(bounds, 2);

        if (tmin > tzmax) || (tzmin > tmax) {
            return false;
        };

        true
    }
}
