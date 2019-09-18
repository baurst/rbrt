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
    pub material_type: String,
    pub albedo: Option<Vec3>,
    pub material_param: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SphereBlueprint {
    pub radius: f64,
    pub center: Vec3,
    pub material_type: String,
    pub albedo: Option<Vec3>,
    pub material_param: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneBlueprint {
    pub mesh_blueprints: Vec<TriangleMeshBlueprint>,
    pub sphere_blueprints: Vec<SphereBlueprint>,
}

fn create_material_from_description(
    mat_type: &str,
    albedo: Option<Vec3>,
    material_param: Option<f64>,
) -> Option<Box<dyn RayScattering + std::marker::Sync + 'static>> {
    if mat_type.contains("metal") {
        return Some(Box::new(Metal {
            albedo: albedo.unwrap(),
            roughness: material_param.unwrap(),
        }));
    } else if mat_type.contains("lambert") {
        return Some(Box::new(Lambertian {
            albedo: albedo.unwrap(),
        }));
    } else if mat_type.contains("dielectric") {
        return Some(Box::new(Dielectric {
            ref_idx: material_param.unwrap(),
        }));
    }
    println!(
        "Cannot figure out material_type from {}, material_type must be one of metal, lambertian or dielectric!", mat_type
    );
    return None;
}

pub fn load_blueprints_from_yaml_file(filepath: &str) -> SceneBlueprint {
    let f = File::open(filepath).unwrap();
    let scene_bp: SceneBlueprint = serde_yaml::from_reader(f).unwrap();
    return scene_bp;
}

fn parse_mesh_bp(mesh_bp: TriangleMeshBlueprint) -> Option<TriangleMesh> {
    let _mat_box_op = create_material_from_description(
        &mesh_bp.material_type,
        mesh_bp.albedo,
        mesh_bp.material_param,
    );

    let tri_mesh = Some(TriangleMesh::new(
        &mesh_bp.obj_filepath,
        mesh_bp.translation,
        mesh_bp.scale,
    ));
    return tri_mesh;
}

fn parse_sphere_bp(sphere_bp: SphereBlueprint) -> Option<Sphere> {
    let mat_box_op = create_material_from_description(
        &sphere_bp.material_type,
        sphere_bp.albedo,
        sphere_bp.material_param,
    );

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
mod tests {}
