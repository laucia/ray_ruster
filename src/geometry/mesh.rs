extern crate nalgebra as na;
extern crate regex;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::num;
use std::path::Path;

use crate::geometry::types::{Direction, Position, Triangle};

/// This class is responsible for holding the geometry of the objects, and provide
/// easy look-ups of things like normals for both triangles and vertices
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Position>,
    pub vertex_normals: Vec<Direction>,
    pub triangles: Vec<Triangle>,
    pub triangle_normals: Vec<Direction>,
    pub vertex_index_triangle_indices_map: HashMap<usize, Vec<usize>>,
}

/// This defines the errors that can occure when parsing an OFF file
#[derive(Debug)]
pub enum OFFError {
    Io(io::Error),
    Re(regex::Error),
    String(&'static str),
    ParseFloat(num::ParseFloatError),
    ParseInt(num::ParseIntError),
}

impl Mesh {
    pub fn from_vertices_and_triangles(vertices: Vec<Position>, triangles: Vec<Triangle>) -> Mesh {
        // Calculate normals
        let triangle_normals = compute_triangle_normals(&triangles, &vertices);
        let vertex_normals = compute_vertex_normals(&triangles, &vertices, &triangle_normals);

        // Build maping
        let mut vertex_index_triangle_indices_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for (triangle_index, triangle) in triangles.iter().enumerate() {
            for i in 0..3 {
                let registry_entry = vertex_index_triangle_indices_map
                    .entry(triangle[i])
                    .or_insert(Vec::<usize>::new());
                registry_entry.push(triangle_index);
            }
        }
        Mesh {
            vertices: vertices,
            vertex_normals: vertex_normals,
            triangles: triangles,
            triangle_normals: triangle_normals,
            vertex_index_triangle_indices_map: vertex_index_triangle_indices_map,
        }
    }
    pub fn load_off_file(path: &Path) -> Result<Mesh, OFFError> {
        let off_file_result = File::open(path).map_err(OFFError::Io)?;

        let mut line = String::new();
        let mut reader = io::BufReader::new(off_file_result);

        // Check Magic Line
        reader.read_line(&mut line).map_err(OFFError::Io)?;
        if line != "OFF\n" {
            return Err(OFFError::String("Magic number OFF not present"));
        }
        line.clear();

        // Parse Number of vertices and triangles
        reader.read_line(&mut line).map_err(OFFError::Io)?;

        let re_size = (regex::Regex::new(
            r"^(?P<nb_vertices>\d+)\s+(?P<nb_triangles>\d+)\s+(?P<nb_x>\d+)\s+$",
        )
        .map_err(OFFError::Re))?;
        let captures = (re_size
            .captures(&line)
            .ok_or("Could not decode vertices and triangle count")
            .map_err(OFFError::String))?;
        let nb_vertices = captures
            .name("nb_vertices")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        let nb_triangles = captures
            .name("nb_triangles")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();

        let mut counter_vertices = nb_vertices;
        let mut count_triangles = nb_triangles;
        let mut vertices: Vec<Position> = Vec::with_capacity(counter_vertices);
        let mut triangles: Vec<Triangle> = Vec::with_capacity(count_triangles);

        let mut point: [f64; 3] = [0.0, 0.0, 0.0];
        let mut index: Triangle = [0, 0, 0];

        for line in reader.lines() {
            if counter_vertices > 0 {
                for (i, split) in line.unwrap().split_whitespace().take(3).enumerate() {
                    point[i] = split.parse::<f64>().map_err(OFFError::ParseFloat)?;
                }
                vertices.push(Position::from_slice(&point));
                counter_vertices -= 1;
            } else if count_triangles > 0 {
                for (i, split) in line.unwrap().split_whitespace().skip(1).take(3).enumerate() {
                    index[i] = split.parse::<usize>().map_err(OFFError::ParseInt)?;
                }
                triangles.push(index);
                count_triangles -= 1;
            } else {
                break;
            }
        }

        let mesh = Mesh::from_vertices_and_triangles(vertices, triangles);

        return Ok(mesh);
    }
}

/// Compute the normals of the triangles.
/// This defines the orientation of the triangles
/// calculated normals are normalized vectors (length 1.0)
fn compute_triangle_normals(triangles: &[Triangle], vertices: &[Position]) -> Vec<Direction> {
    triangles
        .iter()
        .map(|t| {
            let u = vertices[t[1]] - vertices[t[0]];
            let v = vertices[t[2]] - vertices[t[0]];
            u.cross(&v).normalize()
        })
        .collect()
}

/// Compute the normals of vertices
/// by averaging the normals of neighbouring triangles
/// calculated normals are normalized vectors (length 1.0)
fn compute_vertex_normals(
    triangles: &[Triangle],
    vertices: &[Position],
    triangle_normals: &[Direction],
) -> Vec<Direction> {
    let mut vertex_normals: Vec<Direction> = Vec::with_capacity(0);
    vertex_normals.resize(vertices.len(), Direction::new(0.0, 0.0, 0.0));

    for (t, n) in triangles.iter().zip(triangle_normals) {
        for i in 0..3 {
            vertex_normals[t[i]] += n;
        }
    }

    return vertex_normals.iter().map(|n| n.normalize()).collect();
}
