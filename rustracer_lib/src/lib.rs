extern crate image;
extern crate rand;
extern crate rayon;

pub mod vec3;
use vec3::Vec3;

pub mod cam;
use cam::Camera;

pub mod materials;
use materials::random_point_in_unit_sphere;

use image::Rgb;
use std::cmp::Ordering;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Copy, Clone, Debug)]
pub struct HitInformation {
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_color: Vec3,
    pub dist_from_cam: f64,
}

impl HitInformation {
    pub fn zero() -> HitInformation {
        HitInformation {
            hit_point: Vec3::new(0.0, 0.0, 0.0),
            hit_normal: Vec3::new(0.0, 0.0, 0.0),
            hit_color: Vec3::new(0.0, 0.0, 0.0),
            dist_from_cam: std::f64::MAX,
        }
    }
}

pub trait Intersectable {
    fn intersect_with_ray(&self, ray: &Ray, hit_info: &mut HitInformation) -> bool;
}


pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn point_at(&self, ray_param: f64) -> Vec3 {
        return self.origin + ray_param * self.direction;
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub color: Vec3,
    pub radius: f64,
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
    fn intersect_with_ray(&self, ray: &Ray, hit_info: &mut HitInformation) -> bool {
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
            return false;
        } else {
            let ray_param = (-b - sol.sqrt()) / (2.0 * a);
            let hit_point = ray.point_at(ray_param);
            let hit_normal = hit_point - self.center;
            hit_info.hit_normal = hit_normal;
            hit_info.hit_point = hit_point;
            hit_info.hit_color = self.color;
            hit_info.dist_from_cam = hit_point.length();
            return true;
        }
    }
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

pub fn render_scene(
    height: u32,
    width: u32,
    num_samples: u32,
    scene: Scene,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let cam_up = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    let cam_look_at = Vec3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };
    let cam_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let focal_len_mm = 50.0;

    let cam = Camera::new(cam_pos, cam_look_at, cam_up, height, width, focal_len_mm);

    let hdr_img: Vec<Vec<Vec3>> = (0..width)
        .into_par_iter()
        .map(|col_idx| {
            let col: Vec<Vec3> = (0..height)
                .into_par_iter()
                .map(|row_idx| {
                    let bg_color = Vec3 {
                        x: 0.5,
                        y: 0.5,
                        z: 1.0 - row_idx as f64 / height as f64,
                    };

                    let mut color = Vec3::new(0.0, 0.0, 0.0);
                    for _s in 0..num_samples {
                        let ray = cam.get_ray_through_pixel(row_idx, col_idx);
                        let mut hit_info = HitInformation::zero();
                        let mut closest_hit_info = HitInformation::zero();
                        for sphere in &scene.spheres {
                            if sphere.intersect_with_ray(&ray, &mut hit_info) {
                                if hit_info.dist_from_cam < closest_hit_info.dist_from_cam {
                                    closest_hit_info = hit_info
                                }
                            }
                        }
                        if closest_hit_info.dist_from_cam < std::f64::MAX {
                            color += ray
                                .direction
                                .dot(&closest_hit_info.hit_normal.normalize())
                                .abs()
                                * closest_hit_info.hit_color;
                        } else {
                            color += bg_color;
                        }
                    }
                    color = color * (1.0 / num_samples as f64);
                    color
                })
                .collect();
            col
        })
        .collect();

    let mut imgbuf = image::ImageBuffer::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (hdr_img[x as usize][y as usize].x * 255.0) as u8;
        let g = (hdr_img[x as usize][y as usize].y * 255.0) as u8;
        let b = (hdr_img[x as usize][y as usize].z * 255.0) as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    return imgbuf;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
