extern crate tobj;
use std::path::Path;

use crate::lambertian::Lambertian;
use crate::triangle::Triangle;
use crate::vec3::Vec3;
use crate::Intersectable;

/// Axis aligned Bounding Box
pub struct BoundingBox {
    pub lower_bound: Vec3,
    pub upper_bound: Vec3,
}

impl BoundingBox {
    pub fn new(lower_bound: Vec3, upper_bound: Vec3) -> BoundingBox {
        return BoundingBox {
            lower_bound: lower_bound,
            upper_bound: upper_bound,
        };
    }
}

pub struct TriangleMesh {
    pub triangles: Vec<Box<Triangle>>,
    pub bbox: BoundingBox,
}

impl TriangleMesh {
    pub fn new(filepath: &str, translation: Vec3, scale: f64) -> TriangleMesh {
        let mesh = load_mesh_from_file(filepath, translation, scale);
        let (mesh, lower_bound, upper_bound) = compute_min_max_3d(mesh);

        return TriangleMesh {
            triangles: mesh,
            bbox: BoundingBox::new(lower_bound, upper_bound),
        };
    }
}

pub fn compute_min_max_3d(triangle_mesh: Vec<Box<Triangle>>) -> (Vec<Box<Triangle>>, Vec3, Vec3) {
    let mut lower_bound_tmp = Vec3::new(std::f64::MAX, std::f64::MAX, std::f64::MAX);
    let mut upper_bound_tmp = Vec3::new(-std::f64::MAX, -std::f64::MAX, -std::f64::MAX);
    // wow that code stinks!
    for tri in &triangle_mesh {
        for corner in &tri.corners {
            if corner.x < lower_bound_tmp.x {
                lower_bound_tmp.x = corner.x;
            }
            if corner.y < lower_bound_tmp.y {
                lower_bound_tmp.y = corner.y;
            }
            if corner.z < lower_bound_tmp.z {
                lower_bound_tmp.z = corner.z;
            }
            if corner.x > upper_bound_tmp.x {
                upper_bound_tmp.x = corner.x;
            }
            if corner.y > upper_bound_tmp.y {
                upper_bound_tmp.y = corner.y;
            }
            if corner.z > upper_bound_tmp.z {
                upper_bound_tmp.z = corner.z;
            }
        }
    }
    (triangle_mesh, lower_bound_tmp, upper_bound_tmp)
}

pub fn load_mesh_from_file(filepath: &str, translation: Vec3, scale: f64) -> Vec<Box<Triangle>> {
    let mut model_elements: Vec<Box<Triangle>> = Vec::new();

    let loaded_mesh = tobj::load_obj(&Path::new(filepath));
    assert!(loaded_mesh.is_ok());
    let (models, _materials) = loaded_mesh.unwrap();

    for (_i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        assert!(mesh.positions.len() % 3 == 0);
        let mut triangle_vertices: Vec<Vec3> = vec![Vec3::zero(); 3];
        for f in 0..mesh.indices.len() / 3 {
            for idx in 0..3 {
                let x_idx = 3 * mesh.indices[3 * f + idx];
                let y_idx = 3 * mesh.indices[3 * f + idx] + 1;
                let z_idx = 3 * mesh.indices[3 * f + idx] + 2;

                triangle_vertices[idx] = Vec3::new(
                    mesh.positions[x_idx as usize] as f64 * scale,
                    mesh.positions[y_idx as usize] as f64 * scale,
                    mesh.positions[z_idx as usize] as f64 * scale,
                );
            }
            let tri = Box::new(Triangle {
                corners: [
                    triangle_vertices[0] + translation,
                    triangle_vertices[1] + translation,
                    triangle_vertices[2] + translation,
                ],
                material: Box::new(Lambertian {
                    albedo: Vec3::new(0.7, 0.2, 0.2),
                }),
            });
            model_elements.push(tri);
        }
    }
    println!(
        "Loaded {} triangles from file {}!",
        model_elements.len(),
        filepath
    );
    return model_elements;
}
