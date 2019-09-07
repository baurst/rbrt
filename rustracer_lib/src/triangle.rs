use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};

pub struct Triangle {
    ///
    /// Convention: counter clockwise!
    ///
    pub corner_a: Vec3,
    pub corner_b: Vec3,
    pub corner_c: Vec3,
    pub material: Box<dyn RayScattering + Sync>,
}

impl Triangle {
    pub fn get_normal(&self) -> Vec3 {
        let edge1 = self.corner_b - self.corner_a;
        let edge2 = self.corner_c - self.corner_a;
        let normal = edge1.cross_product(&edge2).normalize();
        return normal;
    }
}

impl Intersectable for Triangle {
    fn intersect_with_ray<'a>(&'a self, ray: &Ray) -> Option<HitInformation> {
        let eps = 0.0000001;
        let edge1 = self.corner_b - self.corner_a;
        let edge2 = self.corner_c - self.corner_a;
        let h = ray.direction.cross_product(&edge2);
        let a = edge1.dot(&h);
        if -eps < a && a < eps {
            return None;
        }
        let f = 1.0 / a;
        let s = ray.origin - self.corner_a;
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
            return Some(HitInformation {
                hit_point: ray.point_at(t),
                hit_normal: self.get_normal(),
                hit_material: &*self.material,
                dist_from_ray_orig: t,
            });
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::{Triangle, Vec3};
    // dont need Material here, use Option?
    use crate::lambertian::Lambertian;
    #[test]
    fn test_triangle_normal() {
        let test_tri = Box::new(Triangle {
            corner_a: Vec3::new(1.0, 0.0, 0.0),
            corner_b: Vec3::new(1.0, 1.0, 0.0),
            corner_c: Vec3::new(0.0, 0.0, 0.0),
            material: Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        });

        let normal = test_tri.get_normal();

        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));

        let test_tri = Box::new(Triangle {
            corner_a: Vec3::new(1.0, 0.0, 0.0),
            corner_b: Vec3::new(1.0, 0.0, 1.0),
            corner_c: Vec3::new(0.0, 1.0, 0.0),
            material: Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        });

        let normal = test_tri.get_normal();

        assert_eq!(normal, Vec3::new(-1.0, -1.0, 0.0).normalize());
    }
}