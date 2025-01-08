extern crate ord_subset;
use crate::vec3::Vec3;
use crate::vec3_avx::{avx_cross_product, avx_dot_product};
use crate::vec3_sse::{sse_cross_product, sse_dot_product};
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
            corners,
            normal: get_triangle_normal(&corners),
            material,
            edges: [corners[1] - corners[0], corners[2] - corners[0]],
        }
    }
}

pub fn get_triangle_normal(corners: &[Vec3; 3]) -> Vec3 {
    let edge1 = corners[1] - corners[0];
    let edge2 = corners[2] - corners[0];
    edge1.cross_product(&edge2).normalize()
}

pub fn triangle_soa_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>; 3]; 3],
    edges: &[[Vec<f32>; 3]; 2],
    is_padding_triangle: &[bool],
    min_dist: f32,
    max_dist: f32,
) -> (Option<f32>, Option<usize>) {
    let eps = min_dist;

    let mut min_idx = 0;
    let mut min_param = 1000000.0;

    for (i, is_pad) in is_padding_triangle
        .iter()
        .enumerate()
        .take(vertices[0][0].len())
    {
        let vertex_a = Vec3::new(vertices[0][0][i], vertices[0][1][i], vertices[0][2][i]);

        let edge_a = Vec3::new(edges[0][0][i], edges[0][1][i], edges[0][1][i]);
        let edge_b = Vec3::new(edges[1][0][i], edges[1][1][i], edges[1][1][i]);

        let h = ray.direction.cross_product(&edge_b);
        let a = edge_a.dot(&h);

        if -eps < a && a < eps {
            continue;
        }
        let f = 1.0 / a;
        let s = ray.origin - vertex_a;
        let u = f * s.dot(&h);
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = s.cross_product(&edge_a);
        let v = f * ray.direction.dot(&q);
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * edge_b.dot(&q);
        if t > eps && t < min_param && !is_pad {
            // ray intersection
            min_param = t;
            min_idx = i;
        }
    }

    if min_param > eps && min_param < max_dist {
        (Some(min_param), Some(min_idx))
    } else {
        (None, None)
    }
}

pub fn basic_triangle_intersect_w_ray(
    ray: &Ray,
    vertices: &[Vec3; 3],
    edges: &[Vec3; 2],
    min_dist: f32,
    max_dist: f32,
) -> Option<f32> {
    let eps = min_dist;
    let h = ray.direction.cross_product(&edges[1]);
    let a = edges[0].dot(&h);
    if -eps < a && a < eps {
        return None;
    }
    let f = 1.0 / a;
    let s = ray.origin - vertices[0];
    let u = f * s.dot(&h);
    if !(0.0..=1.0).contains(&u) {
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
    None
}

/// # Safety
/// requires avx
pub unsafe fn triangle_soa_avx_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>; 3]; 3],
    edges: &[[Vec<f32>; 3]; 2],
    is_padding_triangle: &[bool],
    min_dist: f32,
    _max_dist: f32,
) -> (Option<f32>, Option<usize>) {
    let mut ray_params: Vec<f32> = Vec::with_capacity(vertices[0][0].len());
    let eps_f32 = min_dist;

    let eps = _mm256_set1_ps(eps_f32);
    let eps_frac = _mm256_set1_ps(1.0 / eps_f32);
    let neg_eps = _mm256_set1_ps(-eps_f32);
    let zero = _mm256_set1_ps(0.0);
    let one = _mm256_set1_ps(1.0);

    // load ray origin
    let ro_x = _mm256_set1_ps(ray.origin.x);
    let ro_y = _mm256_set1_ps(ray.origin.y);
    let ro_z = _mm256_set1_ps(ray.origin.z);

    // load ray direction
    let rd_x = _mm256_set1_ps(ray.direction.x);
    let rd_y = _mm256_set1_ps(ray.direction.y);
    let rd_z = _mm256_set1_ps(ray.direction.z);

    let chunk_size = 8;

    for (
        (((((((vert_ax, vert_ay), vert_az), edge_ax), edge_ay), edge_az), edge_bx), edge_by),
        edge_bz,
    ) in vertices[0][0]
        .chunks_exact(chunk_size)
        .zip(vertices[0][1].chunks(chunk_size))
        .zip(vertices[0][2].chunks(chunk_size))
        .zip(edges[0][0].chunks(chunk_size))
        .zip(edges[0][1].chunks(chunk_size))
        .zip(edges[0][2].chunks(chunk_size))
        .zip(edges[1][0].chunks(chunk_size))
        .zip(edges[1][1].chunks(chunk_size))
        .zip(edges[1][2].chunks(chunk_size))
    {
        let vert_ax = _mm256_loadu_ps(vert_ax.as_ptr());
        let vert_ay = _mm256_loadu_ps(vert_ay.as_ptr());
        let vert_az = _mm256_loadu_ps(vert_az.as_ptr());

        let edge_ax = _mm256_loadu_ps(edge_ax.as_ptr());
        let edge_ay = _mm256_loadu_ps(edge_ay.as_ptr());
        let edge_az = _mm256_loadu_ps(edge_az.as_ptr());

        let edge_bx = _mm256_loadu_ps(edge_bx.as_ptr());
        let edge_by = _mm256_loadu_ps(edge_by.as_ptr());
        let edge_bz = _mm256_loadu_ps(edge_bz.as_ptr());

        // let h = ray.direction.cross_product(&edges[1]);
        let (h_x, h_y, h_z) = avx_cross_product(rd_x, rd_y, rd_z, edge_bx, edge_by, edge_bz);

        // let a = edges[0].dot(&h);
        let a_sum = avx_dot_product(edge_ax, edge_ay, edge_az, h_x, h_y, h_z);

        // condition 1: -eps < a
        // &&
        // condition 2:  a < eps
        let c1_part1 = _mm256_cmp_ps(neg_eps, a_sum, _CMP_LT_OQ);
        let c1_part2 = _mm256_cmp_ps(a_sum, eps, _CMP_LT_OQ);
        let c1 = _mm256_and_ps(c1_part1, c1_part2);

        // f = 1.0/a
        let f = _mm256_div_ps(one, a_sum);

        // s = ray_origin - vertex[0]
        let s_x = _mm256_sub_ps(ro_x, vert_ax);
        let s_y = _mm256_sub_ps(ro_y, vert_ay);
        let s_z = _mm256_sub_ps(ro_z, vert_az);

        // let u = f * s.dot(&h);
        let u = _mm256_mul_ps(f, avx_dot_product(s_x, s_y, s_z, h_x, h_y, h_z));

        // condition 1: u < 0.0
        // ||
        // condition 2:  u > 1.0
        let c2_part1 = _mm256_cmp_ps(u, zero, _CMP_LT_OQ);
        let c2_part2 = _mm256_cmp_ps(u, one, _CMP_GT_OQ);
        let c2 = _mm256_or_ps(c2_part1, c2_part2);

        // let q = s.cross_product(&edges[0]);
        let (q_x, q_y, q_z) = avx_cross_product(s_x, s_y, s_z, edge_ax, edge_ay, edge_az);

        // let v = f * ray.direction.dot(&q);
        let v = _mm256_mul_ps(f, avx_dot_product(rd_x, rd_y, rd_z, q_x, q_y, q_z));

        // condition 1: v < 0.0
        // ||
        // condition 2:  u + v > 1.0
        let c3_part1 = _mm256_cmp_ps(v, zero, _CMP_LT_OQ);
        let c3_part2 = _mm256_cmp_ps(_mm256_add_ps(u, v), one, _CMP_GT_OQ);
        let c3 = _mm256_or_ps(c3_part1, c3_part2);

        // let t = f * edges[1].dot(&q);
        let t = _mm256_mul_ps(f, avx_dot_product(edge_bx, edge_by, edge_bz, q_x, q_y, q_z));

        // condition1: t > eps
        // &&
        // condition2: t < 1/eps
        let c4_part1 = _mm256_cmp_ps(t, eps, _CMP_GT_OQ);
        let c4_part2 = _mm256_cmp_ps(t, eps_frac, _CMP_LT_OQ);
        let c4 = _mm256_and_ps(c4_part1, c4_part2);

        let c23 = _mm256_or_ps(c2, c3);

        let c_res123 = _mm256_or_ps(c1, c23);

        let has_intersect = _mm256_andnot_ps(c_res123, c4);
        //println!("{:?}",has_intersect);

        let minus_a_lot = _mm256_set1_ps(-1000.0);

        let res = _mm256_or_ps(
            _mm256_and_ps(has_intersect, t),
            _mm256_andnot_ps(has_intersect, minus_a_lot),
        );

        let t_unpacked: [f32; 8] = mem::transmute(res);
        ray_params.extend_from_slice(&t_unpacked);
    }

    find_smallest_element_bigger_than_eps(&ray_params, is_padding_triangle, eps_f32)
}

/// # Safety
/// requires sse
pub unsafe fn triangle_soa_sse_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>; 3]; 3],
    edges: &[[Vec<f32>; 3]; 2],
    is_padding_triangle: &[bool],
    min_dist: f32,
    _max_dist: f32,
) -> (Option<f32>, Option<usize>) {
    let mut ray_params: Vec<f32> = Vec::with_capacity(vertices[0][0].len());
    let eps_f32 = min_dist;
    let eps = _mm_set1_ps(eps_f32);
    let eps_frac = _mm_set1_ps(1.0 / eps_f32);
    let neg_eps = _mm_set1_ps(-eps_f32);
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

        let t_unpacked: [f32; 4] = mem::transmute(res);
        ray_params.extend_from_slice(&t_unpacked);
    }

    find_smallest_element_bigger_than_eps(&ray_params, is_padding_triangle, eps_f32)
}

pub fn find_smallest_element_bigger_than_eps(
    ray_params: &[f32],
    is_padding_triangle: &[bool],
    eps: f32,
) -> (Option<f32>, Option<usize>) {
    let mut min_idx = 0;
    let mut min_param = 1000000.0;
    for (idx, ray_param) in ray_params.iter().enumerate() {
        if *ray_param > eps && *ray_param < min_param && !is_padding_triangle[idx] {
            min_param = *ray_param;
            min_idx = idx;
        }
    }
    if min_param > eps && min_param < 100000.0 {
        (Some(min_param), Some(min_idx))
    } else {
        (None, None)
    }
}

impl Intersectable for BasicTriangle {
    /// see https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    fn intersect_with_ray(
        &self,
        ray: &Ray,
        min_dist: f32,
        max_dist: f32,
    ) -> Option<HitInformation> {
        let ray_param_op =
            basic_triangle_intersect_w_ray(ray, &self.corners, &self.edges, min_dist, max_dist);

        match ray_param_op {
            Some(t) => {
                let hit_point = ray.point_at(t);
                let dist_from_ray_orig = (ray.origin - hit_point).length();
                if dist_from_ray_orig < min_dist || dist_from_ray_orig > max_dist {
                    None
                } else {
                    Some(HitInformation {
                        hit_point,
                        hit_normal: self.normal,
                        hit_material: &*self.material,
                        dist_from_ray_orig,
                    })
                }
            }
            None => None,
        }
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
