extern crate ord_subset;
use crate::vec3::Vec3;
use crate::{HitInformation, Intersectable, Ray, RayScattering};
use std::arch::x86_64::*;
use std::mem;

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

pub unsafe fn sse_dot_product(
    a_x: __m128,
    a_y: __m128,
    a_z: __m128,
    b_x: __m128,
    b_y: __m128,
    b_z: __m128,
) -> __m128 {
    let a_x = _mm_mul_ps(a_x, b_x);
    let a_y = _mm_mul_ps(a_y, b_y);
    let a_z = _mm_mul_ps(a_z, b_z);
    let a_sum = _mm_add_ps(_mm_add_ps(a_x, a_y), a_z);
    return a_sum;
}

pub unsafe fn sse_cross_product(
    a_x: __m128,
    a_y: __m128,
    a_z: __m128,
    b_x: __m128,
    b_y: __m128,
    b_z: __m128,
) -> (__m128, __m128, __m128) {
    let c_x = _mm_sub_ps(_mm_mul_ps(a_y, b_z), _mm_mul_ps(a_z, b_y));
    let c_y = _mm_sub_ps(_mm_mul_ps(a_z, b_x), _mm_mul_ps(a_x, b_z));
    let c_z = _mm_sub_ps(_mm_mul_ps(a_x, b_y), _mm_mul_ps(a_y, b_x));

    return (c_x, c_y, c_z);
}

pub unsafe fn triangle_soa_sse_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>; 3]; 3],
    edges: &[[Vec<f32>; 3]; 2],
    min_dist: f32,
    max_dist: f32,
) -> (Option<f32>, Option<usize>) {
    let mut ray_params: Vec<f32> = vec![];
    ray_params.reserve(vertices[0][0].len());
    let eps_f32 = 0.00001;
    let eps = _mm_set1_ps(eps_f32);
    let eps_frac = _mm_set1_ps(1.0 / eps_f32);
    let neg_eps = _mm_set1_ps(-eps_f32);
    let min_dist = _mm_set1_ps(min_dist);
    let max_dist = _mm_set1_ps(max_dist);
    let zero = _mm_set1_ps(0.0);
    let one = _mm_set1_ps(1.0);

    // load ray origin
    let ro_x = _mm_set_ps1(ray.origin.x);
    let ro_y = _mm_set_ps1(ray.origin.y);
    let ro_z = _mm_set_ps1(ray.origin.z);

    // load ray direction
    let rd_x = _mm_set_ps1(ray.direction.x);
    let rd_y = _mm_set_ps1(ray.direction.y);
    let rd_z = _mm_set_ps1(ray.direction.z);

    for (
        (((((((vert_ax, vert_ay), vert_az), edge_ax), edge_ay), edge_az), edge_bx), edge_by),
        edge_bz,
    ) in vertices[0][0]
        .chunks_exact(4)
        .zip(vertices[0][1].chunks(4))
        .zip(vertices[0][2].chunks(4))
        .zip(edges[0][0].chunks(4))
        .zip(edges[0][1].chunks(4))
        .zip(edges[0][2].chunks(4))
        .zip(edges[1][0].chunks(4))
        .zip(edges[1][1].chunks(4))
        .zip(edges[1][2].chunks(4))
    {
        let vert_ax = _mm_loadu_ps(vert_ax.as_ptr());
        let vert_ay = _mm_loadu_ps(vert_ay.as_ptr());
        let vert_az = _mm_loadu_ps(vert_az.as_ptr());

        let edge_ax = _mm_loadu_ps(edge_ax.as_ptr());
        let edge_ay = _mm_loadu_ps(edge_ay.as_ptr());
        let edge_az = _mm_loadu_ps(edge_az.as_ptr());

        let edge_bx = _mm_loadu_ps(edge_bx.as_ptr());
        let edge_by = _mm_loadu_ps(edge_by.as_ptr());
        let edge_bz = _mm_loadu_ps(edge_bz.as_ptr());

        // let h = ray.direction.cross_product(&edges[1]);
        let (h_x, h_y, h_z) = sse_cross_product(rd_x, rd_y, rd_z, edge_bx, edge_by, edge_bz);

        // let a = edges[0].dot(&h);
        let a_sum = sse_dot_product(edge_ax, edge_ay, edge_az, h_x, h_y, h_z);

        // condition 1: -eps < a
        // &&
        // condition 2:  a < eps
        let c1_part1 = _mm_cmplt_ps(neg_eps, a_sum);
        let c1_part2 = _mm_cmplt_ps(a_sum, eps);
        let c1 = _mm_and_ps(c1_part1, c1_part2);

        // f = 1.0/a
        let f = _mm_div_ps(one, a_sum);

        // s = ray_origin - vertex[0]
        let s_x = _mm_sub_ps(ro_x, vert_ax);
        let s_y = _mm_sub_ps(ro_y, vert_ay);
        let s_z = _mm_sub_ps(ro_z, vert_az);

        // let u = f * s.dot(&h);
        let u = _mm_mul_ps(f, sse_dot_product(s_x, s_y, s_z, h_x, h_y, h_z));

        // condition 1: u < 0.0
        // ||
        // condition 2:  u > 1.0
        let c2_part1 = _mm_cmplt_ps(u, zero);
        let c2_part2 = _mm_cmpgt_ps(u, one);
        let c2 = _mm_or_ps(c2_part1, c2_part2);

        // let q = s.cross_product(&edges[0]);
        let (q_x, q_y, q_z) = sse_cross_product(s_x, s_y, s_z, edge_ax, edge_ay, edge_az);

        // let v = f * ray.direction.dot(&q);
        let v = _mm_mul_ps(f, sse_dot_product(rd_x, rd_y, rd_z, q_x, q_y, q_z));

        // condition 1: v < 0.0
        // ||
        // condition 2:  u + v > 1.0
        let c3_part1 = _mm_cmplt_ps(v, zero);
        let c3_part2 = _mm_cmpgt_ps(_mm_add_ps(u, v), one);
        let c3 = _mm_or_ps(c3_part1, c3_part2);

        // let t = f * edges[1].dot(&q);
        let t = _mm_mul_ps(f, sse_dot_product(edge_bx, edge_by, edge_bz, q_x, q_y, q_z));

        // condition1: t > eps
        // &&
        // condition2: t < 1/eps
        let c4_part1 = _mm_cmpgt_ps(t, eps);
        let c4_part2 = _mm_cmplt_ps(t, eps_frac);
        let c4 = _mm_and_ps(c4_part1, c4_part2);

        let c23 = _mm_or_ps(c2, c3);

        let c_res123 = _mm_or_ps(c1, c23);

        let has_intersect = _mm_andnot_ps(c_res123, c4);
        //println!("{:?}",has_intersect);

        let minus_a_lot = _mm_set1_ps(-1000.0);

        let res = _mm_or_ps(
            _mm_and_ps(has_intersect, t),
            _mm_andnot_ps(has_intersect, minus_a_lot),
        );
        let term_unpacked: (f32, f32, f32, f32) = mem::transmute(res);

        ray_params.push(term_unpacked.0);
        ray_params.push(term_unpacked.1);
        ray_params.push(term_unpacked.2);
        ray_params.push(term_unpacked.3);
    }
    let mut num_intersections = 0;
    let mut min_idx = 0;
    let mut min_param = 1000000.0;
    // look for the minimum param that is larger than 0.0
    for (idx, ray_param) in ray_params.iter().enumerate() {
        if *ray_param > eps_f32 && *ray_param < min_param {
            min_param = *ray_param;
            min_idx = idx;
            num_intersections += 1;
        }
    }
    // println!("Found {} intersections!", num_intersections);
    if min_param > eps_f32 && min_param < 100000.0 {
        return (Some(min_param), Some(min_idx));
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
    use crate::triangle::{sse_cross_product, sse_dot_product};
    use std::arch::x86_64::*;
    use std::mem;

    unsafe fn assert_m128_equal(a: __m128, b: __m128) {
        let a: (f32, f32, f32, f32) = mem::transmute(a);
        let b: (f32, f32, f32, f32) = mem::transmute(b);
        assert_eq!(a.0, b.0);
        assert_eq!(a.1, b.1);
        assert_eq!(a.2, b.2);
        assert_eq!(a.3, b.3);
    }

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

    #[test]
    fn test_sse_cross_product() {
        unsafe {
            let a_x = _mm_loadu_ps(vec![1.0, 0.0, 3.0, 2.0].as_ptr());
            let a_y = _mm_loadu_ps(vec![0.0, 1.0, 4.0, 6.0].as_ptr());
            let a_z = _mm_loadu_ps(vec![0.0, 0.0, 4.0, 3.0].as_ptr());

            let b_x = _mm_loadu_ps(vec![0.0, 0.0, 1.0, 2.0].as_ptr());
            let b_y = _mm_loadu_ps(vec![1.0, 0.0, -2.0, 1.0].as_ptr());
            let b_z = _mm_loadu_ps(vec![0.0, 1.0, 3.0, -2.0].as_ptr());

            let c_x_exp = _mm_loadu_ps(vec![0.0, 1.0, 20.0, -15.0].as_ptr());
            let c_y_exp = _mm_loadu_ps(vec![0.0, 0.0, -5.0, 10.0].as_ptr());
            let c_z_exp = _mm_loadu_ps(vec![1.0, 0.0, -10.0, -10.0].as_ptr());

            let (c_x, c_y, c_z) = sse_cross_product(a_x, a_y, a_z, b_x, b_y, b_z);

            assert_m128_equal(c_x, c_x_exp);
            assert_m128_equal(c_y, c_y_exp);
            assert_m128_equal(c_z, c_z_exp);
        }
    }

    #[test]
    fn test_trivial_sse_cross_product() {
        unsafe {
            let a_x = _mm_set1_ps(0.0);
            let a_y = _mm_set1_ps(0.0);
            let a_z = _mm_set1_ps(1.0);

            let b_x = _mm_set1_ps(1.0);
            let b_y = _mm_set1_ps(0.0);
            let b_z = _mm_set1_ps(0.0);

            let c_x_exp = _mm_set1_ps(0.0);
            let c_y_exp = _mm_set1_ps(1.0);
            let c_z_exp = _mm_set1_ps(0.0);

            let (c_x, c_y, c_z) = sse_cross_product(a_x, a_y, a_z, b_x, b_y, b_z);

            assert_m128_equal(c_x, c_x_exp);
            assert_m128_equal(c_y, c_y_exp);
            assert_m128_equal(c_z, c_z_exp);
        }
    }

    #[test]
    fn test_sse_dot_product() {
        unsafe {
            let a_x = _mm_loadu_ps(vec![1.0, 0.0, 3.0, 2.0].as_ptr());
            let a_y = _mm_loadu_ps(vec![0.0, 1.0, 4.0, 6.0].as_ptr());
            let a_z = _mm_loadu_ps(vec![0.0, 0.0, 4.0, 3.0].as_ptr());

            let b_x = _mm_loadu_ps(vec![0.0, 0.0, 1.0, 2.0].as_ptr());
            let b_y = _mm_loadu_ps(vec![1.0, 0.0, -2.0, 1.0].as_ptr());
            let b_z = _mm_loadu_ps(vec![0.0, 1.0, 3.0, -2.0].as_ptr());

            let s_exp = _mm_loadu_ps(vec![0.0, 0.0, 7.0, 4.0].as_ptr());

            let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

            assert_m128_equal(s, s_exp);
        }
    }

    #[test]
    fn test_trivial_sse_dot_product() {
        unsafe {
            let a_x = _mm_set1_ps(0.0);
            let a_y = _mm_set1_ps(0.0);
            let a_z = _mm_set1_ps(1.0);

            let b_x = _mm_set1_ps(1.0);
            let b_y = _mm_set1_ps(0.0);
            let b_z = _mm_set1_ps(0.0);

            let s_exp = _mm_set1_ps(0.0);

            let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

            assert_m128_equal(s, s_exp);

            let a_x = _mm_set1_ps(1.0);
            let a_y = _mm_set1_ps(1.0);
            let a_z = _mm_set1_ps(1.0);

            let b_x = _mm_set1_ps(1.0);
            let b_y = _mm_set1_ps(1.0);
            let b_z = _mm_set1_ps(1.0);

            let s_exp = _mm_set1_ps(3.0);

            let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

            assert_m128_equal(s, s_exp);
        }
    }
}
