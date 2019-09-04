use std::cmp::Ordering;

use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Box<dyn RayScattering + Sync>,
}

impl Sphere {
    ///
    /// Compute intersection of ray and sphere
    /// ray: r(t) = o + td
    /// sphere: (p-c)*(p-c) = r^2
    /// insert ray for p into sphere equation, then solve quadratic equation for t
    /// (o+td-c)(o+td-c)=r^2
    /// t1/2 = (-B +- sqrt(B^2 - 4AC))/(2A)
    ///
    /// Hitinformation has anonymous lifetime?
    pub fn intersect_with_ray<'a>(&'a self, ray: &Ray) -> Option<HitInformation> {
        let a = ray.direction.dot(&ray.direction);
        let l = ray.origin - self.center;
        let b = (ray.direction * 2.0).dot(&l);
        let c = l.dot(&l) - self.radius.powf(2.0);

        let sol = b.powf(2.0) - 4.0 * a * c;

        let num_hits = match sol.partial_cmp(&0.0).expect("Encountered NAN") {
            Ordering::Less => 0,
            Ordering::Greater => 2,
            Ordering::Equal => 1,
        };

        if num_hits == 0 {
            return None;
        } else {
            let mut ray_param = (-b - sol.sqrt()) / (2.0 * a);
            if num_hits == 2 && ray_param < 0.0 {
                //point is behind the camera!
                ray_param = (-b + sol.sqrt()) / (2.0 * a);
                if ray_param < 0.0 {
                    return None; // both points on the ray are negative
                }
            }

            let hit_point = ray.point_at(ray_param);
            let hit_normal = hit_point - self.center;
            let hit_info = HitInformation {
                hit_normal: hit_normal,
                hit_point: hit_point,
                hit_material: &*self.material,
                dist_from_ray_orig: (ray.origin - hit_point).length(),
            };
            if hit_point.z < 0.0 {
                //println!("Encountered hit at {:?}", hit_point);
                //assert!(false);
            }
            return Some(hit_info);
        }
    }
}
