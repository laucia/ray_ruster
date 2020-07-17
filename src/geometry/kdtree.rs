use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

use crate::geometry::bounding_box::AxisAlignedBoundingBox;
use crate::geometry::mesh::Mesh;
use crate::geometry::ray::Ray;
use crate::geometry::types::Triangle;
use crate::geometry::types::{Direction, Position};

pub struct KdTree {
    pub bounding_box: AxisAlignedBoundingBox,
    left: Option<Box<KdTree>>,
    right: Option<Box<KdTree>>,

    // leaf
    pub vertices_index: Option<Vec<usize>>,
    pub triangle_index: Option<Vec<usize>>,
}

impl KdTree {
    fn new_node(
        bb: AxisAlignedBoundingBox,
        left: Option<Box<KdTree>>,
        right: Option<Box<KdTree>>,
    ) -> KdTree {
        KdTree {
            bounding_box: bb,
            left: left,
            right: right,
            vertices_index: None,
            triangle_index: None,
        }
    }

    fn new_leaf(
        bb: AxisAlignedBoundingBox,
        vertices_index: Vec<usize>,
        triangle_index: Vec<usize>,
    ) -> KdTree {
        KdTree {
            bounding_box: bb,
            left: None,
            right: None,
            vertices_index: Some(vertices_index),
            triangle_index: Some(triangle_index),
        }
    }

    /// Create a KdTree corresponding to the given mesh to
    /// serve spatial queries on the mesh
    ///
    /// This is performed in 2 steps:
    ///    1. The box are defined based on the vertex density
    ///    2. The triangles are put in the leaves they intersect
    pub fn from_mesh(mesh: &Mesh) -> Box<KdTree> {
        fn recursion_internal(
            mesh: &Mesh,
            bb: AxisAlignedBoundingBox,
            index_vertices_pairs: Vec<(usize, &Position)>,
            index_triangle_pairs: Vec<(usize, &Triangle)>,
        ) -> KdTree {
            // Terminal condition
            if index_vertices_pairs.len() < 10 {
                return KdTree::new_leaf(
                    bb,
                    index_vertices_pairs
                        .iter()
                        .map(|(i, _)| i.clone())
                        .collect(),
                    index_triangle_pairs
                        .iter()
                        .map(|(i, _)| i.clone())
                        .collect(),
                );
            }
            // Find split plane
            let largest_dim = bb.largest_dim();
            let vertices: Vec<&Position> =
                index_vertices_pairs.iter().map(|(_, pos)| *pos).collect();
            let median = get_median(largest_dim, &vertices);

            // Split Points
            let right_vertices: Vec<(usize, &Position)> = index_vertices_pairs
                .iter()
                .filter(|&n| {
                    let (_, pos) = n;
                    pos[largest_dim] >= median
                })
                .map(|(i, pos)| (i.clone(), *pos))
                .collect();
            let left_vertices: Vec<(usize, &Position)> = index_vertices_pairs
                .iter()
                .filter(|&n| {
                    let (_, pos) = n;
                    pos[largest_dim] < median
                })
                .map(|(i, pos)| (i.clone(), *pos))
                .collect();
            // Split Bounding Boxes
            let (left_bb, right_bb) = bb.split(largest_dim, median).unwrap();

            // Split triangles
            let left_triangles: Vec<(usize, &Triangle)> = index_triangle_pairs
                .iter()
                .filter(|&n| {
                    let (index, t) = n;
                    let ref t0 = mesh.vertices[t[0]];
                    let ref t1 = mesh.vertices[t[1]];
                    let ref t2 = mesh.vertices[t[2]];
                    let ref n = mesh.triangle_normals[*index];
                    left_bb.intersect_triangle(t0, t1, t2, Some(n))
                })
                .map(|(i, t)| (i.clone(), *t))
                .collect();
            let right_triangles: Vec<(usize, &Triangle)> = index_triangle_pairs
                .iter()
                .filter(|&n| {
                    let (index, t) = n;
                    let ref t0 = mesh.vertices[t[0]];
                    let ref t1 = mesh.vertices[t[1]];
                    let ref t2 = mesh.vertices[t[2]];
                    let ref n = mesh.triangle_normals[*index];
                    right_bb.intersect_triangle(t0, t1, t2, Some(n))
                })
                .map(|(i, t)| (i.clone(), *t))
                .collect();

            // Recursion
            KdTree::new_node(
                bb,
                Some(Box::from(recursion_internal(
                    mesh,
                    left_bb,
                    left_vertices,
                    left_triangles,
                ))),
                Some(Box::from(recursion_internal(
                    mesh,
                    right_bb,
                    right_vertices,
                    right_triangles,
                ))),
            )
        }

        // Initialize the recursion
        let bb = AxisAlignedBoundingBox::new(&mesh.vertices);
        let index_vertices_pairs: Vec<(usize, &Position)> =
            mesh.vertices.iter().enumerate().collect();
        let index_triangles_pairs: Vec<(usize, &Triangle)> =
            mesh.triangles.iter().enumerate().collect();

        Box::from(recursion_internal(
            mesh,
            bb,
            index_vertices_pairs,
            index_triangles_pairs,
        ))
    }

    pub fn is_leaf(&self) -> bool {
        self.vertices_index.is_some()
    }
}

pub fn iter_intersect_ray<'a>(
    kdtree: &'a Box<KdTree>,
    ray: &'a Ray,
) -> BoxIntersectIter<'a, RayIntersector<'a>> {
    let ray_box_intersector = RayIntersector { ray: ray };
    BoxIntersectIter::<'a, RayIntersector>::new(ray_box_intersector, kdtree)
}

pub fn iter_intersect_triangle<'a>(
    kdtree: &'a Box<KdTree>,
    t0: &'a Position,
    t1: &'a Position,
    t2: &'a Position,
    n: &'a Direction,
) -> BoxIntersectIter<'a, TriangleIntersector<'a>> {
    let ray_box_intersector = TriangleIntersector {
        t0: t0,
        t1: t1,
        t2: t2,
        n: n,
    };
    BoxIntersectIter::<'a, TriangleIntersector>::new(ray_box_intersector, kdtree)
}

fn get_median(dim: usize, vertices: &Vec<&Position>) -> f64 {
    let mut largest_dim_values = vertices.iter().map(|x| x[dim]).collect::<Vec<f64>>();
    largest_dim_values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let median_index: usize = largest_dim_values.len() / 2;
    let median = largest_dim_values[median_index];

    median
}

pub struct BoxIntersect<'a> {
    pub distance: f64,
    pub node: &'a Box<KdTree>,
}

impl<'a> Ord for BoxIntersect<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl<'a> PartialOrd for BoxIntersect<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // We are reversing the order to get a min heap
        other.distance.partial_cmp(&self.distance)
    }
}

impl<'a> Eq for BoxIntersect<'a> {}

impl<'a> PartialEq for BoxIntersect<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

/// Yields all the nodes of the tree intersecting with the ray
/// ordered by depth and intersection distance, ascending

pub trait BoxIntersector<'a> {
    fn intersect_box(&self, kdt_node: &'a Box<KdTree>) -> Option<BoxIntersect<'a>>;
}

pub struct RayIntersector<'a> {
    ray: &'a Ray,
}

impl<'a> BoxIntersector<'a> for RayIntersector<'a> {
    fn intersect_box(&self, kdt_node: &'a Box<KdTree>) -> Option<BoxIntersect<'a>> {
        let hit = self.ray.intersect_box(&(*kdt_node).bounding_box.bounds);
        match hit {
            Some(distance) => Some(BoxIntersect {
                distance: distance,
                node: kdt_node,
            }),
            None => None,
        }
    }
}

pub struct TriangleIntersector<'a> {
    t0: &'a Position,
    t1: &'a Position,
    t2: &'a Position,
    n: &'a Direction,
}

impl<'a> BoxIntersector<'a> for TriangleIntersector<'a> {
    fn intersect_box(&self, kdt_node: &'a Box<KdTree>) -> Option<BoxIntersect<'a>> {
        let hit =
            &(*kdt_node)
                .bounding_box
                .intersect_triangle(self.t0, self.t1, self.t2, Some(self.n));
        match hit {
            true => Some(BoxIntersect {
                distance: 1.0, // TODO: Is this OK ?
                node: kdt_node,
            }),
            false => None,
        }
    }
}

pub struct BoxIntersectIter<'a, A: BoxIntersector<'a>> {
    next_nodes: BinaryHeap<BoxIntersect<'a>>,
    box_intersector: A,
}

impl<'a, A> BoxIntersectIter<'a, A>
where
    A: BoxIntersector<'a>,
{
    pub fn new(box_intersector: A, first_node: &'a Box<KdTree>) -> BoxIntersectIter<'a, A> {
        let mut heap = BinaryHeap::new();
        let intersect = box_intersector.intersect_box(first_node);
        if intersect.is_some() {
            heap.push(intersect.unwrap())
        }
        BoxIntersectIter {
            next_nodes: heap,
            box_intersector: box_intersector,
        }
    }
    pub fn closest_branch(self) -> impl Iterator<Item = BoxIntersect<'a>> {
        self.scan(0, |predecessor_is_leaf, intersect: BoxIntersect<'_>| {
            if *predecessor_is_leaf == 1 {
                return None;
            }
            if intersect.node.is_leaf() {
                *predecessor_is_leaf += 1
            }
            Some(intersect)
        })
    }

    /// Yields all the leaves of the tree intersecting with the ray
    /// ordered by intersection distance, ascending
    pub fn leaves(self) -> impl Iterator<Item = BoxIntersect<'a>> {
        self.filter(|x| x.node.is_leaf())
    }
}

impl<'a, A: BoxIntersector<'a>> Iterator for BoxIntersectIter<'a, A> {
    type Item = BoxIntersect<'a>;

    fn next(&mut self) -> Option<BoxIntersect<'a>> {
        let next_node = self.next_nodes.pop();
        if next_node.is_none() {
            return None;
        }

        let cur_node = next_node.unwrap();

        // We have reached a leaf we can stop
        if cur_node.node.is_leaf() {
            return Some(cur_node);
        }

        // Otherwise let's check which child is the next node
        // before returning the node
        let left_child = (*cur_node.node).left.as_ref().unwrap();
        let right_child = (*cur_node.node).right.as_ref().unwrap();
        let intersect_left = self.box_intersector.intersect_box(left_child);
        let intersect_right = self.box_intersector.intersect_box(right_child);

        match (intersect_left, intersect_right) {
            (None, None) => {
                println!("Problem with parent box spliting");
                return None;
            }
            (Some(i_left), None) => {
                self.next_nodes.push(i_left);
            }
            (None, Some(i_right)) => {
                self.next_nodes.push(i_right);
            }
            (Some(i_left), Some(i_right)) => {
                self.next_nodes.push(i_left);
                self.next_nodes.push(i_right);
            }
        }

        return Some(cur_node);
    }
}

/// Return all the leafs under a given KDTree Node
///
/// This iterator is used mostly for debugging, and
/// performs a DFS traversal
pub struct KdTreeLeafIter<'a> {
    /// LIFO queue used for DFS
    pending: VecDeque<&'a Box<KdTree>>,
}

impl<'a> Iterator for KdTreeLeafIter<'a> {
    type Item = &'a Box<KdTree>;

    fn next(&mut self) -> Option<&'a Box<KdTree>> {
        while self.pending.len() > 0 {
            let current = self.pending.pop_back().unwrap();
            if current.is_leaf() {
                return Some(current);
            }
            if current.left.is_some() {
                self.pending.push_back(&current.left.as_ref().unwrap())
            }
            if current.right.is_some() {
                self.pending.push_back(&current.right.as_ref().unwrap())
            }
        }
        return None;
    }
}

impl<'a> KdTreeLeafIter<'a> {
    pub fn new(first_node: &'a Box<KdTree>) -> KdTreeLeafIter<'a> {
        let mut pending = VecDeque::new();
        pending.push_back(first_node);

        KdTreeLeafIter { pending: pending }
    }
}
