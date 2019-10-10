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
    pub rotation_rad: Vec3,
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
pub struct CameraBluePrint {
    pub camera_up: Vec3,
    pub camera_look_at: Vec3,
    pub camera_position: Vec3,
    pub camera_focal_length_mm: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneBlueprint {
    pub camera_blueprint: CameraBluePrint,
    pub mesh_blueprints: Vec<TriangleMeshBlueprint>,
    pub sphere_blueprints: Vec<SphereBlueprint>,
}

fn create_material_from_description(
    mat_type: &str,
    albedo: Option<Vec3>,
    material_param: Option<f64>,
) -> Option<Box<dyn RayScattering + std::marker::Sync + 'static>> {
    if mat_type.to_lowercase().contains("metal") {
        return Some(Box::new(Metal {
            albedo: albedo.expect("you forgot to specify an albedo vector for metal"),
            roughness: material_param
                .expect("you forgot to specify a roughness (i.e. material_param: 0.1) for metal"),
        }));
    } else if mat_type.to_lowercase().contains("lambert") {
        return Some(Box::new(Lambertian {
            albedo: albedo.expect("you forgot to specify an albedo vector for lambertian"),
        }));
    } else if mat_type.to_lowercase().contains("dielectric") {
        return Some(Box::new(Dielectric {
            ref_idx: material_param.expect("you forgot to specify a refractory index vector (i.e. material_param: 1.8) dielectric"),
        }));
    }
    println!(
        "Cannot figure out material_type from {}, material_type must be one of metal, lambertian or dielectric!", mat_type
    );
    return None;
}

pub fn load_blueprints_from_yaml_file(filepath: &str) -> SceneBlueprint {
    let f = File::open(filepath);
    let f = match f {
        Ok(file) => file,
        Err(error) => panic!("Failed to open {:?} to load content.", error),
    };

    let scene_bp = serde_yaml::from_reader(f);

    let scene_bp = match scene_bp {
        Ok(bp) => bp,
        Err(error) => panic!(
            "Unable to parse content of file {:?} to scene blueprint: {:?}",
            filepath, error
        ),
    };

    return scene_bp;
}

fn parse_mesh_bp(mesh_bp: TriangleMeshBlueprint) -> Option<TriangleMesh> {
    let mat_box_op = create_material_from_description(
        &mesh_bp.material_type,
        mesh_bp.albedo,
        mesh_bp.material_param,
    );
    if mat_box_op.is_some() {
        let tri_mesh = Some(TriangleMesh::new(
            &mesh_bp.obj_filepath,
            mesh_bp.translation,
            mesh_bp.rotation_rad,
            mesh_bp.scale,
            mat_box_op.unwrap(),
        ));
        return tri_mesh;
    } else {
        println!("Failed to parse material info provided with mesh!");
        return None;
    }
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
