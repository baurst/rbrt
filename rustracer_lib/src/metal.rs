use crate::materials::{random_point_in_unit_sphere, reflect};
use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

#[derive(Copy, Clone, Debug)]
pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f64,
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
        scattered_ray.direction = (scattered_ray_target.normalize()
            + self.fuzz * random_point_in_unit_sphere())
        .normalize();
        scattered_ray.origin = hit_info.hit_point;
        *attentuation = self.albedo;
        // TODO FIX THIS, why does this sometimes not reflect!?
        //let reflect = scattered_ray.direction.dot(&hit_info.hit_normal) > 0.0;
        /*
        if !reflect{
            println!("Metal did not reflect ray!");
        }
        else{
            println!("Metal did reflect ray!");
        }
        */
        return true;
    }
}
