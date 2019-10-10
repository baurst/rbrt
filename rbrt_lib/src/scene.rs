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
    pub fn hit<'a>(&'a self, ray: &Ray, min_dist: f64, max_dist: f64) -> Option<HitInformation> {
        let mut closest_hit_rec = None;
        let mut closest_so_far = std::f64::MAX;

        for sphere in &self.elements {
            let hit_info_op = sphere.intersect_with_ray(&ray, min_dist, max_dist);
            if hit_info_op.is_some() {
                let hit_rec = hit_info_op.unwrap();
                if hit_rec.dist_from_ray_orig < closest_so_far {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    closest_hit_rec = Some(hit_rec);
                }
            }
        }

        for mesh in &self.triangle_meshes {
            let hit_info_op = mesh.intersect_with_ray(&ray, min_dist, max_dist);
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
