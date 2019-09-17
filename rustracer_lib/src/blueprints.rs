use crate::dielectric::Dielectric;
use crate::lambertian::Lambertian;
use crate::materials::RayScattering;
use crate::mesh::TriangleMesh;
use crate::metal::Metal;

use crate::sphere::Sphere;
use crate::vec3::Vec3;
use crate::{Intersectable, Scene};

use std::fs::File;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TriangleMeshBlueprint {
    pub obj_filepath: String,
    pub scale: f64,
    pub translation: Vec3,
    pub material_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SphereBlueprint {
    pub radius: f64,
    pub center: Vec3,
    pub material_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneBlueprint {
    pub mesh_blueprints: Vec<TriangleMeshBlueprint>,
    pub sphere_blueprints: Vec<SphereBlueprint>,
}

fn get_albedo_vec_from_descr(descr: &str) -> Option<Vec3> {
    let albedo_vec = descr
        .split(";")
        .filter(|s| s.contains("albedo"))
        .last()
        .unwrap()
        .split(":")
        .last()
        .unwrap()
        .replace(&['(', ')', ' '][..], "")
        .split(",")
        .filter_map(|s| s.parse::<f64>().ok())
        .collect::<Vec<_>>();

    if albedo_vec.len() == 3 {
        return Some(Vec3::new(albedo_vec[0], albedo_vec[1], albedo_vec[2]));
    } else {
        println!(
            "An error occured while trying to figure out albedo vector from {}",
            descr
        );
        return None;
    }
}

fn get_scalar_from_descr(descr: &str, scalar_name: &str) -> Option<f64> {
    let relevant_parts = descr
        .split(";")
        .filter(|s| s.contains(&scalar_name))
        .last()
        .unwrap()
        .split(":")
        .last()
        .unwrap()
        .replace(&['(', ')', ' '][..], "")
        .parse::<f64>()
        .ok();
    return relevant_parts;
}

fn create_material_from_description(
    descr: &str,
) -> Option<Box<dyn RayScattering + std::marker::Sync + 'static>> {
    if descr.contains("metal") {
        let albedo_vec_op = get_albedo_vec_from_descr(descr);
        let roughness_op = get_scalar_from_descr(descr, &"roughness".to_string());

        if albedo_vec_op.is_some() & roughness_op.is_some() {
            return Some(Box::new(Metal {
                albedo: albedo_vec_op.unwrap(),
                roughness: roughness_op.unwrap(),
            }));
        }
    } else if descr.contains("lambert") {
        let albedo_vec_op = get_albedo_vec_from_descr(descr);

        if albedo_vec_op.is_some() {
            return Some(Box::new(Lambertian {
                albedo: albedo_vec_op.unwrap(),
            }));
        }
    } else if descr.contains("dielectric") {
        let ref_idx_op = get_scalar_from_descr(descr, &"ref_idx".to_string());
        if ref_idx_op.is_some() {
            return Some(Box::new(Dielectric {
                ref_idx: ref_idx_op.unwrap(),
            }));
        }
    }
    return None;
}

pub fn load_blueprints_from_yaml_file(filepath: &str) -> SceneBlueprint {
    let f = File::open(filepath).unwrap();
    let scene_bp: SceneBlueprint = serde_yaml::from_reader(f).unwrap();
    return scene_bp;
}

fn parse_mesh_bp(mesh_bp: TriangleMeshBlueprint) -> Option<TriangleMesh> {
    let _mat_box_op = create_material_from_description(&mesh_bp.material_description);

    let tri_mesh = Some(TriangleMesh::new(
        &mesh_bp.obj_filepath,
        mesh_bp.translation,
        mesh_bp.scale,
    ));
    return tri_mesh;
}

fn parse_sphere_bp(sphere_bp: SphereBlueprint) -> Option<Sphere> {
    let mat_box_op = create_material_from_description(&sphere_bp.material_description);

    if mat_box_op.is_some() {
        return Some(Sphere {
            center: sphere_bp.center,
            radius: sphere_bp.radius,
            material: mat_box_op.unwrap(),
        });
    } else {
        return None;
    }
}

pub fn create_scene_from_scene_blueprint(scene_bp: SceneBlueprint) -> Scene {
    let mut loaded_meshes = vec![];
    for mesh_bp in scene_bp.mesh_blueprints {
        let tri_mesh_op = parse_mesh_bp(mesh_bp);
        if tri_mesh_op.is_some() {
            loaded_meshes.push(tri_mesh_op.unwrap());
        }
    }

    let mut scene_elements: Vec<
        std::boxed::Box<(dyn Intersectable + std::marker::Sync + 'static)>,
    > = vec![];
    for sphere_bp in scene_bp.sphere_blueprints {
        let sphere_op = parse_sphere_bp(sphere_bp);
        if sphere_op.is_some() {
            scene_elements.push(Box::new(sphere_op.unwrap()));
        }
    }

    let lights = vec![];

    return Scene {
        triangle_meshes: loaded_meshes,
        elements: scene_elements,
        lights: lights,
    };
}

#[cfg(test)]
mod tests {
    use super::{get_albedo_vec_from_descr, get_scalar_from_descr, Vec3};
    #[test]
    fn test_material_descr_parsing() {
        let material_description = "material: lambertian; albedo: (1.0,2.0,3.0)".to_string();
        let albedo = get_albedo_vec_from_descr(&material_description);
        assert_eq!(albedo.unwrap(), Vec3::new(1.0, 2.0, 3.0));
    }
    #[test]
    fn test_material_descr_parsing_w_scalar() {
        let material_description =
            "material: dielectric; albedo: (1.0,0.0,0.0); ref_idx: 1.7".to_string();
        let albedo = get_albedo_vec_from_descr(&material_description);
        assert_eq!(albedo.unwrap(), Vec3::new(1.0, 0.0, 0.0));
    }
    #[test]
    fn test_get_ref_idx() {
        let material_description =
            "material: dielectric; albedo: (1.0,0.0,0.0); ref_idx: 1.7".to_string();
        let ref_idx_name = "ref_idx".to_string();
        let ref_idx = get_scalar_from_descr(&material_description, &ref_idx_name).unwrap();
        assert_eq!(ref_idx, 1.7)
    }

}
