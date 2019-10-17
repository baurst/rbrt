extern crate image;
extern crate rand;
extern crate rayon;

pub mod aabbox;
pub mod blueprints;
pub mod cam;
pub mod dielectric;
pub mod lambertian;
pub mod materials;
pub mod mesh;
pub mod metal;
pub mod ray;
pub mod scene;
pub mod sphere;
pub mod triangle;
pub mod vec3;

use cam::Camera;
use image::Rgb;
use materials::RayScattering;
use ray::Ray;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use scene::Scene;
use std::sync::atomic::{AtomicUsize, Ordering};
use vec3::Vec3;

#[derive(Copy, Clone)]
pub struct HitInformation<'a> {
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_material: &'a dyn RayScattering,
    pub dist_from_ray_orig: f32,
}

pub trait Intersectable: Sync {
    fn intersect_with_ray(&self, ray: &Ray, min_dist: f32, max_dist: f32)
        -> Option<HitInformation>;
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
    cam: Camera,
    num_samples: u32,
    scene: Scene,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    println!("Starting rendering...");

    let progress = AtomicUsize::new(0);

    let hdr_img: Vec<Vec<Vec3>> = (0..cam.img_width_pix)
        .into_par_iter()
        .map(|col_idx| {
            let col: Vec<Vec3> = (0..cam.img_height_pix)
                //.into_par_iter()
                .into_iter()
                .map(|row_idx| {
                    let bg_color = Vec3 {
                        x: 0.05,
                        y: 0.05,
                        z: 0.8,
                    };

                    let mut color = Vec3::zero();
                    for _s in 0..num_samples {
                        let ray = cam.get_ray_through_pixel(row_idx, col_idx);

                        color += colorize(&ray, &scene, &bg_color, 50);
                    }
                    color = color * (1.0 / num_samples as f32);
                    color
                })
                .collect();
            let prog = progress.fetch_add(1, Ordering::SeqCst);

            print!(
                "\rRendering {:.1}% complete!",
                prog as f32 / cam.img_width_pix as f32 * 100.0
            );
            col
        })
        .collect();
    println!("\rRendering 100% complete!");

    let mut imgbuf = image::ImageBuffer::new(cam.img_width_pix, cam.img_height_pix);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (hdr_img[x as usize][y as usize].x.sqrt() * 256.0) as u8;
        let g = (hdr_img[x as usize][y as usize].y.sqrt() * 256.0) as u8;
        let b = (hdr_img[x as usize][y as usize].z.sqrt() * 256.0) as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    return imgbuf;
}
