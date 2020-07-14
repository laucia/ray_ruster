extern crate ray_ruster;
use ray_ruster::geometry::bounding_box::AxisAlignedBoundingBox;
use ray_ruster::geometry::mesh::Mesh;
use ray_ruster::geometry::types::Position;
use std::path::Path;

#[test]
fn get_buggy_triangles() {
    let mesh = Mesh::load_off_file(Path::new("data/ram.off")).unwrap();
    let left_aabb = AxisAlignedBoundingBox::from_bounds([
        Position::new(-0.336138, -0.746779, 0.0000660419),
        Position::new(0.336138, -0.254103, 1.14864),
    ]);
    for i in 0..5 {
        let ref t = mesh.triangles[i];
        let ref t1 = mesh.vertices[t[0]];
        let ref t2 = mesh.vertices[t[1]];
        let ref t3 = mesh.vertices[t[2]];
        assert!(left_aabb.intersect_triangle(t1, t2, t3, None));
    }
}
