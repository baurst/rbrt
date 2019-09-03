extern crate image;
extern crate rand;
extern crate rayon;

pub mod dielectric;
pub mod lambertian;
pub mod metal;
pub mod vec3;

use vec3::Vec3;

pub mod cam;
use cam::Camera;

pub mod materials;
use materials::RayScattering;

use image::Rgb;
use std::cmp::Ordering;

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

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn point_at(&self, ray_param: f64) -> Vec3 {
        return self.origin + ray_param * self.direction;
    }
    pub fn zero() -> Ray {
        Ray {
            origin: Vec3::zero(),
            direction: Vec3::zero(),
        }
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Box<dyn RayScattering + Sync>,
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
    /// Hitinformation has anonymous lifetime?
    fn intersect_with_ray<'a>(&'a self, ray: &Ray) -> Option<HitInformation> {
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
            return None;
        } else {
            let mut ray_param = (-b - sol.sqrt()) / (2.0 * a);
            if num_hits == 2 && ray_param < 0.0 {
                //point is behind the camera!
                ray_param = (-b + sol.sqrt()) / (2.0 * a);
                if ray_param < 0.0 {
                    return None; // both points on the ray are negative
                }
            }

            let hit_point = ray.point_at(ray_param);
            let hit_normal = hit_point - self.center;
            let hit_info = HitInformation {
                hit_normal: hit_normal,
                hit_point: hit_point,
                hit_material: &*self.material,
                dist_from_ray_orig: (ray.origin - hit_point).length(),
            };
            if hit_point.z < 0.0 {
                //println!("Encountered hit at {:?}", hit_point);
                //assert!(false);
            }
            return Some(hit_info);
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
            // println!("Scattered Ray: {:?}", scattered_ray);
            // println!("Attentuation {:?}, next color {:?}", attentuation, next_color);
            return attentuation * colorize(&scattered_ray, scene, bg_color, current_depth - 1);
        } else {
            if current_depth > 0 {
                println!("Ray was comppetely attentuated at depth {}!", current_depth);
                println!("Material was {:?}", attentuation);
            }
            //let t = 0.5 * (ray.direction.y + 1.0);
            //return (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * *bg_color;
            return *bg_color;
        }
    } else {
        //let t = 0.5 * (ray.direction.y + 1.0);
        //return (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * *bg_color;
        /*
        println!("t final: {}", t);
        return *bg_color;
        */
        return *bg_color;
    }
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
        y: 2.0,
        z: 0.0,
    };
    let focal_len_mm = 35.0;

    let cam = Camera::new(cam_pos, cam_look_at, cam_up, height, width, focal_len_mm);

    let hdr_img: Vec<Vec<Vec3>> = (0..width)
        .into_par_iter() // TODO: find way to share!
        .map(|col_idx| {
            let col: Vec<Vec3> = (0..height)
                .into_par_iter() // TODO: find way to share!
                .map(|row_idx| {
                    let bg_color = Vec3 {
                        x: 0.7,
                        y: 0.7,
                        z: 0.9,
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
