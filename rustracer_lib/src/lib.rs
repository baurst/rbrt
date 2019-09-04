extern crate image;
extern crate rand;
extern crate rayon;

pub mod dielectric;
pub mod lambertian;
pub mod metal;
pub mod ray;
pub mod sphere;
pub mod vec3;

use ray::Ray;
use sphere::Sphere;

use vec3::Vec3;

pub mod cam;
use cam::Camera;

pub mod materials;
use materials::RayScattering;

use image::Rgb;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Copy, Clone)]
pub struct HitInformation<'a> {
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_material: &'a dyn RayScattering,
    pub dist_from_ray_orig: f64,
}

pub trait Intersectable {
    fn intersect_with_ray(&self, ray: &Ray) -> Option<HitInformation>;
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

impl Scene {
    fn hit<'a>(&'a self, ray: &Ray, min_dist: f64, max_dist: f64) -> Option<HitInformation> {
        let mut closest_hit_rec = None;
        let mut closest_so_far = std::f64::MAX;

        for sphere in &self.spheres {
            let hit_info_op = sphere.intersect_with_ray(&ray);
            if hit_info_op.is_some() {
                let hit_rec = hit_info_op.unwrap();
                if hit_rec.dist_from_ray_orig < closest_so_far
                    && hit_rec.dist_from_ray_orig > min_dist
                    && hit_rec.dist_from_ray_orig < max_dist
                {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    closest_hit_rec = Some(hit_rec);
                }
            }
        }
        return closest_hit_rec;
    }
}

pub fn colorize(ray: &Ray, scene: &Scene, bg_color: &Vec3, current_depth: u32) -> Vec3 {
    let min_dist = 0.001;
    let max_dist = 2000.0;

    let hit_opt = scene.hit(&ray, min_dist, max_dist);

    if hit_opt.is_some() {
        let closest_hit_info = hit_opt.unwrap();
        let mut scattered_ray = Ray::zero();
        let mut attentuation = Vec3::zero();

        if current_depth > 0
            && closest_hit_info.hit_material.scatter(
                ray,
                &closest_hit_info,
                &mut attentuation,
                &mut scattered_ray,
            )
        {
            return attentuation * colorize(&scattered_ray, scene, bg_color, current_depth - 1);
        } else {
            // ray was completely attentuated
            return Vec3::zero();
        }
    } else {
        let t = 0.5 * (ray.direction.y + 1.0); // t=[0,1]
        return t * Vec3::new(1.0, 1.0, 1.0) + (1.0 - t) * *bg_color;
    }
}

pub fn render_scene(
    height: u32,
    width: u32,
    num_samples: u32,
    scene: Scene,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let cam_up = Vec3::new(0.0, 1.0, 0.4).normalize();
    let cam_look_at = Vec3::new(0.0, -0.3, -1.0).normalize();
    let cam_pos = Vec3::new(0.0, 6.0, 4.0);
    let focal_len_mm = 35.0;

    let cam = Camera::new(cam_pos, cam_look_at, cam_up, height, width, focal_len_mm);

    let hdr_img: Vec<Vec<Vec3>> = (0..width)
        .into_par_iter()
        .map(|col_idx| {
            let col: Vec<Vec3> = (0..height)
                .into_par_iter()
                .map(|row_idx| {
                    let bg_color = Vec3 {
                        x: 0.2,
                        y: 0.2,
                        z: 0.8,
                    };

                    let mut color = Vec3::new(0.0, 0.0, 0.0);
                    for _s in 0..num_samples {
                        let ray = cam.get_ray_through_pixel(row_idx, col_idx);

                        color += colorize(&ray, &scene, &bg_color, 50);
                    }
                    color = color * (1.0 / num_samples as f64);
                    //println!("{:#?}",color );
                    color
                })
                .collect();
            col
        })
        .collect();

    let mut imgbuf = image::ImageBuffer::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (hdr_img[x as usize][y as usize].x.sqrt() * 256.0) as u8;
        let g = (hdr_img[x as usize][y as usize].y.sqrt() * 256.0) as u8;
        let b = (hdr_img[x as usize][y as usize].z.sqrt() * 256.0) as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    return imgbuf;
}
