use crate::vec3::Vec3;
use crate::{HitInformation, Ray};

pub trait RayScattering {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool;
}

#[derive(Copy, Clone, Debug)]
pub struct Lambertian {
    pub albedo: Vec3,
}

impl RayScattering for Lambertian {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool {
        let scattered_ray_target = hit_info.hit_normal + random_point_in_unit_sphere();
        scattered_ray.direction = scattered_ray_target.normalize();
        scattered_ray.origin = hit_info.hit_point;
        *attentuation = self.albedo;
        return true;
    }
}

pub fn random_point_in_unit_sphere() -> Vec3 {
    let mut point = Vec3::new(
        rand::random::<f64>(),
        rand::random::<f64>(),
        rand::random::<f64>(),
    );
    while point.length() > 1.0 {
        point = Vec3::new(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        );
    }
    return point;
}

pub fn reflect(incoming_ray_dir: &Vec3, normal: &Vec3) -> Vec3 {
    let reflected_dir = *incoming_ray_dir - 2.0 * (*normal) * incoming_ray_dir.dot(normal);
    return reflected_dir.normalize();
}

pub fn schlick(cosine: f64, ref_index: f64) -> f64 {
    let r0 = ((1.0 - ref_index) / (1.0 - ref_index)).powf(2.0);
    return r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0);
}

pub fn refract(
    incoming_ray_dir: &Vec3,
    normal: &Vec3,
    ni_over_nt: f64,
    refracted_ray_dir: &mut Vec3,
) -> bool {
    let view = incoming_ray_dir.normalize();
    let cos_theta = view.dot(&normal.normalize());
    let discr = 1.0 - ni_over_nt.powf(2.0) * (1.0 - cos_theta.powf(2.0));
    if discr > 0.0 {
        *refracted_ray_dir = ni_over_nt * (*&view - *normal * cos_theta) - discr.sqrt() * *normal;
        return true;
    }
    return false;
}

#[derive(Copy, Clone, Debug)]
pub struct Metal {
    albedo: Vec3,
}

impl RayScattering for Metal {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool {
        let scattered_ray_target = reflect(&incoming_ray.direction, &hit_info.hit_normal);
        scattered_ray.direction = scattered_ray_target.normalize();
        scattered_ray.origin = hit_info.hit_point;
        *attentuation = self.albedo;
        return true;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    albedo: Vec3,
}

impl RayScattering for Dielectric {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool {
        return true;
    }
}
