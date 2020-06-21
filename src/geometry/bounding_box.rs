extern crate nalgebra;
use crate::geometry::types::Position;

pub struct BoundingBox {
    pub bounds: [Position; 2],
    pub dim: Position,
}

impl BoundingBox {
    pub fn new(vertices: &Vec<Position>) -> BoundingBox {
        let min = vertices
            .iter()
            .fold(vertices[0], |min, vertice| min.inf(vertice));
        let max = vertices
            .iter()
            .fold(vertices[0], |max, vertice| max.sup(vertice));

        BoundingBox {
            bounds: [min, max],
            dim: Position::from(max - min),
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
}
