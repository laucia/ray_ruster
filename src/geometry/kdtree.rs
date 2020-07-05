use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

use crate::geometry::bounding_box::AxisAlignedBoundingBox;
use crate::geometry::ray::Ray;
use crate::geometry::types::Position;

pub struct KdTree {
    pub bounding_box: AxisAlignedBoundingBox,
    left: Option<Box<KdTree>>,
    right: Option<Box<KdTree>>,

    // leaf
    pub vertices_index: Option<Vec<usize>>,
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
        }
    }

    fn new_leaf(bb: AxisAlignedBoundingBox, vertices_index: Option<Vec<usize>>) -> KdTree {
        KdTree {
            bounding_box: bb,
            left: None,
            right: None,
            vertices_index: vertices_index,
        }
    }

    pub fn from_vertices(vertices: &Vec<Position>) -> KdTree {
        let bb = AxisAlignedBoundingBox::new(vertices);

        let largest_dim = bb.largest_dim();
        let median = get_median(largest_dim, &vertices);

        let right: Vec<(usize, Position)> = vertices
            .iter()
            .enumerate()
            .filter(|&n| {
                let (_, pos) = n;
                pos[largest_dim] >= median
            })
            .map(|(i, pos)| (i, pos.clone()))
            .collect();
        let left: Vec<(usize, Position)> = vertices
            .iter()
            .enumerate()
            .filter(|&n| {
                let (_, pos) = n;
                pos[largest_dim] < median
            })
            .map(|(i, pos)| (i, pos.clone()))
            .collect();

        let (left_bb, right_bb) = bb.split(largest_dim, median).unwrap();

        KdTree::new_node(
            bb,
            Some(Box::from(KdTree::from_vertices_internal(left_bb, left))),
            Some(Box::from(KdTree::from_vertices_internal(right_bb, right))),
        )
    }

    fn from_vertices_internal(
        bb: AxisAlignedBoundingBox,
        index_vertices_pairs: Vec<(usize, Position)>,
    ) -> KdTree {
        // Terminal condition
        if index_vertices_pairs.len() < 10 {
            return KdTree::new_leaf(
                bb,
                Some(
                    index_vertices_pairs
                        .iter()
                        .map(|(i, _)| i.clone())
                        .collect(),
                ),
            );
        }

        let largest_dim = bb.largest_dim();
        let vertices = index_vertices_pairs
            .iter()
            .map(|(_, pos)| pos.clone())
            .collect();
        let median = get_median(largest_dim, &vertices);

        let right: Vec<(usize, Position)> = index_vertices_pairs
            .iter()
            .filter(|&n| {
                let (_, pos) = n;
                pos[largest_dim] >= median
            })
            .map(|(i, pos)| (i.clone(), pos.clone()))
            .collect();
        let left: Vec<(usize, Position)> = index_vertices_pairs
            .iter()
            .filter(|&n| {
                let (_, pos) = n;
                pos[largest_dim] < median
            })
            .map(|(i, pos)| (i.clone(), pos.clone()))
            .collect();

        let (left_bb, right_bb) = bb.split(largest_dim, median).unwrap();

        KdTree::new_node(
            bb,
            Some(Box::from(KdTree::from_vertices_internal(left_bb, left))),
            Some(Box::from(KdTree::from_vertices_internal(right_bb, right))),
        )
    }

    pub fn is_leaf(&self) -> bool {
        self.vertices_index.is_some()
    }
}

fn get_median(dim: usize, vertices: &Vec<Position>) -> f64 {
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
pub struct BoxIntersectIter<'a> {
    ray: &'a Ray,
    next_nodes: BinaryHeap<BoxIntersect<'a>>,
}

impl<'a, 'b> BoxIntersectIter<'a> {
    pub fn new(ray: &'a Ray, first_node: &'a Box<KdTree>) -> BoxIntersectIter<'a> {
        let mut heap = BinaryHeap::new();
        let hit = ray.intersect_box(&(*first_node).bounding_box.bounds);
        if hit.is_some() {
            heap.push(BoxIntersect {
                distance: hit.unwrap(),
                node: first_node,
            })
        }
        BoxIntersectIter {
            ray: ray,
            next_nodes: heap,
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

impl<'a> Iterator for BoxIntersectIter<'a> {
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
        let hit_left = self.ray.intersect_box(&(*left_child).bounding_box.bounds);
        let hit_right = self.ray.intersect_box(&(*right_child).bounding_box.bounds);

        match (hit_left, hit_right) {
            (None, None) => {
                println!("Problem with parent box spliting");
                return None;
            }
            (Some(distance_left), None) => {
                self.next_nodes.push(BoxIntersect {
                    distance: distance_left,
                    node: &left_child,
                });
            }
            (None, Some(distance_right)) => {
                self.next_nodes.push(BoxIntersect {
                    distance: distance_right,
                    node: &right_child,
                });
            }
            (Some(distance_left), Some(distance_right)) => {
                self.next_nodes.push(BoxIntersect {
                    distance: distance_left,
                    node: &left_child,
                });
                self.next_nodes.push(BoxIntersect {
                    distance: distance_right,
                    node: &right_child,
                });
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
