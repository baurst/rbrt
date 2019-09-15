extern crate clap;
extern crate rustracer_lib;

use clap::{App, Arg};
use rustracer_lib::dielectric::Dielectric;
use rustracer_lib::lambertian::Lambertian;
use rustracer_lib::metal::Metal;
use rustracer_lib::triangle::Triangle;

use rustracer_lib::sphere::Sphere;
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
        center: Vec3::new(0.5, 1.5, -9.0),
        radius: 2.0,
        material: Box::new(Dielectric { ref_idx: 1.4 }),
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
        let bunny_trans = Vec3::new(5.5, -2.0, -12.0);
        let bunny_scale = 45.0;
        loaded_meshes.push(rustracer_lib::mesh_io::TriangleMesh::new(
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
