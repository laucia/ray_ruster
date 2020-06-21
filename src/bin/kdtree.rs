extern crate gio;
extern crate gtk;
extern crate image;
extern crate nalgebra as na;
extern crate ray_ruster;
extern crate tempfile;

use gio::prelude::*;
use gtk::prelude::*;
use image::{Rgb, RgbImage};
use ray_ruster::geometry::bounding_box::BoundingBox;
use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::ray::Ray;
use ray_ruster::geometry::types::{Direction, Position};
use ray_ruster::render::config;
use ray_ruster::render::config::CameraConfig;
use std::path::Path;
use tempfile::tempdir;

pub fn render(mesh: &Mesh, camera_config: &CameraConfig) -> RgbImage {
    let mut img = RgbImage::new(camera_config.width, camera_config.height);

    let step_x = camera_config.fov.tan() / (camera_config.width as f64);
    let step_y =
        camera_config.fov.tan() / camera_config.aspect_ratio / (camera_config.height as f64);
    let camera_position = camera_config.camera_position;
    let width = camera_config.width;
    let height = camera_config.height;

    let bb = BoundingBox::new(&mesh.vertices);

    for i in 0..width {
        for j in 0..height {
            let dir = ((i as f64 - (width as f64) / 2.0) * step_x * camera_config.x
                + (j as f64 - (height as f64) / 2.0) * step_y * camera_config.y
                + camera_config.z)
                .normalize();
            let ray = Ray::new(camera_position, dir);
            let hit = ray.intersect_box(&bb.bounds);

            if hit {
                let color = 255;
                img.put_pixel(i, height - 1 - j, Rgb([color, color, color]));
            } else {
                img.put_pixel(i, height - 1 - j, Rgb([0, 0, 0]));
            }
        }
    }

    return img;
}

fn main() {
    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    let bb = BoundingBox::new(&mesh.vertices);
    let rot = na::Rotation3::face_towards(
        &Direction::new(-1.0, 1.0, 0.0),
        &Direction::new(0.0, 0.0, 1.0),
    );
    let camera_config = config::CameraConfig {
        camera_position: rot * Position::new(0.0, 0.5, -10.0),
        x: rot * Direction::new(1.0, 0.0, 0.0),
        y: rot * Direction::new(0.0, 1.0, 0.0),
        z: rot * Direction::new(0.0, 0.0, 1.0),
        fov: 60.0,
        aspect_ratio: 4.0 / 3.0,
        width: 400,
        height: 300,
    };
    println!("{:?}", bb.bounds[0]);
    println!("{:?}", bb.bounds[1]);

    let img = render(&mesh, &camera_config);
    let dir = tempdir().ok().unwrap();
    let file_path = dir.path().join("render.png");
    let _ = img.save(Path::new(&file_path));
    let application = gtk::Application::new(Some("main.ray_ruster"), Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("ray_ruster");
        window.set_default_size(350, 70);
        let im = gtk::Image::new_from_file(Path::new(&file_path));
        window.add(&im);
        window.show_all();
    });

    application.run(&[]);
}
