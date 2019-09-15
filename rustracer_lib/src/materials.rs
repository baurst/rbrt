use crate::vec3::Vec3;
use crate::{HitInformation, Ray};

pub trait RayScattering {
    fn scatter(
        &self,
        incoming_ray: &Ray,
        hit_info: &HitInformation,
        attentuation: &mut Vec3,
        scattered_ray: &mut Ray,
    ) -> bool;
}

pub fn random_point_in_unit_sphere() -> Vec3 {
    let mut point =
        2.0 * Vec3::new(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        ) - Vec3::new(1.0, 1.0, 1.0);
    while point.length() > 1.0 {
        point =
            2.0 * Vec3::new(
                rand::random::<f64>(),
                rand::random::<f64>(),
                rand::random::<f64>(),
            ) - Vec3::new(1.0, 1.0, 1.0);
    }
    return point;
}

pub fn reflect(incoming_ray_dir: &Vec3, normal: &Vec3) -> Vec3 {
    let inc_ray_dir_unit = incoming_ray_dir.normalize();
    let normal_unit = normal.normalize();
    let reflected_dir = inc_ray_dir_unit - 2.0 * normal_unit * inc_ray_dir_unit.dot(&normal_unit);
    return reflected_dir.normalize();
}

#[cfg(test)]
mod tests {
    use super::{random_point_in_unit_sphere, reflect, Vec3};
    #[test]
    fn test_random_points_in_unit_sphere() {
        for _i in 0..20 {
            assert!(random_point_in_unit_sphere().length() < 1.0);
        }
    }
    #[test]
    fn test_reflection() {
        let incoming = Vec3::new(1.0, 1.0, 1.0);
        let normal = Vec3::new(1.0, 1.0, 1.0);
        let refl = reflect(&incoming, &normal);
        assert_eq!(refl, -1.0 * incoming.normalize());

        let incoming = Vec3::new(1.0, 1.0, 0.0);
        let normal = Vec3::new(-1.0, 0.0, 0.0);
        let refl = reflect(&incoming, &normal);
        assert_eq!(
            refl,
            Vec3::new(-0.7071067811865476, 0.7071067811865476, 0.0)
        );
    }

}
