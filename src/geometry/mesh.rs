extern crate nalgebra as na;
extern crate regex;

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::num;
use std::path::Path;

use crate::geometry::types::{Direction, Position, Triangle};

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Position>,
    pub normals: Vec<Direction>,
    pub triangles: Vec<Triangle>,
}

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
        let normals = compute_vertex_normals(&triangles, &vertices);
        Mesh {
            vertices: vertices,
            normals: normals,
            triangles: triangles,
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

fn compute_vertex_normals(triangles: &[Triangle], vertices: &[Position]) -> Vec<Direction> {
    let triangle_normals: Vec<Direction> = triangles
        .iter()
        .map(|t| {
            let u = vertices[t[1]] - vertices[t[0]];
            let v = vertices[t[2]] - vertices[t[0]];
            u.cross(&v).normalize()
        })
        .collect();
    let mut vertex_normals: Vec<Direction> = Vec::with_capacity(0);
    vertex_normals.resize(vertices.len(), Direction::new(0.0, 0.0, 0.0));

    for (t, n) in triangles.iter().zip(triangle_normals) {
        for i in 0..3 {
            vertex_normals[t[i]] += n;
        }
    }

    return vertex_normals.iter().map(|n| n.normalize()).collect();
}
