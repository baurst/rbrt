use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

pub struct Triangle {
    pub corner_a: Vec3,
    pub corner_b: Vec3,
    pub corner_c: Vec3,
    pub material: Box<dyn RayScattering + Sync>,
}

impl Triangle {
    pub fn intersect_with_ray<'a>(&'a self, ray: &Ray) -> Option<HitInformation> {
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
        /*
        const float EPSILON = 0.0000001;
            Vector3D vertex0 = inTriangle->vertex0;
            Vector3D vertex1 = inTriangle->vertex1;
            Vector3D vertex2 = inTriangle->vertex2;
            Vector3D edge1, edge2, h, s, q;
            float a,f,u,v;
            edge1 = vertex1 - vertex0;
            edge2 = vertex2 - vertex0;
            h = rayVector.crossProduct(edge2);
            a = edge1.dotProduct(h);
            if (a > -EPSILON && a < EPSILON)
                return false;    // This ray is parallel to this triangle.
            f = 1.0/a;
            s = rayOrigin - vertex0;
            u = f * s.dotProduct(h);
            if (u < 0.0 || u > 1.0)
                return false;
            q = s.crossProduct(edge1);
            v = f * rayVector.dotProduct(q);
            if (v < 0.0 || u + v > 1.0)
                return false;
            // At this stage we can compute t to find out where the intersection point is on the line.
            float t = f * edge2.dotProduct(q);
            if (t > EPSILON) // ray intersection
            {
                outIntersectionPoint = rayOrigin + rayVector * t;
                return true;
            }
            else // This means that there is a line intersection but not a ray intersection.
                return false;
                */
    }

    pub fn get_normal(&self) -> Vec3 {
        let edge1 = self.corner_b - self.corner_a;
        let edge2 = self.corner_c - self.corner_a;
        let normal = edge1.cross_product(&edge2).normalize();
        return normal;
    }
    /*
    pub fn new(corner_a: Vec3, corner_b: Vec3, corner_c: Vec3, material: RayScattering)-> Triangle{
        return Triangle{
            corner_a: corner_a, corner_b: corner_b, corner_c: corner_c, material: Box::new(material),
        }
    }
    */
}
