use crate::materials::random_point_in_unit_sphere;
use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

#[derive(Copy, Clone, Debug)]
pub struct Lambertian {
    pub albedo: Vec3,
}

impl RayScattering for Lambertian {
    fn scatter(
        &self,
        _incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool {
        let scattered_ray_target_point =
            hit_info.hit_point + hit_info.hit_normal + random_point_in_unit_sphere();
        scattered_ray.direction = (scattered_ray_target_point - hit_info.hit_point).normalize();
        scattered_ray.origin = hit_info.hit_point;
        *attentuation = self.albedo;
        return true;
    }
}
