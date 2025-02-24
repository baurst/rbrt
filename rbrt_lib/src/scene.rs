use crate::mesh::TriangleMesh;
use crate::vec3::Vec3;
use crate::HitInformation;
use crate::Intersectable;
use crate::Ray;

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct Scene {
    pub elements: Vec<Box<dyn Intersectable + Sync>>,
    pub triangle_meshes: Vec<TriangleMesh>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn hit(&self, ray: &Ray, min_dist: f32, max_dist: f32) -> Option<HitInformation> {
        let mut closest_hit_rec = None;
        let mut closest_so_far = f32::MAX;

        for sphere in &self.elements {
            let hit_info_op = sphere.intersect_with_ray(ray, min_dist, max_dist);
            if let Some(hit_rec) = hit_info_op {
                if hit_rec.dist_from_ray_orig < closest_so_far {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    closest_hit_rec = Some(hit_rec);
                }
            }
        }

        for mesh in &self.triangle_meshes {
            let hit_info_op = mesh.intersect_with_ray(ray, min_dist, max_dist);
            if let Some(hit_rec) = hit_info_op {
                if hit_rec.dist_from_ray_orig < closest_so_far {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    closest_hit_rec = Some(hit_rec);
                }
            }
        }
        closest_hit_rec
    }
}
