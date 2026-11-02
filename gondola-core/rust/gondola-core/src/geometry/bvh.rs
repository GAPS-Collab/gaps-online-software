//! Axis aligned bounding boxes and a bounding volume
//! hierarchy (BVH) over them, to find the primitives
//! (triangles, placements) a line passes close to.
// This file is part of gaps-online-software and published
// under the GPLv3 license

use glam::DVec3;

use crate::geometry::model::Transform;

/// Axis aligned bounding box
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb {
  pub min : DVec3,
  pub max : DVec3,
}

impl Aabb {
  /// A box which contains nothing, it can be grown
  /// with grow and merge
  pub const EMPTY : Self = Self {
    min : DVec3::splat(f64::INFINITY),
    max : DVec3::splat(f64::NEG_INFINITY),
  };

  pub fn from_points(points : impl IntoIterator<Item = DVec3>) -> Self {
    let mut aabb = Self::EMPTY;
    for p in points {
      aabb.grow(p);
    }
    aabb
  }

  pub fn is_empty(&self) -> bool {
    self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
  }

  pub fn grow(&mut self, p : DVec3) {
    self.min = self.min.min(p);
    self.max = self.max.max(p);
  }

  pub fn merge(&mut self, other : &Aabb) {
    self.min = self.min.min(other.min);
    self.max = self.max.max(other.max);
  }

  pub fn intersection(&self, other : &Aabb) -> Aabb {
    Aabb {
      min : self.min.max(other.min),
      max : self.max.min(other.max),
    }
  }

  /// The box grown by margin in all directions
  pub fn expanded(&self, margin : f64) -> Aabb {
    Aabb {
      min : self.min - DVec3::splat(margin),
      max : self.max + DVec3::splat(margin),
    }
  }

  pub fn center(&self) -> DVec3 {
    0.5*(self.min + self.max)
  }

  pub fn surface_area(&self) -> f64 {
    if self.is_empty() {
      return 0.0;
    }
    let d = self.max - self.min;
    2.0*(d.x*d.y + d.y*d.z + d.z*d.x)
  }

  /// The bounding box (in the parent frame) of this box
  /// (in the local frame) after the transformation
  pub fn transformed(&self, t : &Transform) -> Aabb {
    if self.is_empty() {
      return *self;
    }
    let corners = (0..8).map(|k| DVec3::new(
      if k & 1 == 0 { self.min.x } else { self.max.x },
      if k & 2 == 0 { self.min.y } else { self.max.y },
      if k & 4 == 0 { self.min.z } else { self.max.z },
    ));
    Aabb::from_points(corners.map(|c| t.apply_point(c)))
  }

  /// The range [t0, t1] (within [tmin, tmax]) of the
  /// parameter t for which origin + t*dir is in the box,
  /// None if the line misses the box
  pub fn clip_line(&self, origin : DVec3, dir : DVec3, tmin : f64, tmax : f64) -> Option<(f64, f64)> {
    let (mut t0, mut t1) = (tmin, tmax);
    for k in 0..3 {
      if dir[k] == 0.0 {
        if origin[k] < self.min[k] || origin[k] > self.max[k] {
          return None;
        }
        continue;
      }
      let inv = 1.0/dir[k];
      let (mut ta, mut tb) = ((self.min[k] - origin[k])*inv, (self.max[k] - origin[k])*inv);
      if ta > tb {
        std::mem::swap(&mut ta, &mut tb);
      }
      t0 = t0.max(ta);
      t1 = t1.min(tb);
      if t0 > t1 {
        return None;
      }
    }
    Some((t0, t1))
  }
}

//------------------------------------------------------------

/// Number of primitives up to which a node is not split
const MAX_LEAF_SIZE : usize = 4;
/// Number of bins for the surface area heuristic
const N_BINS        : usize = 16;

#[derive(Debug, Clone, PartialEq)]
struct BvhNode {
  aabb  : Aabb,
  /// leaf: index of the first primitive in Bvh::indices,
  /// inner node: index of the left child (the right one
  /// follows it)
  first : u32,
  /// number of primitives, 0 for inner nodes
  count : u32,
}

/// Bounding volume hierarchy over a set of boxes, built
/// with the binned surface area heuristic
#[derive(Debug, Clone, PartialEq)]
pub struct Bvh {
  nodes   : Vec<BvhNode>,
  /// primitive indices, ordered such that every leaf
  /// refers to a contiguous range
  indices : Vec<u32>,
}

impl Bvh {
  pub fn build(boxes : &[Aabb]) -> Self {
    let mut bvh = Self {
      nodes   : Vec::with_capacity(2*boxes.len()/MAX_LEAF_SIZE + 1),
      indices : (0..boxes.len() as u32).collect(),
    };
    let centers : Vec<DVec3> = boxes.iter().map(|b| b.center()).collect();
    bvh.nodes.push(BvhNode { aabb : Aabb::EMPTY, first : 0, count : boxes.len() as u32 });
    bvh.subdivide(0, boxes, &centers);
    bvh
  }

  pub fn bounds(&self) -> Aabb {
    self.nodes[0].aabb
  }

  fn subdivide(&mut self, node : usize, boxes : &[Aabb], centers : &[DVec3]) {
    let first = self.nodes[node].first as usize;
    let count = self.nodes[node].count as usize;
    let range = first..first + count;
    let mut aabb     = Aabb::EMPTY;
    let mut centroid = Aabb::EMPTY;
    for &i in &self.indices[range.clone()] {
      aabb.merge(&boxes[i as usize]);
      centroid.grow(centers[i as usize]);
    }
    self.nodes[node].aabb = aabb;
    if count <= MAX_LEAF_SIZE {
      return;
    }
    let Some((axis, split)) = Self::find_split(&self.indices[range.clone()], boxes, centers, &centroid, aabb.surface_area()*count as f64) else {
      return;
    };
    // partition the primitives in place
    let (mut i, mut j) = (first, first + count);
    while i < j {
      if centers[self.indices[i] as usize][axis] < split {
        i += 1;
      } else {
        j -= 1;
        self.indices.swap(i, j);
      }
    }
    let n_left = i - first;
    if n_left == 0 || n_left == count {
      return;
    }
    let left = self.nodes.len();
    self.nodes.push(BvhNode { aabb : Aabb::EMPTY, first : first as u32, count : n_left as u32 });
    self.nodes.push(BvhNode { aabb : Aabb::EMPTY, first : i as u32, count : (count - n_left) as u32 });
    self.nodes[node].first = left as u32;
    self.nodes[node].count = 0;
    self.subdivide(left, boxes, centers);
    self.subdivide(left + 1, boxes, centers);
  }

  /// Best (axis, position) to split the primitives at,
  /// None if no split is cheaper than a leaf (leaf_cost)
  fn find_split(indices : &[u32], boxes : &[Aabb], centers : &[DVec3], centroid : &Aabb, leaf_cost : f64) -> Option<(usize, f64)> {
    let mut best : Option<(usize, f64)> = None;
    let mut best_cost = leaf_cost;
    for axis in 0..3 {
      let (lo, hi) = (centroid.min[axis], centroid.max[axis]);
      if hi <= lo {
        continue;
      }
      let scale = N_BINS as f64/(hi - lo);
      let mut bins = [(Aabb::EMPTY, 0usize); N_BINS];
      for &i in indices {
        let b = (((centers[i as usize][axis] - lo)*scale) as usize).min(N_BINS - 1);
        bins[b].0.merge(&boxes[i as usize]);
        bins[b].1 += 1;
      }
      // cost of splitting after bin k, from sweeps from
      // the left and from the right
      let mut left_cost = [0.0; N_BINS - 1];
      let (mut aabb, mut n) = (Aabb::EMPTY, 0);
      for k in 0..N_BINS - 1 {
        aabb.merge(&bins[k].0);
        n += bins[k].1;
        left_cost[k] = aabb.surface_area()*n as f64;
      }
      let (mut aabb, mut n) = (Aabb::EMPTY, 0);
      for k in (1..N_BINS).rev() {
        aabb.merge(&bins[k].0);
        n += bins[k].1;
        let cost = left_cost[k - 1] + aabb.surface_area()*n as f64;
        if cost < best_cost {
          best_cost = cost;
          best      = Some((axis, lo + k as f64/scale));
        }
      }
    }
    best
  }

  /// Call f for every primitive whose leaf box is hit by
  /// the line origin + t*dir with t in [tmin, tmax]. A
  /// primitive is reported at most once.
  pub fn traverse(&self, origin : DVec3, dir : DVec3, tmin : f64, tmax : f64, mut f : impl FnMut(u32)) {
    if self.indices.is_empty() {
      return;
    }
    let mut stack = vec![0usize];
    while let Some(k) = stack.pop() {
      let node = &self.nodes[k];
      if node.aabb.clip_line(origin, dir, tmin, tmax).is_none() {
        continue;
      }
      if node.count > 0 {
        let first = node.first as usize;
        for &i in &self.indices[first..first + node.count as usize] {
          f(i);
        }
      } else {
        stack.push(node.first as usize);
        stack.push(node.first as usize + 1);
      }
    }
  }
}

//------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clip_line() {
    let b = Aabb { min : DVec3::splat(-1.0), max : DVec3::splat(1.0) };
    assert_eq!(b.clip_line(DVec3::new(-5.0, 0.0, 0.0), DVec3::X, f64::NEG_INFINITY, f64::INFINITY), Some((4.0, 6.0)));
    assert_eq!(b.clip_line(DVec3::new(-5.0, 0.0, 0.0), DVec3::X, 0.0, 4.5), Some((4.0, 4.5)));
    assert_eq!(b.clip_line(DVec3::new(-5.0, 2.0, 0.0), DVec3::X, f64::NEG_INFINITY, f64::INFINITY), None);
  }

  #[test]
  fn traverse_finds_all_hit_boxes() {
    // a row of unit cubes along x and one off the line
    let mut boxes : Vec<Aabb> = (0..100).map(|i| Aabb {
      min : DVec3::new(i as f64, 0.0, 0.0),
      max : DVec3::new(i as f64 + 1.0, 1.0, 1.0),
    }).collect();
    boxes.push(Aabb { min : DVec3::new(50.0, 50.0, 0.0), max : DVec3::new(51.0, 51.0, 1.0) });
    let bvh = Bvh::build(&boxes);
    let mut hits = Vec::new();
    bvh.traverse(DVec3::new(-1.0, 0.5, 0.5), DVec3::X, 0.0, 10.5, |i| hits.push(i));
    // the leaves may contain more boxes than were hit, but
    // never miss one
    hits.sort();
    for i in 0..10 {
      assert!(hits.contains(&i));
    }
    assert!(!hits.contains(&100));
  }
}
