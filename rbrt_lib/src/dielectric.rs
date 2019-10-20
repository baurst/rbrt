use crate::materials::reflect;
use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    pub ref_idx: f32,
}

impl RayScattering for Dielectric {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool {
        *attentuation = Vec3::new(1.0, 1.0, 1.0);
        let reflected_ray_dir = reflect(&incoming_ray.direction, &hit_info.hit_normal);
        let outward_normal;
        let ni_over_nt;
        let cosine;
        let reflect_prob;
        let a = incoming_ray
            .direction
            .normalize()
            .dot(&hit_info.hit_normal.normalize());
        if a > 0.0 {
            outward_normal = -1.0 * hit_info.hit_normal;
            ni_over_nt = self.ref_idx;
            cosine = self.ref_idx * a;
        } else {
            outward_normal = hit_info.hit_normal;
            ni_over_nt = 1.0 / self.ref_idx;
            cosine = -a;
        }

        let mut refracted_ray_dir = Vec3::zero();
        if refract(
            &incoming_ray.direction,
            &outward_normal,
            ni_over_nt,
            &mut refracted_ray_dir,
        ) {
            reflect_prob = schlick(cosine, self.ref_idx);
        } else {
            reflect_prob = 1.0;
        }
        if rand::random::<f32>() < reflect_prob {
            *scattered_ray = Ray {
                origin: hit_info.hit_point,
                direction: reflected_ray_dir,
            };
        } else {
            *scattered_ray = Ray {
                origin: hit_info.hit_point,
                direction: refracted_ray_dir,
            };
        }
        return true;
    }
}

pub fn schlick(cosine: f32, ref_index: f32) -> f32 {
    let r0 = ((1.0 - ref_index) / (1.0 + ref_index)).powi(2);
    return r0 + (1.0 - r0) * (1.0 - cosine).powi(5);
}

pub fn refract(
    incoming_ray_dir: &Vec3,
    normal: &Vec3,
    ni_over_nt: f32,
    refracted_ray_dir: &mut Vec3,
) -> bool {
    let view_unit = incoming_ray_dir.normalize();
    let normal_unit = normal.normalize();

    let cos_theta = view_unit.dot(&normal_unit);
    let discr = 1.0 - ni_over_nt.powi(2) * (1.0 - cos_theta.powi(2));
    if discr > 0.0 {
        *refracted_ray_dir =
            ni_over_nt * (view_unit - normal_unit * cos_theta) - discr.sqrt() * normal_unit;
        return true;
    }
    return false;
}

#[cfg(test)]
mod test {
    use crate::dielectric::refract;
    use crate::vec3::Vec3;

    #[test]
    fn test_refraction() {
        let incoming = Vec3::new(1.0, 1.0, 0.0);
        let normal = Vec3::new(-1.0, 0.0, 0.0);
        let mut refr_ray_dir = Vec3::zero();
        let ni_over_nt = 1.4;

        let refracted = refract(
            &incoming.normalize(),
            &normal.normalize(),
            ni_over_nt,
            &mut refr_ray_dir,
        );

        assert!(refracted);
        assert_eq!(
            refr_ray_dir,
            Vec3 {
                x: 0.14142191,
                y: 0.9899495,
                z: 0.0
            }
        );
    }
}
