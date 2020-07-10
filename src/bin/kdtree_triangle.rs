extern crate gio;
extern crate gtk;
extern crate nalgebra as na;
extern crate ray_ruster;
extern crate tempfile;

use gio::prelude::*;
use gtk::prelude::*;
use std::path::Path;
use tempfile::tempdir;

use ray_ruster::geometry::kdtree::{iter_intersect_ray, KdTree, KdTreeLeafIter};
use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::ray::Ray;
use ray_ruster::geometry::types::{Direction, Position};
use ray_ruster::render::config;
use ray_ruster::render::image;
use ray_ruster::render::ray_tracer;

fn kdt_to_mesh(kdt: &Box<KdTree>, mesh: &Mesh) -> Mesh {
    let vertices_index: Vec<usize> = KdTreeLeafIter::new(kdt)
        .flat_map(|x| x.vertices_index.as_ref().unwrap().iter())
        .map(|x| x.clone())
        .collect();
    println!("vertices: {:}", vertices_index.len());
    let mut triangle_index: Vec<usize> = KdTreeLeafIter::new(kdt)
        .flat_map(|x| x.triangle_index.as_ref().unwrap().iter())
        .map(|x| x.clone())
        .collect();
    triangle_index.sort_unstable();
    triangle_index.dedup();
    println!("triangles: {:}", triangle_index.len());

    // We copy all the vertices because this is debug
    // This avoids having to convert the triangles
    let vertices = mesh.vertices.iter().map(|x| x.clone()).collect();
    let triangles = triangle_index
        .iter()
        .map(|i| mesh.triangles[*i])
        .map(|x| x.clone())
        .collect();

    Mesh::from_vertices_and_triangles(vertices, triangles)
}

fn make_sample_ray(i: usize, j: usize, camera_config: &config::CameraConfig) -> Ray {
    let step_x = camera_config.fov.tan() / (camera_config.width as f64);
    let step_y =
        camera_config.fov.tan() / camera_config.aspect_ratio / (camera_config.height as f64);

    let dir = ((i as f64 - (camera_config.width as f64) / 2.0) * step_x * camera_config.x
        + (j as f64 - (camera_config.height as f64) / 2.0) * step_y * camera_config.y
        + camera_config.z)
        .normalize();

    Ray::new(camera_config.camera_position, dir)
}

fn main() {
    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    let kdt = KdTree::from_mesh(&mesh);
    println!(
        "Mesh: {:?} vertices, {:?} triangles",
        mesh.vertices.len(),
        mesh.triangles.len()
    );
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
        aspect_ratio: 1.0,
        width: 300,
        height: 300,
    };

    let rendering_config = config::RenderingConfig {
        normal_mode: config::NormalMode::Triangle,
    };

    let sample_ray = make_sample_ray(150, 150, &camera_config);
    let box_iter = iter_intersect_ray(&kdt, &sample_ray).closest_branch();

    // Render all images
    let dir = tempdir().ok().unwrap();
    let mut paths = Vec::new();

    for (depth, kdt_node) in box_iter.take(12).enumerate() {
        let mesh = kdt_to_mesh(kdt_node.node, &mesh);
        let img = image::render_image(
            ray_tracer::make_naive_ray_tracer(&mesh, &camera_config, &rendering_config),
            &camera_config,
        );
        let file_path = dir
            .path()
            .join(format!("render_{depth}.png", depth = depth));
        let _ = img.save(Path::new(&file_path));
        paths.push(file_path)
    }

    let application = gtk::Application::new(Some("main.ray_ruster"), Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("ray_ruster");
        window.set_default_size(350, 70);
        let grid = gtk::Grid::new();
        for (i, path) in paths.iter().enumerate() {
            let im = gtk::Image::new_from_file(Path::new(path));
            grid.attach(&im, (i % 3) as i32, (i / 3) as i32, 1, 1);
        }
        window.add(&grid);
        window.show_all();
    });

    application.run(&[]);
}
