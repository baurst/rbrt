extern crate ord_subset;
use ord_subset::OrdSubsetIterExt;
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

macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

pub unsafe fn sse_dot_product(a_x: __m128, a_y: __m128, a_z: __m128,
                        b_x: __m128, b_y: __m128, b_z: __m128) -> __m128 {
    let a_x = _mm_mul_ps(a_x,b_x);
    let a_y = _mm_mul_ps(a_y,b_y);
    let a_z = _mm_mul_ps(a_z,b_z);
    let a_sum = _mm_add_ps(_mm_add_ps(a_x,a_y),a_z);
    return a_sum;
}

pub unsafe fn sse_cross_product(a_x: __m128, a_y: __m128, a_z: __m128,
                        b_x: __m128, b_y: __m128, b_z: __m128) -> (__m128, __m128, __m128) {
    let c_x = _mm_sub_ps(_mm_mul_ps(a_y,b_z),_mm_mul_ps(a_z, b_y));
    let c_y = _mm_sub_ps(_mm_mul_ps(a_z,b_x),_mm_mul_ps(a_x, b_z));
    let c_z = _mm_sub_ps(_mm_mul_ps(a_x,b_y),_mm_mul_ps(a_y, b_x));

    return (c_x, c_y, c_z);
}

pub unsafe fn triangle_soa_sse_intersect_with_ray(
    ray: &Ray,
    vertices: &[[Vec<f32>;3];3],
    edges: &[[Vec<f32>;3];2],
    min_dist: f32,
    max_dist: f32,
) -> (Option<f32>, Option<usize>) {
    let mut ray_params : Vec<f32> = vec![];
    ray_params.reserve(vertices[0][0].len()); 

    let eps = _mm_set1_ps(0.0001);
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

    for ((((((((((((((vert_ax, vert_ay), vert_az), vert_bx), vert_by), vert_bz), vert_cx), vert_cy),vert_cz), edge_ax), edge_ay), edge_az), edge_bx), edge_by), edge_bz) in vertices[0][0]
        .chunks(4)
        .zip(vertices[0][1].chunks(4))
        .zip(vertices[0][2].chunks(4))
        .zip(vertices[1][0].chunks(4))
        .zip(vertices[1][1].chunks(4))
        .zip(vertices[1][2].chunks(4))
        .zip(vertices[2][0].chunks(4))
        .zip(vertices[2][1].chunks(4))
        .zip(vertices[2][2].chunks(4))
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

    let vert_bx = _mm_loadu_ps(vert_bx.as_ptr());
    let vert_by = _mm_loadu_ps(vert_by.as_ptr());
    let vert_bz = _mm_loadu_ps(vert_bz.as_ptr());

    let vert_cx = _mm_loadu_ps(vert_cx.as_ptr());
    let vert_cy = _mm_loadu_ps(vert_cy.as_ptr());
    let vert_cz = _mm_loadu_ps(vert_cz.as_ptr());

    let edge_ax = _mm_loadu_ps(edge_ax.as_ptr());
    let edge_ay = _mm_loadu_ps(edge_ay.as_ptr());
    let edge_az = _mm_loadu_ps(edge_az.as_ptr());

    let edge_bx = _mm_loadu_ps(edge_bx.as_ptr());
    let edge_by = _mm_loadu_ps(edge_by.as_ptr());
    let edge_bz = _mm_loadu_ps(edge_bz.as_ptr());

    // let h = ray.direction.cross_product(&edges[1]);
    let (h_x, h_y, h_z) = sse_cross_product(rd_x, rd_y, rd_z, edge_bx, edge_by, edge_bz);

    // dot product
    // let a = edges[0].dot(&h);
    let a_x = _mm_mul_ps(edge_ax,h_x);
    let a_y = _mm_mul_ps(edge_ay,h_y);
    let a_z = _mm_mul_ps(edge_az,h_z);
    let a_sum = _mm_add_ps(_mm_add_ps(a_x,a_y),a_z);

    // condition 1: -eps < a
    // condition 2:  a < eps
    let c1_part1 = _mm_cmplt_ps(_mm_xor_ps(eps, _mm_set1_ps(-0.0)), a_sum);
    let c1_part2 = _mm_cmplt_ps(a_sum, eps);
    let c1 =  _mm_and_ps(c1_part1, c1_part2);

    let f = _mm_div_ps(one, a_sum);
    let s_x = _mm_sub_ps(ro_x, vert_ax);
    let s_y = _mm_sub_ps(ro_y, vert_ay);
    let s_z = _mm_sub_ps(ro_z, vert_az);

    let u = _mm_mul_ps(f, sse_dot_product(s_x, s_y, s_z, h_x, h_y, h_z));

    // condition 1: u < 0.0
    // condition 2:  u > 1.0
    let c2_part1 = _mm_cmplt_ps(u, zero);
    let c2_part2 = _mm_cmplt_ps(u, one);
    let c2 = _mm_or_ps(c2_part1, c2_part2);

    // let q = s.cross_product(&edges[0]);
    let (q_x, q_y, q_z) = sse_cross_product(s_x, s_y, s_z, edge_ax, edge_ay, edge_az);
    let v = _mm_mul_ps(f, sse_dot_product(rd_x, rd_y, rd_z, q_x, q_y, q_z));
    
    // condition 1: v < 0.0
    // condition 2:  u + v > 1.0
    let c3_part1 = _mm_cmplt_ps(v, zero);
    let c3_part2 = _mm_cmpgt_ps(_mm_add_ps(u,v), one);
    let c3 = _mm_or_ps(c3_part1, c3_part2);

    // let t = f * edges[1].dot(&q);
    let t = _mm_mul_ps(f, sse_dot_product(edge_bx, edge_by, edge_bz, q_x, q_y, q_z));

    // t > eps
    let c4 = _mm_cmpgt_ps(t, eps);

    let c_res123 = _mm_or_ps(c1, _mm_or_ps(c2, c3));
    
    let has_intersect = _mm_andnot_ps(c_res123, c4);

    let minus_a_lot = _mm_set1_ps(-1000.0);

    let res = _mm_or_ps(_mm_and_ps(has_intersect, t),
                           _mm_andnot_ps(has_intersect, minus_a_lot));
    let term_unpacked: (f32, f32, f32, f32) = mem::transmute(res);
    ray_params.push(term_unpacked.3);
    ray_params.push(term_unpacked.2);
    ray_params.push(term_unpacked.1);
    ray_params.push(term_unpacked.0);
    }
    
    //let max_param = *ray_params.iter().ord_subset_max_by_key(|n| n.abs()).unwrap();
    let max_param = *ray_params.iter().ord_subset_max().unwrap();
    
    if max_param > 0.0 {
        let idx = ray_params.iter().position(|&r| r == max_param).unwrap();
        return (Some(max_param), Some(idx))

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
