use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};
use std::arch::x86_64::*;

pub struct BasicTriangle {
    ///
    /// Convention: counter clockwise!
    ///
    pub corners: [Vec3; 3],
    pub normal: Vec3,
    pub edges: [Vec3; 2],
    pub material: Box<dyn RayScattering + Sync>,
}

impl BasicTriangle {
    pub fn new(corners: [Vec3; 3], material: Box<dyn RayScattering + Sync>) -> BasicTriangle {
        BasicTriangle {
            corners: corners,
            normal: get_triangle_normal(&corners),
            material: material,
            edges: [corners[1] - corners[0], corners[2] - corners[0]],
        }
    }
}

pub fn get_triangle_normal(corners: &[Vec3; 3]) -> Vec3 {
    let edge1 = corners[1] - corners[0];
    let edge2 = corners[2] - corners[0];
    let normal = edge1.cross_product(&edge2).normalize();
    return normal;
}

pub fn triangle_soa_intersect_with_ray(
    ray: &Ray,
    vertices: &[Vec3; 3],
    edges: &[Vec3; 2],
    min_dist: f32,
    max_dist: f32,
) -> Option<f32> {
    let eps = 0.0000001;
    let h = ray.direction.cross_product(&edges[1]);
    let a = edges[0].dot(&h);
    if -eps < a && a < eps {
        return None;
    }
    let f = 1.0 / a;
    let s = ray.origin - vertices[0];
    let u = f * s.dot(&h);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = s.cross_product(&edges[0]);
    let v = f * ray.direction.dot(&q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = f * edges[1].dot(&q);
    if t > eps
    // ray intersection
    {
        let hit_point = ray.point_at(t);
        let dist_from_ray_orig = (ray.origin - hit_point).length();
        if dist_from_ray_orig < min_dist || dist_from_ray_orig > max_dist {
            return None;
        } else {
            return Some(t);
        }
    }

    return None;
}

macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

pub unsafe fn sse_cross_product(a: __m128, b: __m128) -> __m128 {
    return _mm_sub_ps(
        _mm_mul_ps(
            _mm_shuffle_ps(a, a, _MM_SHUFFLE!(3, 0, 2, 1)),
            _mm_shuffle_ps(b, b, _MM_SHUFFLE!(3, 1, 0, 2)),
        ),
        _mm_mul_ps(
            _mm_shuffle_ps(a, a, _MM_SHUFFLE!(3, 1, 0, 2)),
            _mm_shuffle_ps(b, b, _MM_SHUFFLE!(3, 0, 2, 1)),
        ),
    );
}
pub unsafe fn triangle_soa_sse_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>;3];3],
    edges: &[[Vec<f32>;3];2],
    min_dist: f32,
    max_dist: f32,
) -> (Option<f32>, Option<u32>) {
    let eps_sse = _mm_set1_ps(0.0001);
    let min_dist_sse = _mm_set1_ps(min_dist);
    let max_dist_sse = _mm_set1_ps(max_dist);

    // load ray origin
    let ro_x = _mm_set_ps1(ray.origin.x);
    let ro_y = _mm_set_ps1(ray.origin.y);
    let ro_z = _mm_set_ps1(ray.origin.z);

    // load ray direction
    let rd_x = _mm_set_ps1(ray.direction.x);
    let rd_y = _mm_set_ps1(ray.direction.y);
    let rd_z = _mm_set_ps1(ray.direction.z);

    for ((((verts_a, verts_b), verts_c), egdes_a), edges_b) in vertices[0]
        .chunks(4)
        .zip(vertices[1].chunks(4))
        .zip(vertices[2].chunks(4))
        .zip(edges[0].chunks(4))
        .zip(edges[1].chunks(4))
    {
    
    // let h = ray.direction.cross_product(&edges[1]);
    //
    }
    return (None, None);
}

impl Intersectable for BasicTriangle {
    /// see https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    fn intersect_with_ray<'a>(
        &'a self,
        ray: &Ray,
        min_dist: f32,
        max_dist: f32,
    ) -> Option<HitInformation> {
        let ray_param_op =
            triangle_soa_intersect_with_ray(&ray, &self.corners, &self.edges, min_dist, max_dist);

        if ray_param_op.is_some()
        // ray intersection
        {
            let t = ray_param_op.unwrap();
            let hit_point = ray.point_at(t);
            let dist_from_ray_orig = (ray.origin - hit_point).length();
            if dist_from_ray_orig < min_dist || dist_from_ray_orig > max_dist {
                return None;
            } else {
                return Some(HitInformation {
                    hit_point: hit_point,
                    hit_normal: self.normal,
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
        let test_tri = Box::new(BasicTriangle::new(
            [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
            ],
            Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        ));

        assert_eq!(test_tri.normal, Vec3::new(0.0, 0.0, 1.0));

        let test_tri = Box::new(BasicTriangle::new(
            [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        ));

        assert_eq!(test_tri.normal, Vec3::new(-1.0, -1.0, 0.0).normalize());
    }
}
