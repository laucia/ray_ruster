extern crate nalgebra;
use crate::geometry::types::{Direction, Position};

pub struct AxisAlignedBoundingBox {
    pub bounds: [Position; 2],
    pub dim: Position,
    pub center: Position,
    pub extent: Direction,
}

impl AxisAlignedBoundingBox {
    pub fn new(vertices: &Vec<Position>) -> Self {
        let min = vertices
            .iter()
            .fold(vertices[0], |min, vertice| min.inf(vertice));
        let max = vertices
            .iter()
            .fold(vertices[0], |max, vertice| max.sup(vertice));

        Self::from_bounds([min, max])
    }

    pub fn from_bounds(bounds: [Position; 2]) -> Self {
        let c = nalgebra::center(&bounds[0], &bounds[1]);
        AxisAlignedBoundingBox {
            bounds: bounds,
            dim: Position::from(bounds[1] - bounds[0]),
            center: c,
            extent: Direction::from(bounds[1] - c),
        }
    }

    pub fn get_dimension(&self, i: usize) -> f64 {
        return self.dim[i];
    }

    pub fn width(&self) -> f64 {
        return self.dim[0];
    }

    pub fn height(&self) -> f64 {
        return self.dim[1];
    }

    pub fn length(&self) -> f64 {
        return self.dim[2];
    }

    pub fn largest_dim(&self) -> usize {
        if self.width() > self.length() && self.width() > self.height() {
            return 0;
        }
        if self.height() > self.length() {
            return 1;
        }
        2
    }

    pub fn split(&self, dim: usize, at: f64) -> Option<(Self, Self)> {
        let min = self.bounds[0].clone();
        let max = self.bounds[1].clone();

        let mut at_min = min.clone();
        let mut at_max = max.clone();

        at_min[dim] = at;
        at_max[dim] = at;

        if (min[dim] > at) || (at > max[dim]) {
            return None;
        };

        Some((
            Self::from_bounds([min, at_max]),
            Self::from_bounds([at_min, max]),
        ))
    }

    fn projected_radius(&self, axis: &Direction) -> f64 {
        self.extent.dot(&axis.abs())
    }

    /// Is the given triangle intersecting the box
    ///
    /// # Principle
    /// This test is based on the SAT (Separating Axis Theorem)
    /// We are testing all the possible separating axis by projecting the geometry
    /// onto the orthogonal axis, and observing separation or not.
    /// If the 2 figures are disjointed on the axis, then we can exit, otherwise we
    /// continue. Once all possible axis have been tested, we know there is no separating axis
    /// and the 2 shapes are intersecting.
    ///
    /// # Reference
    /// * http://fileadmin.cs.lth.se/cs/Personal/Tomas_Akenine-Moller/code/tribox_tam.pdf
    /// * https://stackoverflow.com/questions/17458562/efficient-aabb-triangle-intersection-in-c-sharp
    pub fn intersect_triangle(
        &self,
        t0: &Position,
        t1: &Position,
        t2: &Position,
        triangle_normal: Option<&Direction>,
    ) -> bool {
        /// Get the maximum projection of a polygon on the given axis
        ///
        /// Returns: min, max on the given axis
        fn project(points: &[&Position], axis: &Direction) -> (f64, f64) {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for p in points {
                let val = axis.dot(&p.coords);
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
            (min, max)
        }

        // Test the box normals (x, y and z axis)
        let box_normals: [Direction; 3] = [
            Direction::new(1.0, 0.0, 0.0),
            Direction::new(0.0, 1.0, 0.0),
            Direction::new(0.0, 0.0, 1.0),
        ];
        for (i, box_normal) in box_normals.iter().enumerate() {
            let (min_triangle, max_triangle) = project(&[t0, t1, t2], box_normal);
            if max_triangle < self.bounds[0][i] || min_triangle > self.bounds[1][i] {
                return false;
            }
        }

        let ref u0 = Position::from(t0 - self.center);
        let ref u1 = Position::from(t1 - self.center);
        let ref u2 = Position::from(t2 - self.center);

        // Test Triangle normal
        {
            let ref n = match triangle_normal {
                Some(v) => *v,
                None => {
                    let u = t1 - t0;
                    let v = t2 - t0;
                    u.cross(&v).normalize()
                }
            };
            let triangle_projection = n.dot(&u0.coords);
            let r = self.projected_radius(n);
            if triangle_projection.abs() > r {
                return false;
            }
        }

        // Test the nine edge cross-products
        let triangle_edges = [*t1 - *t0, *t2 - *t1, *t0 - *t2];

        for edge in &triangle_edges {
            for box_normal in &box_normals {
                let ref axis = edge.cross(box_normal).normalize();
                let r = self.projected_radius(axis);
                let (min_triangle, max_triangle) = project(&[u0, u1, u2], axis);

                if r < min_triangle || -r > max_triangle {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_intersection_no_normal() {
        let aabb = AxisAlignedBoundingBox::from_bounds([
            Position::new(0.0, 0.0, 0.0),
            Position::new(10.0, 10.0, 10.0),
        ]);
        let ref t0 = Position::new(12.0, 9.0, 9.0);
        let ref t1 = Position::new(9.0, 12.0, 9.0);
        let ref t2 = Position::new(19.0, 19.0, 20.0);

        assert!(!aabb.intersect_triangle(t0, t1, t2, None));
    }

    #[test]
    fn triangle_intersection_no_normal_non_0_aligned() {
        let aabb = AxisAlignedBoundingBox::from_bounds([
            Position::new(10.0, 10.0, 10.0),
            Position::new(20.0, 20.0, 20.0),
        ]);
        let ref t0 = Position::new(22.0, 19.0, 19.0);
        let ref t1 = Position::new(19.0, 22.0, 19.0);
        let ref t2 = Position::new(29.0, 29.0, 30.0);

        assert!(!aabb.intersect_triangle(t0, t1, t2, None));
    }

    #[test]
    fn triangle_in_aabb_intersection_no_normal() {
        let aabb = AxisAlignedBoundingBox::from_bounds([
            Position::new(0.0, 0.0, 0.0),
            Position::new(10.0, 10.0, 10.0),
        ]);
        let ref t0 = Position::new(1.0, 1.0, 1.0);
        let ref t1 = Position::new(9.0, 12.0, 9.0);
        let ref t2 = Position::new(5.0, 5.0, 5.0);

        assert!(aabb.intersect_triangle(t0, t1, t2, None));
    }

    #[test]
    fn triangle_in_flat_aabb_intersection_no_normal() {
        let aabb = AxisAlignedBoundingBox::from_bounds([
            Position::new(0.0, 0.0, 0.0),
            Position::new(10.0, 10.0, 0.0),
        ]);
        // This triangle is lying intersecting with the box as a
        let ref t0 = Position::new(1.0, 1.0, 0.0);
        let ref t1 = Position::new(9.0, 12.0, 0.0);
        let ref t2 = Position::new(5.0, 5.0, 1.0);

        assert!(aabb.intersect_triangle(t0, t1, t2, None));
    }
}
