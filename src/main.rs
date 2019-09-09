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
            albedo: Vec3::new(0.02, 0.2, 0.02),
        }),
    });

    let matte_sphere = Box::new(Sphere {
        center: Vec3::new(-3.5, 1.0, -9.0),
        radius: 1.5,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.1, 0.1, 0.9),
        }),
    });

    let metal_sphere = Box::new(Sphere {
        center: Vec3::new(-1.0, 1.5, -15.0),
        radius: 3.0,
        material: Box::new(Metal {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            fuzz: 0.05,
        }),
    });

    let glass_sphere = Box::new(Sphere {
        center: Vec3::new(2.75, 0.5, -9.0),
        radius: 1.5,
        material: Box::new(Dielectric { ref_idx: 1.4 }),
    });

    let light = Light {
        position: Vec3::new(100.0, 100.0, -7.0),
        color: Vec3::new(0.4, 1.0, 0.4),
    };

    let lights = vec![light];

    let mut model_elements: Vec<Box<dyn Intersectable + Sync>> = Vec::new();

    let cube = tobj::load_obj(&Path::new("bunny.obj"));
    // todo: fix this hacked filepath!
    assert!(cube.is_ok());
    let (models, _materials) = cube.unwrap();

    let bunny_trans = Vec3::new(6.5, -2.0, -12.0);
    let bunny_scale = 45.0;

    for (_i, m) in models.iter().enumerate() {
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
                    mesh.positions[x_idx as usize] as f64 * bunny_scale,
                    mesh.positions[y_idx as usize] as f64 * bunny_scale,
                    mesh.positions[z_idx as usize] as f64 * bunny_scale,
                );
            }
            let tri = Box::new(Triangle {
                corner_a: triangle_vertices[0] + bunny_trans,
                corner_b: triangle_vertices[1] + bunny_trans,
                corner_c: triangle_vertices[2] + bunny_trans,
                material: Box::new(Lambertian {
                    albedo: Vec3::new(0.7, 0.2, 0.2),
                }),
            });
            model_elements.push(tri);
        }
    }

    println!("Loaded {} triangles!", model_elements.len());

    let mut elements: Vec<Box<dyn Intersectable + Sync>> =
        vec![matte_sphere, glass_sphere, metal_sphere, earth];

    elements.append(&mut model_elements);

    let scene = Scene {
        elements: elements,
        lights: lights,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
