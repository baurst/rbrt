extern crate clap;
extern crate rustracer_lib;
extern crate tobj;

use std::path::Path;

use clap::{App, Arg};
use rustracer_lib::dielectric::Dielectric;
use rustracer_lib::lambertian::Lambertian;
use rustracer_lib::metal::Metal;

use rustracer_lib::sphere::Sphere;
use rustracer_lib::triangle::Triangle;
use rustracer_lib::vec3::Vec3;
use rustracer_lib::{Intersectable, Light, Scene};

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

    let earth = Box::new(Sphere {
        center: Vec3::new(0.0, -1000.5, 0.0),
        radius: 1000.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.05, 0.2, 0.05),
        }),
    });

    let matte_sphere = Box::new(Sphere {
        center: Vec3::new(0.0, 1.0, -11.0),
        radius: 2.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.1, 0.1, 0.9),
        }),
    });

    let metal_sphere = Box::new(Sphere {
        center: Vec3::new(2.5, 1.0, -7.0),
        radius: 1.5,
        material: Box::new(Metal {
            albedo: Vec3::new(0.7, 0.7, 0.7),
            fuzz: 0.05,
        }),
    });

    let glass_sphere = Box::new(Sphere {
        center: Vec3::new(-2.5, 1.0, -7.0),
        radius: 1.5,
        material: Box::new(Dielectric { ref_idx: 1.4 }),
    });

    let light = Light {
        position: Vec3::new(100.0, 100.0, -5.0),
        color: Vec3::new(0.4, 1.0, 0.4),
    };

    let test_tri = Box::new(Triangle {
        corner_b: Vec3::new(4.0, 2.0, -4.0),
        corner_a: Vec3::new(3.0, 2.0, -4.0),
        corner_c: Vec3::new(3.0, 3.0, -4.0),
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.9, 0.2, 0.2),
        }),
    });

    let lights = vec![light];

    let mut model_elements: Vec<Box<dyn Intersectable + Sync>> = Vec::new();

    let cube = tobj::load_obj(&Path::new("cube.obj"));
    // todo: fix this hacked filepath!
    assert!(cube.is_ok());
    let (models, materials) = cube.unwrap();

    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!("Size of model[{}].indices: {}", i, mesh.indices.len());
        for f in 0..mesh.indices.len() / 3 {
            println!(
                "    idx[{}] = {}, {}, {}.",
                f,
                mesh.indices[3 * f],
                mesh.indices[3 * f + 1],
                mesh.indices[3 * f + 2]
            );
        }

        // Normals and texture coordinates are also loaded, but not printed in this example
        println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..mesh.positions.len() / 3 {
            println!(
                "    v[{}] = ({}, {}, {})",
                v,
                mesh.positions[3 * v],
                mesh.positions[3 * v + 1],
                mesh.positions[3 * v + 2]
            );
        }
    }
    let transl = Vec3::new(-2.0, 3.0, -7.0);
    let scale = 1.0;

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        // Normals and texture coordinates are also loaded, but not printed in this example
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
                println!(
                    "    triangle {}, corner {} = {:?} ",
                    f, idx, triangle_vertices[idx]
                );
            }
            let tri = Box::new(Triangle {
                corner_a: triangle_vertices[0] + transl,
                corner_b: triangle_vertices[1] + transl,
                corner_c: triangle_vertices[2] + transl,
                material: Box::new(Metal {
                    albedo: Vec3::new(0.0, 0.2, 0.9),
                    fuzz: 0.01,
                }),
            });
            model_elements.push(tri);
        }
    }

    println!("# of model_elements: {}", model_elements.len());

    let mut elements: Vec<Box<dyn Intersectable + Sync>> =
        vec![matte_sphere, glass_sphere, metal_sphere, test_tri, earth];

    elements.append(&mut model_elements);

    let scene = Scene {
        elements: elements,
        lights: lights,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
