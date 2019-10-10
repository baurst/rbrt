extern crate tobj;
use std::path::Path;

use crate::lambertian::Lambertian;
use crate::triangle::{get_triangle_normal, triangle_soa_intersect_with_ray, BasicTriangle};
use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};

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
    /// for explanation see https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
    /// see also https://gamedev.stackexchange.com/questions/18436/most-efficient-aabb-vs-ray-collision-algorithms
    pub fn hit(&self, ray: &Ray) -> bool {
        // get ray parameters that show where the ray intersects the box planes
        let t_lower_x = (self.lower_bound.x - ray.origin.x) / ray.direction.x;
        let t_upper_x = (self.upper_bound.x - ray.origin.x) / ray.direction.x;
        let t_lower_y = (self.lower_bound.y - ray.origin.y) / ray.direction.y;
        let t_upper_y = (self.upper_bound.y - ray.origin.y) / ray.direction.y;
        let t_lower_z = (self.lower_bound.z - ray.origin.z) / ray.direction.z;
        let t_upper_z = (self.upper_bound.z - ray.origin.z) / ray.direction.z;

        let t_min_x = min(t_lower_x, t_upper_x);
        let t_min_y = min(t_lower_y, t_upper_y);
        let t_min_z = min(t_lower_z, t_upper_z);
        // look for the biggest lower intersection across all dimensions
        let t_min = max(max(t_min_x, t_min_y), t_min_z);

        let t_max_x = max(t_lower_x, t_upper_x);
        let t_max_y = max(t_lower_y, t_upper_y);
        let t_max_z = max(t_lower_z, t_upper_z);
        // look for the smallest upper intersection across all dimensions
        let t_max = min(min(t_max_x, t_max_y), t_max_z);

        // intersection, but opposite to ray direction
        if t_max < 0.0 {
            return false;
        }

        if t_min > t_max {
            return false;
        }
        return true;
    }
}

pub struct TriangleMesh {
    pub vertices: Vec<[Vec3; 3]>,
    pub normals: Vec<Vec3>,
    pub edges: Vec<[Vec3; 2]>,

    pub bbox: BoundingBox,
    pub material: Box<dyn RayScattering + Sync>,
}

impl TriangleMesh {
    pub fn new(
        filepath: &str,
        translation: Vec3,
        rotation: Vec3,
        scale: f64,
        material: Box<dyn RayScattering + Sync>,
    ) -> TriangleMesh {
        let vertices = load_mesh_vertices_from_file(filepath, translation, rotation, scale);

        let mut normals = vec![];
        for triangle_vertices in &vertices {
            normals.push(get_triangle_normal(&triangle_vertices));
        }

        let mut edges = vec![];
        for triangle_vertices in &vertices {
            edges.push([
                triangle_vertices[1] - triangle_vertices[0],
                triangle_vertices[2] - triangle_vertices[0],
            ]);
        }

        let (lower_bound, upper_bound) = compute_min_max_3d(&vertices);

        return TriangleMesh {
            vertices: vertices,
            normals: normals,
            edges: edges,
            bbox: BoundingBox::new(lower_bound, upper_bound),
            material: material,
        };
    }
}

/// computes the axis aligned bounding box extents of triangles
pub fn compute_min_max_3d(triangle_mesh: &Vec<[Vec3; 3]>) -> (Vec3, Vec3) {
    let mut lower_bound_tmp = Vec3::new(std::f64::MAX, std::f64::MAX, std::f64::MAX);
    let mut upper_bound_tmp = Vec3::new(-std::f64::MAX, -std::f64::MAX, -std::f64::MAX);
    for tri in triangle_mesh {
        for idx in 0..3 {
            let vertex = tri[idx];
            if vertex.x < lower_bound_tmp.x {
                lower_bound_tmp.x = vertex.x;
            }
            if vertex.y < lower_bound_tmp.y {
                lower_bound_tmp.y = vertex.y;
            }
            if vertex.z < lower_bound_tmp.z {
                lower_bound_tmp.z = vertex.z;
            }
            if vertex.x > upper_bound_tmp.x {
                upper_bound_tmp.x = vertex.x;
            }
            if vertex.y > upper_bound_tmp.y {
                upper_bound_tmp.y = vertex.y;
            }
            if vertex.z > upper_bound_tmp.z {
                upper_bound_tmp.z = vertex.z;
            }
        }
    }
    (lower_bound_tmp, upper_bound_tmp)
}

/// Loads mesh from obj file, scales and translates it
pub fn load_mesh_vertices_from_file(
    filepath: &str,
    translation: Vec3,
    rotation: Vec3,
    scale: f64,
) -> Vec<[Vec3; 3]> {
    let mut model_vertices: Vec<[Vec3; 3]> = Vec::new();

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
            model_vertices.push([
                triangle_vertices[0].rotate_point(rotation) + translation,
                triangle_vertices[1].rotate_point(rotation) + translation,
                triangle_vertices[2].rotate_point(rotation) + translation,
            ]);
        }
    }
    println!(
        "Successfully loaded {} triangles from file {}!",
        model_vertices.len(),
        filepath
    );
    return model_vertices;
}

/// Loads mesh from obj file, scales and translates it
pub fn load_mesh_from_file(
    filepath: &str,
    translation: Vec3,
    rotation: Vec3,
    scale: f64,
    albedo: Vec3,
) -> Vec<Box<BasicTriangle>> {
    let mut model_elements: Vec<Box<BasicTriangle>> = Vec::new();

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
            let tri = Box::new(BasicTriangle::new(
                [
                    triangle_vertices[0].rotate_point(rotation) + translation,
                    triangle_vertices[1].rotate_point(rotation) + translation,
                    triangle_vertices[2].rotate_point(rotation) + translation,
                ],
                Box::new(Lambertian { albedo: albedo }),
            ));
            model_elements.push(tri);
        }
    }
    println!(
        "Successfully loaded {} triangles from file {}!",
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

        let mut hit_occured = false;
        let mut closest_ray_param = std::f64::MAX;
        // saving the normal here apparently prevents a cache miss later on
        let mut closes_hit_normal = Vec3::zero();
        for (triangle_idx, triangle_vertices) in self.vertices.iter().enumerate() {
            let hit_info_op = triangle_soa_intersect_with_ray(
                &ray,
                &triangle_vertices,
                &self.edges[triangle_idx],
                min_dist,
                max_dist,
            );

            if hit_info_op.is_some() {
                let ray_param_cand = hit_info_op.unwrap();
                if ray_param_cand < closest_ray_param {
                    closest_ray_param = ray_param_cand;
                    hit_occured = true;
                    closes_hit_normal = self.normals[triangle_idx];
                }
            }
        }

        if hit_occured {
            let hit_point = ray.point_at(closest_ray_param);
            let dist_from_ray_orig = (ray.origin - hit_point).length();

            return Some(HitInformation {
                hit_point: hit_point,
                hit_normal: closes_hit_normal,
                hit_material: &*self.material,
                dist_from_ray_orig: dist_from_ray_orig,
            });
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_min_max_3d, Vec3};

    #[test]
    fn test_mesh_aabbox() {
        let test_tri = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];

        let tris = vec![test_tri];

        let (lower_bound, upper_bound) = compute_min_max_3d(&tris);

        assert_eq!(lower_bound, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(upper_bound, Vec3::new(1.0, 1.0, 1.0));
    }
}
