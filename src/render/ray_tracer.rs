extern crate image;

use self::image::{Rgb, RgbImage};

use crate::geometry::mesh::Mesh;
use crate::geometry::ray::Ray;
use crate::geometry::types::{Direction, Position};
use crate::render::config::CameraConfig;

fn clamp_u8(f: f64) -> u8 {
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

pub fn render(mesh: &Mesh, camera_config: &CameraConfig) -> RgbImage {
    let mut img = RgbImage::new(camera_config.width, camera_config.height);

    let step_x = camera_config.fov.tan() / (camera_config.width as f64);
    let step_z =
        camera_config.fov.tan() / camera_config.aspect_ratio / (camera_config.height as f64);
    let camera_position = camera_config.camera_position;
    let width = camera_config.width;
    let height = camera_config.height;

    for i in 0..width {
        for j in 0..height {
            let dir = ((i as f64 - (width as f64) / 2.0) * step_x * camera_config.x
                + (j as f64 - (height as f64) / 2.0) * step_z * camera_config.z
                + camera_config.y)
                .normalize();
            let vertex = Ray {
                position: camera_position,
                direction: dir,
            };

            let mut closest_intersection = Position::new(f64::NAN, f64::NAN, f64::NAN);
            let mut closest_normal = Direction::new(f64::NAN, f64::NAN, f64::NAN);
            let mut hit = false;

            for triangle in mesh.triangles.iter() {
                let ref t1 = mesh.vertices[triangle[0]];
                let ref t2 = mesh.vertices[triangle[1]];
                let ref t3 = mesh.vertices[triangle[2]];

                let intersection_opt = vertex.intersect(t1, t2, t3);
                if intersection_opt.is_some() {
                    let (intersection_point, bar_coord) = intersection_opt.unwrap();
                    // Init the value
                    if !hit
                        || (closest_intersection - camera_position).norm_squared()
                            >= (intersection_point - camera_position).norm_squared()
                    {
                        closest_intersection = intersection_point;
                        closest_normal = interpolation_n_phong(
                            &mesh.normals[triangle[0]],
                            &mesh.normals[triangle[1]],
                            &mesh.normals[triangle[2]],
                            &bar_coord,
                        );
                    }
                    if !hit {
                        hit = true;
                    }
                }
            }
            if hit {
                let color = clamp_u8(
                    (camera_position - closest_intersection)
                        .normalize()
                        .dot(&closest_normal)
                        * 255.0,
                );
                img.put_pixel(i, height - 1 - j, Rgb([color, color, color]));
            } else {
                img.put_pixel(i, height - 1 - j, Rgb([0, 0, 0]));
            }
        }
    }

    return img;
}
