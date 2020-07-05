extern crate gio;
extern crate gtk;
extern crate nalgebra as na;
extern crate rand;
extern crate ray_ruster;
extern crate tempfile;
use gio::prelude::*;
use gtk::prelude::*;

use rand::prelude::*;
use std::path::Path;
use tempfile::tempdir;

use ray_ruster::geometry::bounding_box::AxisAlignedBoundingBox;
use ray_ruster::geometry::kdtree::BoxIntersectIter;
use ray_ruster::geometry::kdtree::KdTree;
use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::ray::Ray;
use ray_ruster::geometry::types::{Direction, Position};
use ray_ruster::render::config;
use ray_ruster::render::config::CameraConfig;
use ray_ruster::render::image;
use ray_ruster::render::ray_tracer::clamp_u8;

/// Get the normal of the box face that we hit
/// This assumes that the intersection lies on the box,
/// otherwise this will return a 0 vector.
fn get_box_normal_debug(intersection: &Position, bb: &AxisAlignedBoundingBox) -> Direction {
    let mut normal = Direction::new(0.0, 0.0, 0.0);

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
        normal = Direction::new(0.0, 0.0, -1.0);
    }
    if (bb.bounds[1][2] - intersection[2]).abs() <= f32::EPSILON.into() {
        normal = Direction::new(0.0, 0.0, 1.0);
    }
    normal
}

fn make_box_tracer<'a>(
    kdt: &'a Box<KdTree>,
    max_depth: usize,
    camera_config: &'a CameraConfig,
) -> impl Fn(Ray) -> [u8; 3] + 'a {
    move |ray| {
        let box_iter = BoxIntersectIter::new(&ray, &kdt).closest_branch();
        let box_intersect = box_iter
            //.inspect(|x| println!("[{:},{:}]looking at: {:?}", i, j, x.bounding_box.bounds))
            .take(max_depth)
            .last();

        if box_intersect.is_some() {
            let ref hit = box_intersect.as_ref().unwrap().distance;
            let ref kd_node = box_intersect.as_ref().unwrap().node;
            let ref bb = kd_node.bounding_box;

            let intersection = ray.position + *hit * ray.direction;
            let normal = get_box_normal_debug(&intersection, bb);

            // Generate a random color from the box pointer
            let my_num_ptr: *const KdTree = &***kd_node;
            let random_seed = my_num_ptr as u64;
            let mut color_gen = rand::rngs::StdRng::seed_from_u64(random_seed);

            let color: [u8; 3] = [color_gen.gen(), color_gen.gen(), color_gen.gen()];
            let shade = (camera_config.camera_position - intersection)
                .normalize()
                .dot(&normal);
            return [
                clamp_u8(color[0] as f64 * shade),
                clamp_u8(color[1] as f64 * shade),
                clamp_u8(color[2] as f64 * shade),
            ];
        } else {
            return [0, 0, 0];
        }
    }
}

fn main() {
    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    let kdt = Box::new(KdTree::from_vertices(&mesh.vertices));

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

    // Render all images
    let dir = tempdir().ok().unwrap();
    let mut paths = Vec::new();

    for depth in 1..10 {
        let img = image::render_image(make_box_tracer(&kdt, depth, &camera_config), &camera_config);
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
