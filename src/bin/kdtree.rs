extern crate gio;
extern crate gtk;
extern crate image;
extern crate nalgebra as na;
extern crate ray_ruster;
extern crate tempfile;

use gio::prelude::*;
use gtk::prelude::*;
use image::{Rgb, RgbImage};
use ray_ruster::geometry::bounding_box::AxisAlignedBoundingBox;
use ray_ruster::geometry::kdtree::BoxIntersectIter;
use ray_ruster::geometry::kdtree::KdTree;

use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::ray::Ray;
use ray_ruster::geometry::types::{Direction, Position};
use ray_ruster::render::config;
use ray_ruster::render::config::CameraConfig;
use ray_ruster::render::ray_tracer::clamp_u8;
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

    let kdt = Box::new(KdTree::from_vertices(&mesh.vertices));
    let mut has_printed = false;

    for i in 0..width {
        for j in 0..height {
            let dir = ((i as f64 - (width as f64) / 2.0) * step_x * camera_config.x
                + (j as f64 - (height as f64) / 2.0) * step_y * camera_config.y
                + camera_config.z)
                .normalize();
            let ray = Ray::new(camera_position, dir);
            let box_iter = BoxIntersectIter::new(&ray, &kdt);
            let kd_node = box_iter
                //.inspect(|x| println!("[{:},{:}]looking at: {:?}", i, j, x.bounding_box.bounds))
                .last();

            if kd_node.is_some() {
                let bb = &kd_node.unwrap().bounding_box;
                let hit = ray.intersect_box(&bb.bounds);
                if hit.is_none() {
                    println!("OUPS");
                    img.put_pixel(i, height - 1 - j, Rgb([255, 0, 0]));
                    continue;
                }

                let intersection = ray.position + hit.unwrap() * ray.direction;
                let mut normal = Direction::new(0.0, 0.0, 0.0);

                // Get the normal of the box face that we hit
                // This is ugly and should maybe be refactored
                if (bb.bounds[0][0] - intersection[0]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(-1.0, 0.0, 0.0);
                }
                if (bb.bounds[1][0] - intersection[0]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(1.0, 0.0, 0.0);
                }
                if (bb.bounds[0][1] - intersection[1]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(0.0, -1.0, 0.0);
                }
                if (bb.bounds[1][1] - intersection[1]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(0.0, 1.0, 0.0);
                }
                if (bb.bounds[0][2] - intersection[2]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(-1.0, 0.0, -1.0);
                }
                if (bb.bounds[1][2] - intersection[2]).abs() <= f32::EPSILON.into() {
                    normal = Direction::new(0.0, 0.0, 1.0);
                }
                if !has_printed {
                    println!("{:} {:}", bb.bounds[0], bb.bounds[1]);
                    println!("{:}", intersection);
                    has_printed = true;
                }

                let color = kd_node.unwrap().color;
                let shade =
                    clamp_u8((camera_position - intersection).normalize().dot(&normal) * 255.0);
                img.put_pixel(
                    i,
                    height - 1 - j,
                    Rgb([color[0] * shade, color[1] * shade, color[2] * shade]),
                );
            } else {
                img.put_pixel(i, height - 1 - j, Rgb([0, 0, 0]));
            }
        }
    }

    return img;
}

fn main() {
    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    let bb = AxisAlignedBoundingBox::new(&mesh.vertices);
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
