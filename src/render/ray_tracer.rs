extern crate image;

use crate::geometry::kdtree::{iter_intersect_ray, KdTree};
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

/// Return a function that given a ray will calculate its observed color
/// i.e. background or object
///
/// This function proceeds by iterating all the triangles in the mesh to
/// look for intersections
pub fn make_naive_ray_tracer<'a>(
    mesh: &'a Mesh,
    camera_config: &'a CameraConfig,
    rendering_config: &'a RenderingConfig,
) -> impl Fn(Ray) -> [u8; 3] + 'a {
    move |ray| {
        let all_triangle_indices_iter = 0..mesh.triangles.len();
        let triangle_intersect = triangles_closest_intersection(
            all_triangle_indices_iter.collect::<Vec<usize>>().iter(),
            &ray,
            mesh,
        );
        match triangle_intersect {
            Some(intersect) => {
                shade_triangle_hit(&intersect, mesh, camera_config, rendering_config)
            }
            None => [0, 0, 0],
        }
    }
}

/// Return a function that given a ray will calculate its observed color
/// i.e. background or object
///
/// This function leverages a kd-tree for faster triangle/ray intersection
pub fn make_kdt_ray_tracer<'a>(
    mesh: &'a Mesh,
    kdt: &'a Box<KdTree>,
    camera_config: &'a CameraConfig,
    rendering_config: &'a RenderingConfig,
) -> impl Fn(Ray) -> [u8; 3] + 'a {
    move |ray| {
        let box_iter = iter_intersect_ray(&kdt, &ray).leaves();
        for box_intersect in box_iter {
            let ref triangle_index = box_intersect.node.triangle_index.as_ref().unwrap();
            let triangle_intersect =
                triangles_closest_intersection(triangle_index.iter(), &ray, mesh);
            if triangle_intersect.is_none() {
                continue;
            }
            return shade_triangle_hit(
                &triangle_intersect.unwrap(),
                mesh,
                camera_config,
                rendering_config,
            );
        }

        return [0, 0, 0];
    }
}

pub struct TriangleIntersect {
    pub triangle_index: usize,
    pub intersection: Position,
    pub barycentric_coordinate: [f64; 2],
}

fn triangles_closest_intersection<'a, I>(
    triangle_indices: I,
    ray: &Ray,
    mesh: &Mesh,
) -> Option<TriangleIntersect>
where
    I: Iterator<Item = &'a usize>,
{
    let mut closest_triangle_index: usize = 0;
    let mut closest_intersection = Position::new(f64::NAN, f64::NAN, f64::NAN);
    let mut closest_bar_coord = [f64::NAN, f64::NAN];
    let mut hit = false;
    for triangle_index in triangle_indices {
        let ref triangle = mesh.triangles[*triangle_index];
        let ref t0 = mesh.vertices[triangle[0]];
        let ref t1 = mesh.vertices[triangle[1]];
        let ref t2 = mesh.vertices[triangle[2]];

        let intersection_opt = ray.intersect_triangle(t0, t1, t2);
        if intersection_opt.is_some() {
            let (intersection_point, bar_coord) = intersection_opt.unwrap();
            // Init the value
            if !hit
                || (closest_intersection - ray.position).norm_squared()
                    >= (intersection_point - ray.position).norm_squared()
            {
                closest_triangle_index = *triangle_index;
                closest_intersection = intersection_point;
                closest_bar_coord = bar_coord;
            }
            if !hit {
                hit = true;
            }
        }
    }
    match hit {
        true => Some(TriangleIntersect {
            triangle_index: closest_triangle_index,
            intersection: closest_intersection,
            barycentric_coordinate: closest_bar_coord,
        }),
        _ => None,
    }
}

fn shade_triangle_hit(
    intersect: &TriangleIntersect,
    mesh: &Mesh,
    camera_config: &CameraConfig,
    rendering_config: &RenderingConfig,
) -> [u8; 3] {
    let closest_normal = match rendering_config.normal_mode {
        NormalMode::Phong => {
            let ref triangle = mesh.triangles[intersect.triangle_index];
            interpolation_n_phong(
                &mesh.vertex_normals[triangle[0]],
                &mesh.vertex_normals[triangle[1]],
                &mesh.vertex_normals[triangle[2]],
                &intersect.barycentric_coordinate,
            )
        }
        NormalMode::Triangle => mesh.triangle_normals[intersect.triangle_index],
    };
    let color = clamp_u8(
        (camera_config.camera_position - intersect.intersection)
            .normalize()
            .dot(&closest_normal)
            * 255.0,
    );
    [color, color, color]
}
