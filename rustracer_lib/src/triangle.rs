use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};

pub struct BasicTriangle {
    ///
    /// Convention: counter clockwise!
    ///
    pub corners: [Vec3; 3],
    pub material: Box<dyn RayScattering + Sync>,
}

impl BasicTriangle {
    pub fn get_normal(&self) -> Vec3 {
        let edge1 = self.corners[1] - self.corners[0];
        let edge2 = self.corners[2] - self.corners[0];
        let normal = edge1.cross_product(&edge2).normalize();
        return normal;
    }
}

impl Intersectable for BasicTriangle {
    fn intersect_with_ray<'a>(
        &'a self,
        ray: &Ray,
        min_dist: f64,
        max_dist: f64,
    ) -> Option<HitInformation> {
        let eps = 0.0000001;
        let edge1 = self.corners[1] - self.corners[0];
        let edge2 = self.corners[2] - self.corners[0];
        let h = ray.direction.cross_product(&edge2);
        let a = edge1.dot(&h);
        if -eps < a && a < eps {
            return None;
        }
        let f = 1.0 / a;
        let s = ray.origin - self.corners[0];
        let u = f * s.dot(&h);
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = s.cross_product(&edge1);
        let v = f * ray.direction.dot(&q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * edge2.dot(&q);
        if t > eps
        // ray intersection
        {
            let hit_point = ray.point_at(t);
            let dist_from_ray_orig = (ray.origin - hit_point).length();
            if dist_from_ray_orig < min_dist || dist_from_ray_orig > max_dist {
                return None;
            } else {
                return Some(HitInformation {
                    hit_point: hit_point,
                    hit_normal: self.get_normal(),
                    hit_material: &*self.material,
                    dist_from_ray_orig: dist_from_ray_orig,
                });
            }
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::{BasicTriangle, Vec3};
    // dont need Material here, use Option?
    use crate::lambertian::Lambertian;
    #[test]
    fn test_triangle_normal() {
        let test_tri = Box::new(BasicTriangle {
            corners: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
            ],

            material: Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        });

        let normal = test_tri.get_normal();

        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));

        let test_tri = Box::new(BasicTriangle {
            corners: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            material: Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        });

        let normal = test_tri.get_normal();

        assert_eq!(normal, Vec3::new(-1.0, -1.0, 0.0).normalize());
    }
}
