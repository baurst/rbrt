use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
    }
}
impl AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, other: &Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
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

impl PartialEq for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x: x, y: y, z: z }
    }
    pub fn zero() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
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
    pub fn cross_product(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// rotate a point Z,X,Z
    /// angles in radians
    pub fn rotate_point(&self, r_x: f64, r_y: f64, r_z: f64) -> Vec3 {
        let s_x = r_x.sin();
        let s_y = r_y.sin();
        let s_z = r_z.sin();

        let c_x = r_x.cos();
        let c_y = r_y.cos();
        let c_z = r_z.cos();

        let (x, y, z) = (self.x, self.y, self.z);

        let res = Vec3::new(
            (c_x * c_z - c_y * s_x * s_z) * x - (c_x * s_z + c_y * c_z * s_x) * y + s_x * s_y * z,
            (c_z * s_x + c_x * c_y * s_z) * x + (c_x * c_y * c_z - s_x * s_z) * y - c_x * s_y * z,
            s_y * s_z * x + c_z * s_y * y + c_y * z,
        );
        return res;
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        return (*self * *other).sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn test_cross_product() {
        assert_eq!(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0).cross_product(&Vec3::new(0.0, 1.0, 0.0))
        );
    }
    #[test]
    fn test_normalize() {
        assert_eq!(1.0, Vec3::new(5.0, 2.0, 3.0).normalize().length());
    }
    #[test]
    fn test_dot_product() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0).dot(&Vec3::new(1.0, 2.0, 3.0)),
            14.0
        );
    }
    #[test]
    fn test_mul() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0) * Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(1.0, 4.0, 9.0)
        );
    }
    #[test]
    fn test_add() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0) + Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(2.0, 4.0, 6.0)
        );
    }
    #[test]
    fn test_subtract() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0) - Vec3::new(1.0, 2.0, 3.0),
            Vec3::zero()
        );
    }

    #[test]
    fn test_rotate_yaw() {
        let a = 0.7071067657322372;
        let f_delta = 0.000001;

        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point(0.0, 0.0, (45.0 as f32).to_radians() as f64)
                - Vec3::new(a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point(0.0, 0.0, (-45.0 as f32).to_radians() as f64)
                - Vec3::new(a, -a, 0.0))
            .length()
                < f_delta
        );

        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point(0.0, 0.0, (45.0 as f32).to_radians() as f64)
                - Vec3::new(-a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point(0.0, 0.0, (-45.0 as f32).to_radians() as f64)
                - Vec3::new(a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point(0.0, 0.0, (45.0 as f32).to_radians() as f64)
                - Vec3::new(0.0, 0.0, 1.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point(0.0, 0.0, (-45.0 as f32).to_radians() as f64)
                - Vec3::new(0.0, 0.0, 1.0))
            .length()
                < f_delta
        );
    }
    #[test]
    fn test_rotate_pitch() {
        let a = 0.7071067657322372;
        let f_delta = 0.000001;

        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point(0.0, (45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(1.0, 0.0, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point(0.0, (-45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(1.0, 0.0, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point(0.0, (45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(0.0, a, a))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point(0.0, (-45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(0.0, a, -a))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point(0.0, (45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(0.0, -a, a))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point(0.0, (-45.0 as f32).to_radians() as f64, 0.0)
                - Vec3::new(0.0, a, a))
            .length()
                < f_delta
        );
    }
    #[test]
    fn test_rotate_roll() {
        let a = 0.7071067657322372;
        let f_delta = 0.000001;
        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point((45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(1.0, 0.0, 0.0).rotate_point((-45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(a, -a, 0.0))
            .length()
                < f_delta
        );

        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point((45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(-a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 1.0, 0.0).rotate_point((-45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(a, a, 0.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point((45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(0.0, 0.0, 1.0))
            .length()
                < f_delta
        );
        assert!(
            (Vec3::new(0.0, 0.0, 1.0).rotate_point((-45.0 as f32).to_radians() as f64, 0.0, 0.0)
                - Vec3::new(0.0, 0.0, 1.0))
            .length()
                < f_delta
        );
    }
}
