use crate::Ray;
use crate::Vec3;

/// Axis aligned Bounding Box
pub struct BoundingBox {
    pub lower_bound: Vec3,
    pub upper_bound: Vec3,
}

/// just for better readabilty
pub fn max(a: f32, b: f32) -> f32 {
    a.max(b)
}

pub fn min(a: f32, b: f32) -> f32 {
    a.min(b)
}

impl BoundingBox {
    pub fn new(lower_bound: Vec3, upper_bound: Vec3) -> BoundingBox {
        BoundingBox {
            lower_bound,
            upper_bound,
        }
    }
    /// for explanation see https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
    /// see also https://gamedev.stackexchange.com/questions/18436/most-efficient-aabb-vs-ray-collision-algorithms
    pub fn hit(&self, ray: &Ray) -> bool {
        // get ray parameters that show where the ray intersects the box planes
        let t_lower_x = (self.lower_bound.x - ray.origin.x) / ray.direction.x;
        let t_upper_x = (self.upper_bound.x - ray.origin.x) / ray.direction.x;
        let t_lower_y = (self.lower_bound.y - ray.origin.y) / ray.direction.y;
        let t_upper_y = (self.upper_bound.y - ray.origin.y) / ray.direction.y;
        let t_lower_z = (self.lower_bound.z - ray.origin.z) / ray.direction.z;
        let t_upper_z = (self.upper_bound.z - ray.origin.z) / ray.direction.z;

        let t_min_x = min(t_lower_x, t_upper_x);
        let t_min_y = min(t_lower_y, t_upper_y);
        let t_min_z = min(t_lower_z, t_upper_z);
        // look for the biggest lower intersection across all dimensions
        let t_min = max(max(t_min_x, t_min_y), t_min_z);

        let t_max_x = max(t_lower_x, t_upper_x);
        let t_max_y = max(t_lower_y, t_upper_y);
        let t_max_z = max(t_lower_z, t_upper_z);
        // look for the smallest upper intersection across all dimensions
        let t_max = min(min(t_max_x, t_max_y), t_max_z);

        // intersection, but opposite to ray direction
        if t_max < 0.0 {
            return false;
        }

        if t_min > t_max {
            return false;
        }
        true
    }
}

/// computes the axis aligned bounding box extents of triangles
pub fn compute_min_max_3d(triangle_mesh: &[[Vec3; 3]]) -> (Vec3, Vec3) {
    let mut lower_bound_tmp = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut upper_bound_tmp = Vec3::new(-f32::MAX, -f32::MAX, -f32::MAX);
    for tri in triangle_mesh {
        for vertex in tri.iter().take(3) {
            if vertex.x < lower_bound_tmp.x {
                lower_bound_tmp.x = vertex.x;
            }
            if vertex.y < lower_bound_tmp.y {
                lower_bound_tmp.y = vertex.y;
            }
            if vertex.z < lower_bound_tmp.z {
                lower_bound_tmp.z = vertex.z;
            }
            if vertex.x > upper_bound_tmp.x {
                upper_bound_tmp.x = vertex.x;
            }
            if vertex.y > upper_bound_tmp.y {
                upper_bound_tmp.y = vertex.y;
            }
            if vertex.z > upper_bound_tmp.z {
                upper_bound_tmp.z = vertex.z;
            }
        }
    }
    (lower_bound_tmp, upper_bound_tmp)
}

#[cfg(test)]
mod tests {
    use super::{compute_min_max_3d, Vec3};

    #[test]
    fn test_mesh_aabbox() {
        let test_tri = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];

        let tris = vec![test_tri];

        let (lower_bound, upper_bound) = compute_min_max_3d(&tris);

        assert_eq!(lower_bound, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(upper_bound, Vec3::new(1.0, 1.0, 1.0));
    }
}
