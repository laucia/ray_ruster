extern crate image;

use crate::geometry::mesh::Mesh;
use crate::geometry::ray::Ray;
use crate::geometry::types::{Direction, Position};
use crate::render::config::{CameraConfig, NormalMode, RenderingConfig};

pub fn clamp_u8(f: f64) -> u8 {
    if f <= 0.0 {
        return 0;
    } else if f >= 255.0 {
        return 255;
    } else {
        return f.ceil() as u8;
    }
}

fn interpolation_n_phong(
    n1: &Direction,
    n2: &Direction,
    n3: &Direction,
    coord: &[f64; 2],
) -> Direction {
    return (*n1 * (1.0 - coord[0] - coord[1]) + coord[0] * *n2 + coord[1] * *n3).normalize();
}

pub fn make_naive_ray_tracer<'a>(
    mesh: &'a Mesh,
    camera_config: &'a CameraConfig,
    rendering_config: &'a RenderingConfig,
) -> impl Fn(Ray) -> [u8; 3] + 'a {
    move |ray| {
        let mut closest_intersection = Position::new(f64::NAN, f64::NAN, f64::NAN);
        let mut closest_normal = Direction::new(f64::NAN, f64::NAN, f64::NAN);
        let mut hit = false;

        for (triangle_index, triangle) in mesh.triangles.iter().enumerate() {
            let ref t1 = mesh.vertices[triangle[0]];
            let ref t2 = mesh.vertices[triangle[1]];
            let ref t3 = mesh.vertices[triangle[2]];

            let intersection_opt = ray.intersect_triangle(t1, t2, t3);
            if intersection_opt.is_some() {
                let (intersection_point, bar_coord) = intersection_opt.unwrap();
                // Init the value
                if !hit
                    || (closest_intersection - camera_config.camera_position).norm_squared()
                        >= (intersection_point - camera_config.camera_position).norm_squared()
                {
                    closest_intersection = intersection_point;
                    closest_normal = match rendering_config.normal_mode {
                        NormalMode::Phong => interpolation_n_phong(
                            &mesh.vertex_normals[triangle[0]],
                            &mesh.vertex_normals[triangle[1]],
                            &mesh.vertex_normals[triangle[2]],
                            &bar_coord,
                        ),
                        NormalMode::Triangle => mesh.triangle_normals[triangle_index],
                    }
                }
                if !hit {
                    hit = true;
                }
            }
        }
        match hit {
            true => {
                let color = clamp_u8(
                    (camera_config.camera_position - closest_intersection)
                        .normalize()
                        .dot(&closest_normal)
                        * 255.0,
                );
                [color, color, color]
            }
            _ => [0, 0, 0],
        }
    }
}
