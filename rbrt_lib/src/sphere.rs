use std::cmp::Ordering;

use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Box<dyn RayScattering + Sync>,
}

impl Intersectable for Sphere {
    ///
    /// Compute intersection of ray and sphere
    /// ray: r(t) = o + td
    /// sphere: (p-c)*(p-c) = r^2
    /// insert ray for p into sphere equation, then solve quadratic equation for t
    /// (o+td-c)(o+td-c)=r^2
    /// t1/2 = (-B +- sqrt(B^2 - 4AC))/(2A)
    fn intersect_with_ray(
        &self,
        ray: &Ray,
        min_dist: f32,
        max_dist: f32,
    ) -> Option<HitInformation> {
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
            None
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
            let dist_from_ray_orig = (ray.origin - hit_point).length();

            if dist_from_ray_orig < min_dist || dist_from_ray_orig > max_dist {
                None
            } else {
                let hit_normal = hit_point - self.center;
                let hit_info = HitInformation {
                    hit_normal,
                    hit_point,
                    hit_material: &*self.material,
                    dist_from_ray_orig,
                };
                Some(hit_info)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Ray, Sphere, Vec3};
    use crate::metal::Metal;
    use crate::Intersectable;

    #[test]
    fn test_sphere_intersection() {
        // Camera looks onto sphere from front
        let test_sphere = Sphere {
            center: Vec3::new(0.0, 0.0, -10.0),
            radius: 1.0,
            material: Box::new(Metal {
                albedo: Vec3::new(0.8, 0.8, 0.8),
                roughness: 0.005,
            }),
        };
        let test_ray = Ray {
            origin: Vec3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
        };

        let hit_info = test_sphere.intersect_with_ray(&test_ray, 0.001, 1000.0);

        assert!(hit_info.is_some());

        let hit_info = hit_info.unwrap();
        assert_eq!(hit_info.hit_point, Vec3::new(0.0, 0.0, -9.0));
        assert_eq!(hit_info.hit_normal, Vec3::new(0.0, 0.0, 1.0));

        // ray hits sphere from behind
        let test_ray = Ray {
            origin: Vec3::new(0.0, 0.0, -15.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
        };

        let hit_info = test_sphere.intersect_with_ray(&test_ray, 0.001, 1000.0);

        assert!(hit_info.is_some());

        let hit_info = hit_info.unwrap();
        assert_eq!(hit_info.hit_point, Vec3::new(0.0, 0.0, -11.0));
        assert_eq!(hit_info.hit_normal, Vec3::new(0.0, 0.0, -1.0));
    }
}
