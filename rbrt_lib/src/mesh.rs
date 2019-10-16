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
    // 3 vertices with 3 coords (x,y,z) each
    pub vertices: [[Vec<f32>; 3]; 3],
    // 2 edges with 3 coords (x,y,z) each
    pub edges: [[Vec<f32>; 3]; 2],
    // 1 normal with 3 coords (x,y,z) each
    pub normals: [Vec<f32>; 3],

    //pub is_padding_triangle: Vec<bool>,
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
        let pre_vertices = load_mesh_vertices_from_file(filepath, translation, rotation, scale);

        //   let num_lanes = 4;
        // let num_triangles =

        let mut pre_normals = vec![];
        for triangle_vertices in &pre_vertices {
            pre_normals.push(get_triangle_normal(&triangle_vertices));
        }

        let mut pre_edges = vec![];
        for triangle_vertices in &pre_vertices {
            pre_edges.push([
                triangle_vertices[1] - triangle_vertices[0],
                triangle_vertices[2] - triangle_vertices[0],
            ]);
        }

        let (lower_bound, upper_bound) = compute_min_max_3d(&pre_vertices);
        let mut vertices: [[Vec<f32>; 3]; 3] = [
            [vec![], vec![], vec![]],
            [vec![], vec![], vec![]],
            [vec![], vec![], vec![]],
        ];

        for vertex in pre_vertices {
            vertices[0][0].push(vertex[0].x);
            vertices[0][1].push(vertex[0].y);
            vertices[0][2].push(vertex[0].z);
            vertices[1][0].push(vertex[1].x);
            vertices[1][1].push(vertex[1].y);
            vertices[1][2].push(vertex[1].z);
            vertices[2][0].push(vertex[2].x);
            vertices[2][1].push(vertex[2].y);
            vertices[2][2].push(vertex[2].z);
        }
        let mut edges: [[Vec<f32>; 3]; 2] = [[vec![], vec![], vec![]], [vec![], vec![], vec![]]];
        for edge in pre_edges {
            edges[0][0].push(edge[0].x);
            edges[0][1].push(edge[0].y);
            edges[0][2].push(edge[0].z);
            edges[1][0].push(edge[1].x);
            edges[1][1].push(edge[1].y);
            edges[1][2].push(edge[1].z);
        }

        let mut normals: [Vec<f32>; 3] = [vec![], vec![], vec![]];
        for normal in pre_normals {
            normals[0].push(normal.x);
            normals[1].push(normal.y);
            normals[2].push(normal.z);
        }

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

        // saving the normal here apparently prevents a cache miss later on
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
                let hit_point = ray.point_at(ray_param_cand);
                let dist_from_ray_orig = (ray.origin - hit_point).length();
                if dist_from_ray_orig > min_dist && dist_from_ray_orig < max_dist {
                    let hit_idx = hit_idx_op.unwrap();
                    return Some(HitInformation {
                        hit_point: hit_point,
                        hit_normal: Vec3::new(
                            self.normals[0][hit_idx],
                            self.normals[1][hit_idx],
                            self.normals[2][hit_idx],
                        ),
                        hit_material: &*self.material,
                        dist_from_ray_orig: dist_from_ray_orig,
                    });
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
}
