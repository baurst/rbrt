extern crate tobj;
use std::path::Path;

use crate::lambertian::Lambertian;
use crate::triangle::Triangle;
use crate::vec3::Vec3;
use crate::Intersectable;

pub fn load_mesh_from_file(
    filepath: &str,
    translation: Vec3,
    scale: f64,
) -> Vec<Box<dyn Intersectable + Sync>> {
    let mut model_elements: Vec<Box<dyn Intersectable + Sync>> = Vec::new();

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
                corners: [triangle_vertices[0] + translation,
                triangle_vertices[1] + translation,
                triangle_vertices[2] + translation],
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
