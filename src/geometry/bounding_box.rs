extern crate nalgebra;
use crate::geometry::types::Position;

pub struct AxisAlignedBoundingBox {
    pub bounds: [Position; 2],
    pub dim: Position,
}

impl AxisAlignedBoundingBox {
    pub fn new(vertices: &Vec<Position>) -> AxisAlignedBoundingBox {
        let min = vertices
            .iter()
            .fold(vertices[0], |min, vertice| min.inf(vertice));
        let max = vertices
            .iter()
            .fold(vertices[0], |max, vertice| max.sup(vertice));

        AxisAlignedBoundingBox::from_bounds([min, max])
    }

    pub fn from_bounds(bounds: [Position; 2]) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            bounds: bounds,
            dim: Position::from(bounds[1] - bounds[0]),
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

    pub fn split(
        &self,
        dim: usize,
        at: f64,
    ) -> Option<(AxisAlignedBoundingBox, AxisAlignedBoundingBox)> {
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
            AxisAlignedBoundingBox::from_bounds([min, at_max]),
            AxisAlignedBoundingBox::from_bounds([at_min, max]),
        ))
    }
}
