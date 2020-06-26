extern crate rand;
use crate::geometry::bounding_box::AxisAlignedBoundingBox;
use crate::geometry::ray::Ray;
use crate::geometry::types::Position;
use rand::Rng;

pub struct KdTree {
    pub bounding_box: AxisAlignedBoundingBox,
    left: Option<Box<KdTree>>,
    right: Option<Box<KdTree>>,

    // leaf
    vertices_index: Option<Vec<usize>>,

    // Debug
    pub color: [u8; 3],
}

fn make_rng_color() -> [u8; 3] {
    let mut rng = rand::thread_rng();
    [rng.gen(), rng.gen(), rng.gen()]
}

impl KdTree {
    pub fn from_vertices(vertices: &Vec<Position>) -> KdTree {
        let bb = AxisAlignedBoundingBox::new(vertices);

        let largest_dim = bb.largest_dim();
        let median = get_median(largest_dim, &vertices);

        println!("median: dim {:?}, median {:?}", largest_dim, median);

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

        println!("{:} in right", right.len());
        println!("{:} in left", left.len());

        KdTree {
            bounding_box: bb,
            left: Some(Box::from(KdTree::from_vertices_internal(left_bb, left))),
            right: Some(Box::from(KdTree::from_vertices_internal(right_bb, right))),
            vertices_index: None,
            color: make_rng_color(),
        }
    }

    fn from_vertices_internal(
        bb: AxisAlignedBoundingBox,
        index_vertices_pairs: Vec<(usize, Position)>,
    ) -> KdTree {
        println!("children box is: {:?}", bb.bounds);
        // Terminal condition
        if index_vertices_pairs.len() < 10 {
            return KdTree {
                bounding_box: bb,
                left: None,
                right: None,
                vertices_index: Some(
                    index_vertices_pairs
                        .iter()
                        .map(|(i, _)| i.clone())
                        .collect(),
                ),
                color: make_rng_color(),
            };
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

        KdTree {
            bounding_box: bb,
            left: Some(Box::from(KdTree::from_vertices_internal(left_bb, left))),
            right: Some(Box::from(KdTree::from_vertices_internal(right_bb, right))),
            vertices_index: None,
            color: make_rng_color(),
        }
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

pub struct BoxIntersectIter<'a, 'b> {
    ray: &'a Ray,
    next_node: Option<&'b Box<KdTree>>,
}

impl<'a, 'b> BoxIntersectIter<'a, 'b> {
    pub fn new(ray: &'a Ray, first_node: &'b Box<KdTree>) -> BoxIntersectIter<'a, 'b> {
        let hit = ray.intersect_box(&(*first_node).bounding_box.bounds);
        if hit.is_some() {
            return BoxIntersectIter {
                ray: ray,
                next_node: Some(first_node),
            };
        } else {
            return BoxIntersectIter {
                ray: ray,
                next_node: None,
            };
        }
    }
}

impl<'a, 'b> Iterator for BoxIntersectIter<'a, 'b> {
    type Item = &'b Box<KdTree>;

    fn next(&mut self) -> Option<&'b Box<KdTree>> {
        if self.next_node.is_none() {
            return None;
        }

        let cur_node = self.next_node.unwrap();

        // We have reached a leaf we can stop
        if cur_node.is_leaf() {
            self.next_node = None;
            return Some(cur_node);
        }

        // Otherwise let's check which child is the next node
        // before returning the node
        let left_child = (*cur_node).left.as_ref().unwrap();
        let right_child = (*cur_node).right.as_ref().unwrap();
        let hit_left = self.ray.intersect_box(&(*left_child).bounding_box.bounds);
        let hit_right = self.ray.intersect_box(&(*right_child).bounding_box.bounds);

        if hit_right.is_none() && hit_left.is_none() {
            return None;
        }

        match (hit_left, hit_right) {
            (None, None) => {
                println!("Problem with parent box spliting");
                return None;
            }
            (Some(_), None) => {
                self.next_node = Some(&left_child);
            }
            (None, Some(_)) => {
                self.next_node = Some(&right_child);
            }
            (Some(i), Some(j)) => {
                if i < j {
                    self.next_node = Some(&left_child);
                } else {
                    self.next_node = Some(&right_child);
                }
            }
        }

        return Some(cur_node);
    }
}
