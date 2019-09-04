use crate::vec3::Vec3;
use crate::{HitInformation, Ray, RayScattering};

pub struct Triangle {
    pub corner_a: Vec3,
    pub corner_b: Vec3,
    pub corner_c: Vec3,
    pub material: Box<dyn RayScattering + Sync>,
}

impl Triangle {
    pub fn intersect_with_ray<'a>(&'a self, ray: &Ray) -> Option<HitInformation> {
        return None;
    }

    /*
    pub fn new(corner_a: Vec3, corner_b: Vec3, corner_c: Vec3, material: RayScattering)-> Triangle{
        return Triangle{
            corner_a: corner_a, corner_b: corner_b, corner_c: corner_c, material: Box::new(material),
        }
    }
    */
}
