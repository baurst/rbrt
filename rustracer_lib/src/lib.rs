extern crate image;
extern crate rand;
extern crate rayon;

use image::{GenericImage, Pixel, Pixels, Rgb};
use std::cmp::Ordering;
use std::ops::{Add, Mul, Neg, Sub};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self * other.x,
            y: self * other.y,
            z: self * other.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: f64) -> Vec3 {
        Vec3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }
    pub fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
    }
    pub fn sum(&self) -> f64 {
        return self.x + self.y + self.z;
    }
    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        let vec = Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        };
        return vec;
    }
    fn cross_product(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        return (*self * *other).sum();
    }
}

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Camera {
    pub hor_fov_rad: f64,
    pub img_width_pix: u32,
    pub img_height_mm: f64,
    pub vert_fov_rad: f64,
    pub img_height_pix: u32,
    pub img_width_mm: f64,
    pub position: Vec3,
    pub focal_len_mm: f64,
    pub look_at: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub img_center_point: Vec3,
    pub mm_per_pix_hor: f64,
    pub mm_per_pix_vert: f64,
}

impl Camera {
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        up: Vec3,
        img_height_pix: u32,
        img_width_pix: u32,
        focal_len_mm: f64,
    ) -> Camera {
        let right = look_at
            .normalize()
            .cross_product(&up.normalize())
            .normalize();

        let img_width_mm = 60.0;
        let mm_per_pix_hor = img_width_mm / img_width_pix as f64;
        
        let img_height_mm = img_height_pix as f64 * mm_per_pix_hor;
        let mm_per_pix_vert = img_height_mm / img_height_pix as f64;

        let img_center_point = position + focal_len_mm / 1000.0 * look_at.normalize();
        let hor_fov_rad = 2.0 * (2.0 * focal_len_mm / img_width_mm as f64).atan();
        let vert_fov_rad = 2.0 * (2.0 * focal_len_mm / img_height_mm as f64).atan();

        Camera {
            hor_fov_rad: hor_fov_rad,
            img_width_pix: img_width_pix,
            img_width_mm: img_width_mm,
            vert_fov_rad: vert_fov_rad,
            img_height_pix: img_height_pix,
            img_height_mm: img_height_mm,
            position: position,
            focal_len_mm: focal_len_mm,
            up: up,
            right: right,
            look_at: look_at,
            img_center_point: img_center_point,
            mm_per_pix_hor: mm_per_pix_hor,
            mm_per_pix_vert: mm_per_pix_vert,
        }
    }

    pub fn get_ray_through_pixel_center(&self, img_row_pix: u32, img_col_pix: u32) -> Ray {
        let img_col_center_offset = img_col_pix as i32 - (self.img_width_pix / 2) as i32 ;
        let img_row_center_offset = img_row_pix as i32 -  (self.img_height_pix / 2) as i32;

        let img_col_center_offset_mm = img_col_center_offset as f64 * self.mm_per_pix_hor;
        let img_row_center_offset_mm = img_row_center_offset as f64 * self.mm_per_pix_vert;

        let ray_target_in_img_plane = self.img_center_point
            + 0.001 * img_col_center_offset_mm * self.right
            - 0.001 * img_row_center_offset_mm * self.up;
        let ray_direction = (ray_target_in_img_plane - self.position).normalize();

        let ray = Ray {
            origin: self.position,
            direction: ray_direction,
        };
        return ray;
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
    pub fn interset_w_ray(&self, ray: &Ray) -> bool {
        let A = ray.direction.dot(&ray.direction);
        let l = ray.origin - self.center;
        let B = (ray.direction * 2.0).dot(&l);
        let C = l.dot(&l) - self.radius.powf(2.0);

        let sol = B.powf(2.0) - 4.0 * A * C;

        let num_hits = match sol.partial_cmp(&0.0).expect("Encountered NAN") {
            Ordering::Less => 0,
            Ordering::Greater => 2,
            Ordering::Equal => 1,
        };

        return num_hits > 0;
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
                    let mut color_vector: Vec3 = Vec3 {
                        x: 0.5,
                        y: 0.5,
                        z: 1.0 - row_idx as f64 / height as f64,
                    };
                    let ray = cam.get_ray_through_pixel_center(row_idx, col_idx);
                    for _s in 0..num_samples {
                        for sphere in &scene.spheres {
                            if sphere.interset_w_ray(&ray)
                            {
                                color_vector = sphere.color;
                                break;
                            }
                        }
                    }
                    color_vector = color_vector * (1.0 / num_samples as f64);
                    color_vector
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
