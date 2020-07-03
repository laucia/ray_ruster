extern crate gio;
extern crate gtk;
extern crate nalgebra as na;
extern crate ray_ruster;
extern crate tempfile;

use gio::prelude::*;
use gtk::prelude::*;
use std::path::Path;
use std::time::Instant;

use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::types::{Direction, Position};
use ray_ruster::render::config;
use ray_ruster::render::ray_tracer;

use tempfile::tempdir;

fn main() {
    let start = Instant::now();

    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    println!("{:?}: loaded OFF model", start.elapsed());
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
    let img = ray_tracer::render(&mesh, &camera_config);
    println!("{:?}: rendering done", start.elapsed());
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
