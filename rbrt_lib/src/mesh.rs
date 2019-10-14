extern crate tobj;
use std::path::Path;

use crate::aabbox::{compute_min_max_3d, BoundingBox};
use crate::lambertian::Lambertian;
use crate::triangle::{
    get_triangle_normal, triangle_soa_intersect_with_ray, triangle_soa_sse_intersect_with_ray,
    BasicTriangle,
};
use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};

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
        scale: f32,
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

/// Loads mesh from obj file, scales and translates it
pub fn load_mesh_vertices_from_file(
    filepath: &str,
    translation: Vec3,
    rotation: Vec3,
    scale: f32,
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
                    mesh.positions[x_idx as usize] as f32 * scale,
                    mesh.positions[y_idx as usize] as f32 * scale,
                    mesh.positions[z_idx as usize] as f32 * scale,
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
    scale: f32,
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
                    mesh.positions[x_idx as usize] as f32 * scale,
                    mesh.positions[y_idx as usize] as f32 * scale,
                    mesh.positions[z_idx as usize] as f32 * scale,
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
        min_dist: f32,
        max_dist: f32,
    ) -> Option<HitInformation> {
        // first check if bounding box is hit
        if !self.bbox.hit(ray) {
            return None;
        }

        let mut hit_occured = false;
        let mut closest_ray_param = std::f32::MAX;
        // saving the normal here apparently prevents a cache miss later on
        let mut closes_hit_normal = Vec3::zero();
        unsafe {
            let (hit_info_op, hit_idx_op) = triangle_soa_sse_intersect_with_ray(
                &ray,
                &self.vertices,
                &self.edges,
                min_dist,
                max_dist,
            );
            if hit_info_op.is_some() && hit_idx_op.is_some() {
                let ray_param_cand = hit_info_op.unwrap();
                let triangle_idx = hit_idx_op.unwrap();
                if ray_param_cand < closest_ray_param {
                    closest_ray_param = ray_param_cand;
                    hit_occured = true;
                    closes_hit_normal = self.normals[triangle_idx as usize];
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
}
