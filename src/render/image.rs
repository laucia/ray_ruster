extern crate image;

use self::image::{Rgb, RgbImage};
use crate::geometry::ray::Ray;
use crate::render::config::CameraConfig;

pub fn render_image<F: Fn(Ray) -> [u8; 3]>(
    ray_tracer: F,
    camera_config: &CameraConfig,
) -> RgbImage {
    let mut img = RgbImage::new(camera_config.width, camera_config.height);

    let step_x = camera_config.fov.tan() / (camera_config.width as f64);
    let step_y =
        camera_config.fov.tan() / camera_config.aspect_ratio / (camera_config.height as f64);
    let camera_position = camera_config.camera_position;
    let width = camera_config.width;
    let height = camera_config.height;

    for i in 0..width {
        for j in 0..height {
            let dir = ((i as f64 - (width as f64) / 2.0) * step_x * camera_config.x
                + (j as f64 - (height as f64) / 2.0) * step_y * camera_config.y
                + camera_config.z)
                .normalize();
            let ray = Ray::new(camera_position, dir);
            let color = ray_tracer(ray);
            img.put_pixel(i, height - 1 - j, Rgb([color[0], color[1], color[2]]));
        }
    }

    return img;
}
