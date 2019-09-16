extern crate clap;
extern crate rustracer_lib;

use clap::{App, Arg};
use rustracer_lib::dielectric::Dielectric;
use rustracer_lib::lambertian::Lambertian;
use rustracer_lib::materials::RayScattering;
use rustracer_lib::mesh::TriangleMesh;
use rustracer_lib::metal::Metal;
use rustracer_lib::triangle::Triangle;

use rustracer_lib::sphere::Sphere;
use rustracer_lib::vec3::Vec3;
use rustracer_lib::{Intersectable, Light, Scene};

//extern crate serde_yaml;
//extern crate serde;
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;

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

fn create_material_from_description(
    descr: &str,
) -> Option<Box<dyn RayScattering + std::marker::Sync + 'static>> {
    if descr.contains("metal") {
        println!("Metal!");
        return Some(Box::new(Metal {
            albedo: Vec3::new(1.0, 1.0, 1.0),
            fuzz: 0.005,
        }));
    } else if descr.contains("lambert") {
        println!("Lambert!");
        return Some(Box::new(Lambertian {
            albedo: Vec3::new(1.0, 1.0, 1.0),
        }));
    } else if descr.contains("dielectric") {
        println!("Glass!");
        return Some(Box::new(Dielectric { ref_idx: 1.7 }));
    }
    return None;
}

fn load_blueprints_from_yaml_file(filepath: &str) -> SceneBlueprint {
    use std::fs::File;
    let f = File::open(filepath).unwrap();
    let scene_bp: SceneBlueprint = serde_yaml::from_reader(f).unwrap();
    println!("{:?}", scene_bp);
    return scene_bp;
}

fn parse_mesh_bp(mesh_bp: TriangleMeshBlueprint) -> Option<TriangleMesh> {
    let transl = Vec3::zero();
    let scale = 0.0;
    let tri_mesh = Some(TriangleMesh::new(&mesh_bp.obj_filepath, transl, scale));
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

fn scene_from_scene_bp(scene_bp: SceneBlueprint) -> Scene {
    let mut loaded_meshes = vec![];
    for mesh_bp in scene_bp.mesh_blueprints {
        let tri_mesh_op = parse_mesh_bp(mesh_bp);
        if tri_mesh_op.is_some() {
            loaded_meshes.push(tri_mesh_op.unwrap());
        }
    }

    let mut scene_elements = vec![];
    for sphere_bp in scene_bp.sphere_blueprints {
        let sphere_op = parse_sphere_bp(sphere_bp);
        if sphere_op.is_some() {
            scene_elements.push(sphere_op.unwrap());
        }
    }

    let elements = vec![];
    let lights = vec![];

    return Scene {
        triangle_meshes: loaded_meshes,
        elements: elements,
        lights: lights,
    };
}

fn main() {
    let app = App::new("rustracer")
        .version("0.1")
        .author("baurst")
        .about("a lighweight raytracer written in rust")
        .arg(
            Arg::with_name("target_file")
                .short("t")
                .long("target_file")
                .help("file that will be created witht he rendered output")
                .default_value("dbg_out.png"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .help("target image resolution height")
                .default_value("600"),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .help("target image resolution width")
                .default_value("800"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("YAML file that specifies the scene layout.")
                .default_value("./scenes/example.yaml"),
        )
        .arg(
            Arg::with_name("dry_run")
                .short("d")
                .long("dry-run")
                .help("performs very fast dry run without expensive meshes"),
        )
        .arg(
            Arg::with_name("samples")
                .short("s")
                .long("samples")
                .help("number of samples to draw per pixel")
                .default_value("5"),
        );
    let matches = app.get_matches();

    let target_image_path = matches.value_of("target_file").unwrap();
    let height: u32 = matches
        .value_of("height")
        .unwrap()
        .parse::<u32>()
        .expect("Please provide valid height!");
    let width: u32 = matches
        .value_of("width")
        .unwrap()
        .parse::<u32>()
        .expect("Please provide valid width!");
    let num_samples: u32 = matches
        .value_of("samples")
        .unwrap()
        .parse::<u32>()
        .expect("Please provide valid number of samples per pixel!");
    let config_file: String = matches
        .value_of("config")
        .unwrap()
        .parse::<String>()
        .expect("Please specify a valid scene layout yaml file!");

    //let blah = load_blueprints_from_yaml_file(&config_file);

    let test_bp = SphereBlueprint {
        radius: 6.0,
        center: Vec3::new(3.0, 4.0, 5.0),
        material_description: "material: lambertian; albedo: (1.0,0.0,0.0)".to_string(),
    };

    let test_bp2 = SphereBlueprint {
        radius: 6.0,
        center: Vec3::new(3.0, 4.0, 5.0),
        material_description: "material: Metal, albedo: (1.0,0.0,0.0), fuzz: 0.005".to_string(),
    };

    let bunny_trans = Vec3::new(5.0, -2.0, -12.5);
    let bunny_scale = 45.0;

    let test_mesh = TriangleMeshBlueprint {
        obj_filepath: "bunny.obj".to_string(),
        scale: 45.0,
        translation: Vec3::new(5.0, -2.0, -12.5),
        material_description: "material: lambertian; albedo: (1.0,0.0,0.0)".to_string(),
    };

    let mesh_bps = vec![test_mesh];
    let sphere_blueprints = vec![test_bp, test_bp2];

    let sbp = SceneBlueprint {
        mesh_blueprints: mesh_bps,
        sphere_blueprints: sphere_blueprints,
    };

    let s = serde_yaml::to_string(&sbp).unwrap();
    println!("{:?}", s);

    let scene_bp_result: Result<SceneBlueprint, _> = serde_yaml::from_str(&s);
    let scene_bp = scene_bp_result.unwrap(); // make this nice!
    let scene = scene_from_scene_bp(scene_bp);

    // todo: create scene from scene bp

    let is_dry_run = match matches.occurrences_of("dry_run") {
        0 => false,
        1 | _ => {
            println!("Rendering a dry run, fast but no meshes!");
            true
        }
    };

    let earth = Box::new(Sphere {
        center: Vec3::new(0.0, -1000.5, 0.0),
        radius: 1000.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.02, 0.2, 0.02),
        }),
    });

    let matte_sphere = Box::new(Sphere {
        center: Vec3::new(-5.0, 1.0, -9.0),
        radius: 1.5,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.1, 0.1, 0.9),
        }),
    });

    let metal_sphere = Box::new(Sphere {
        center: Vec3::new(-2.5, 2.8, -15.0),
        radius: 3.0,
        material: Box::new(Metal {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            fuzz: 0.005,
        }),
    });

    let glass_sphere = Box::new(Sphere {
        center: Vec3::new(1.5, 0.8, -9.0),
        radius: 1.5,
        material: Box::new(Dielectric { ref_idx: 1.8 }),
    });

    let light = Light {
        position: Vec3::new(100.0, 100.0, -7.0),
        color: Vec3::new(0.4, 1.0, 0.4),
    };

    let lights = vec![light];

    let mut loaded_meshes = vec![];
    let mut test_tris: Vec<Box<dyn Intersectable + Sync>> = vec![];
    if !is_dry_run {
        let fp = "bunny.obj";
        let bunny_trans = Vec3::new(5.0, -2.0, -12.5);
        let bunny_scale = 45.0;
        loaded_meshes.push(rustracer_lib::mesh::TriangleMesh::new(
            fp,
            bunny_trans,
            bunny_scale,
        ));
    } else {
        let test_tri = Box::new(Triangle {
            corners: [
                Vec3::new(-2.0, 1.0, -7.0),
                Vec3::new(0.0, 2.0, -7.0),
                Vec3::new(-1.0, 1.0, -7.0),
            ],
            material: Box::new(Lambertian {
                albedo: Vec3::new(0.5, 0.2, 0.2),
            }),
        });
        test_tris.push(test_tri);
    }

    let mut elements: Vec<Box<dyn Intersectable + Sync>> =
        vec![matte_sphere, glass_sphere, metal_sphere, earth];
    elements.append(&mut test_tris);

    let scene = Scene {
        triangle_meshes: loaded_meshes,
        elements: elements,
        lights: lights,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
