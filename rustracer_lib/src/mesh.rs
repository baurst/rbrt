extern crate tobj;
use std::path::Path;

use crate::lambertian::Lambertian;
use crate::triangle::Triangle;
use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray};

/// Axis aligned Bounding Box
pub struct BoundingBox {
    pub lower_bound: Vec3,
    pub upper_bound: Vec3,
}

/// just for better readabilty
pub fn max(a: f64, b: f64) -> f64 {
    return a.max(b);
}

pub fn min(a: f64, b: f64) -> f64 {
    return a.min(b);
}

impl BoundingBox {
    pub fn new(lower_bound: Vec3, upper_bound: Vec3) -> BoundingBox {
        return BoundingBox {
            lower_bound: lower_bound,
            upper_bound: upper_bound,
        };
    }
    /// see https://gamedev.stackexchange.com/questions/18436/most-efficient-aabb-vs-ray-collision-algorithms
    pub fn hit(&self, ray: &Ray) -> bool {
        let t1 = (self.lower_bound.x - ray.origin.x) / ray.direction.x;
        let t2 = (self.upper_bound.x - ray.origin.x) / ray.direction.x;
        let t3 = (self.lower_bound.y - ray.origin.y) / ray.direction.y;
        let t4 = (self.upper_bound.y - ray.origin.y) / ray.direction.y;
        let t5 = (self.lower_bound.z - ray.origin.z) / ray.direction.z;
        let t6 = (self.upper_bound.z - ray.origin.z) / ray.direction.z;

        let tmin = max(max(min(t1, t2), min(t3, t4)), min(t5, t6));
        let tmax = min(min(max(t1, t2), max(t3, t4)), max(t5, t6));

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        if tmax < 0.0 {
            return false;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            return false;
        }
        return true;
    }
}

pub struct TriangleMesh {
    pub triangles: Vec<Box<Triangle>>,
    pub bbox: BoundingBox,
}

impl TriangleMesh {
    pub fn new(filepath: &str, translation: Vec3, scale: f64) -> TriangleMesh {
        let mesh = load_mesh_from_file(filepath, translation, scale);
        let (mesh, lower_bound, upper_bound) = compute_min_max_3d(mesh);

        return TriangleMesh {
            triangles: mesh,
            bbox: BoundingBox::new(lower_bound, upper_bound),
        };
    }
}

pub fn compute_min_max_3d(triangle_mesh: Vec<Box<Triangle>>) -> (Vec<Box<Triangle>>, Vec3, Vec3) {
    let mut lower_bound_tmp = Vec3::new(std::f64::MAX, std::f64::MAX, std::f64::MAX);
    let mut upper_bound_tmp = Vec3::new(-std::f64::MAX, -std::f64::MAX, -std::f64::MAX);
    // wow that code stinks!
    for tri in &triangle_mesh {
        for corner in &tri.corners {
            if corner.x < lower_bound_tmp.x {
                lower_bound_tmp.x = corner.x;
            }
            if corner.y < lower_bound_tmp.y {
                lower_bound_tmp.y = corner.y;
            }
            if corner.z < lower_bound_tmp.z {
                lower_bound_tmp.z = corner.z;
            }
            if corner.x > upper_bound_tmp.x {
                upper_bound_tmp.x = corner.x;
            }
            if corner.y > upper_bound_tmp.y {
                upper_bound_tmp.y = corner.y;
            }
            if corner.z > upper_bound_tmp.z {
                upper_bound_tmp.z = corner.z;
            }
        }
    }
    (triangle_mesh, lower_bound_tmp, upper_bound_tmp)
}

/// Loads mesh from obj file, scales and translates it
pub fn load_mesh_from_file(filepath: &str, translation: Vec3, scale: f64) -> Vec<Box<Triangle>> {
    let mut model_elements: Vec<Box<Triangle>> = Vec::new();

    let loaded_mesh = tobj::load_obj(&Path::new(filepath));
    assert!(loaded_mesh.is_ok());
    let (models, _materials) = loaded_mesh.unwrap();

    for (_i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        assert!(mesh.positions.len() % 3 == 0);
        let mut triangle_vertices: Vec<Vec3> = vec![Vec3::zero(); 3];
        for f in 0..mesh.indices.len() / 3 {
            for idx in 0..3 {
                let x_idx = 3 * mesh.indices[3 * f + idx];
                let y_idx = 3 * mesh.indices[3 * f + idx] + 1;
                let z_idx = 3 * mesh.indices[3 * f + idx] + 2;

                triangle_vertices[idx] = Vec3::new(
                    mesh.positions[x_idx as usize] as f64 * scale,
                    mesh.positions[y_idx as usize] as f64 * scale,
                    mesh.positions[z_idx as usize] as f64 * scale,
                );
            }
            let tri = Box::new(Triangle {
                corners: [
                    triangle_vertices[0] + translation,
                    triangle_vertices[1] + translation,
                    triangle_vertices[2] + translation,
                ],
                material: Box::new(Lambertian {
                    albedo: Vec3::new(0.8, 0.1, 0.1),
                }),
            });
            model_elements.push(tri);
        }
    }
    println!(
        "Loaded {} triangles from file {}!",
        model_elements.len(),
        filepath
    );
    return model_elements;
}

impl Intersectable for TriangleMesh {
    fn intersect_with_ray<'a>(
        &'a self,
        ray: &Ray,
        min_dist: f64,
        max_dist: f64,
    ) -> Option<HitInformation> {
        // first check if bounding box is hit
        if !self.bbox.hit(ray) {
            return None;
        }
        // if bounding box is hit, check all triangles
        let mut closest_hit_rec = None;
        let mut closest_so_far = std::f64::MAX;

        for triangle in &self.triangles {
            let hit_info_op = triangle.intersect_with_ray(&ray, min_dist, max_dist);
            if hit_info_op.is_some() {
                let hit_rec = hit_info_op.unwrap();
                if hit_rec.dist_from_ray_orig < closest_so_far {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    closest_hit_rec = Some(hit_rec);
                }
            }
        }

        return closest_hit_rec;
    }
}
