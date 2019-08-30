extern crate image;
extern crate rand;
extern crate rayon;

pub mod vec3;
use vec3::Vec3;

pub mod cam;
use cam::Camera;

pub mod materials;
use materials::{RayScattering,Lambertian, Dielectric, Metal};

use image::Rgb;
use std::cmp::Ordering;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Copy, Clone)]
pub struct HitInformation<'a> {
    pub hit_point: Vec3,
    pub hit_normal: Vec3,
    pub hit_material: Option<&'a(dyn RayScattering + 'a)>,
    pub dist_from_ray_orig: f64,
}

impl<'a> HitInformation<'a> {
    pub fn zero() -> HitInformation<'a> {
        HitInformation {
            hit_point: Vec3::new(0.0, 0.0, 0.0),
            hit_normal: Vec3::new(0.0, 0.0, 0.0),
            hit_material: None,
            dist_from_ray_orig: std::f64::MAX,
        }
    }
}

pub trait Intersectable {
    fn intersect_with_ray(&self, ray: &Ray, hit_info: &mut HitInformation) -> bool;
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

pub struct Sphere<'a> {
    pub center: Vec3,
    pub radius: f64,
    pub material:Box<dyn RayScattering + 'a>,
}

impl<'a> Sphere<'a> {

    pub fn new(center: Vec3, radius: f64, material: Box<dyn RayScattering + 'a>) -> Sphere<'a>{
        let s = Sphere{center: center, radius: radius, material: material};
        return s;
    }

    ///
    /// Compute intersection of ray and sphere
    /// ray: r(t) = o + td
    /// sphere: (p-c)*(p-c) = r^2
    /// insert ray for p into sphere equation, then solve quadratic equation for t
    /// (o+td-c)(o+td-c)=r^2
    /// t1/2 = (-B +- sqrt(B^2 - 4AC))/(2A)
    ///
    /// Hitinformation has anonymous lifetime?
    fn intersect_with_ray(&self, ray: &Ray, hit_info: &mut HitInformation<'a>) -> bool {
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
            hit_info.hit_material = Some(&*self.material);
            hit_info.dist_from_ray_orig = hit_point.length();
            return true;
        }
    }
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct Scene<'a> {
    pub spheres: Vec<Sphere<'a>>,
    pub lights: Vec<Light>,
}

impl Scene<'_> {
    fn hit(&self, ray: &Ray, min_dist: f64, hit_info: &mut HitInformation) -> bool {
        let mut hit_anything = false;
        let mut hit_rec = HitInformation::zero();
        let mut closest_so_far = std::f64::MAX;
        for sphere in &self.spheres {
            if sphere.intersect_with_ray(&ray, &mut hit_rec) {
                if hit_rec.dist_from_ray_orig < closest_so_far
                    && hit_rec.dist_from_ray_orig > min_dist
                {
                    closest_so_far = hit_rec.dist_from_ray_orig;
                    hit_info.dist_from_ray_orig = hit_rec.dist_from_ray_orig;
                    hit_info.hit_material = hit_rec.hit_material;
                    hit_info.hit_normal = hit_rec.hit_normal;
                    hit_info.hit_point = hit_rec.hit_point;
                    hit_anything = true;
                }
            }
        }
        hit_anything
    }
}

pub fn colorize(ray: &Ray, scene: &Scene, bg_color: &Vec3, current_depth: u32) -> Vec3 {
    let min_dist = 0.001;
    let mut closest_hit_info = HitInformation::zero();

    if scene.hit(&ray, min_dist, &mut closest_hit_info) {
        let mut scattered_ray = Ray::zero();
        let mut attentuation = Vec3::zero();

        if current_depth > 0
            && closest_hit_info.hit_material.unwrap().scatter(
                ray,
                &closest_hit_info,
                &mut attentuation,
                &mut scattered_ray,
            )
        {
            // println!("Scattered Ray: {:?}", scattered_ray);
            let next_color = colorize(&scattered_ray, scene, bg_color, current_depth - 1);
            // println!("Attentuation {:?}, next color {:?}", attentuation, next_color);
            return attentuation * next_color;
        } else {
            let t = 0.5 * (ray.direction.y + 1.0);

            return t * Vec3::new(1.0, 1.0, 1.0) + (1.0-t) * *bg_color;
            /*
            println!("t interm: {}", t);
            return *bg_color;
            */
        }
    } else {
        let t = 0.5 * (ray.direction.y + 1.0);
        return t * Vec3::new(1.0, 1.0, 1.0) + (1.0-t) * *bg_color;
        /*
        println!("t final: {}", t);
        return *bg_color;
        */
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
        y: 0.0,
        z: 0.0,
    };
    let focal_len_mm = 50.0;

    let cam = Camera::new(cam_pos, cam_look_at, cam_up, height, width, focal_len_mm);

    let hdr_img: Vec<Vec<Vec3>> = (0..width)
        .into_par_iter() // TODO: find way to share!
        .map(|col_idx| {
            let col: Vec<Vec3> = (0..height)
                .into_par_iter() // TODO: find way to share!
                .map(|row_idx| {
                    let bg_color = Vec3 {
                        x: 0.8,
                        y: 0.8,
                        z: 0.8,
                    };

                    let mut color = Vec3::new(0.0, 0.0, 0.0);
                    for _s in 0..num_samples {
                        let ray = cam.get_ray_through_pixel(row_idx, col_idx);

                        color += colorize(&ray, &scene, &bg_color, 3);
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}