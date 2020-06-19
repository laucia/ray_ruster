extern crate nalgebra as na;

pub use na::Norm;
use na::{Point3, Vector3};

/// The type of vertex coordinates.
pub type Position = Point3<f64>;
/// The type of normals.
pub type Direction = Vector3<f64>;
/// Triangle as indices of a vertex array
pub type Triangle = [usize; 3];
