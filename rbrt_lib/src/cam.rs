use crate::vec3::Vec3;
use crate::Ray;

pub struct Camera {
    pub hor_fov_rad: f32,
    pub img_width_pix: u32,
    pub img_height_mm: f32,
    pub vert_fov_rad: f32,
    pub img_height_pix: u32,
    pub img_width_mm: f32,
    pub position: Vec3,
    pub focal_len_mm: f32,
    pub look_at: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub img_center_point: Vec3,
    pub mm_per_pix_hor: f32,
    pub mm_per_pix_vert: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        up: Vec3,
        img_height_pix: u32,
        img_width_pix: u32,
        focal_len_mm: f32,
    ) -> Camera {
        let right = look_at
            .normalize()
            .cross_product(&up.normalize())
            .normalize();

        // full frame sensor so that focal length is intuitive
        let img_width_mm = 35.0;
        let mm_per_pix_hor = img_width_mm / img_width_pix as f32;

        let img_height_mm = img_height_pix as f32 * mm_per_pix_hor;
        let mm_per_pix_vert = img_height_mm / img_height_pix as f32;

        let img_center_point = position + focal_len_mm / 1000.0 * look_at.normalize();
        let hor_fov_rad = 2.0 * (2.0 * focal_len_mm / img_width_mm).atan();
        let vert_fov_rad = 2.0 * (2.0 * focal_len_mm / img_height_mm).atan();

        Camera {
            hor_fov_rad,
            img_width_pix,
            img_width_mm,
            vert_fov_rad,
            img_height_pix,
            img_height_mm,
            position,
            focal_len_mm,
            up,
            right,
            look_at,
            img_center_point,
            mm_per_pix_hor,
            mm_per_pix_vert,
        }
    }

    pub fn get_ray_through_pixel(&self, img_row_pix: u32, img_col_pix: u32) -> Ray {
        let img_col_center_offset = img_col_pix as f32 - (self.img_width_pix / 2) as f32;
        let img_row_center_offset = img_row_pix as f32 - (self.img_height_pix / 2) as f32;

        let img_col_center_offset_mm =
            (img_col_center_offset + rand::random::<f32>() - 0.5) * self.mm_per_pix_hor;
        let img_row_center_offset_mm =
            (img_row_center_offset + rand::random::<f32>() - 0.5) * self.mm_per_pix_vert;

        let ray_target_in_img_plane = self.img_center_point
            + 0.001 * img_col_center_offset_mm * self.right
            - 0.001 * img_row_center_offset_mm * self.up;
        let ray_direction = (ray_target_in_img_plane - self.position).normalize();

        Ray {
            origin: self.position,
            direction: ray_direction,
        }
    }
}
