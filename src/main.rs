extern crate clap;
extern crate rustracer_lib;

use clap::{App, Arg};
use rustracer_lib::dielectric::Dielectric;
use rustracer_lib::lambertian::Lambertian;
use rustracer_lib::metal::Metal;

use rustracer_lib::sphere::Sphere;
use rustracer_lib::triangle::Triangle;
use rustracer_lib::vec3::Vec3;
use rustracer_lib::{Light, Scene};

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

    let earth = Sphere {
        center: Vec3::new(0.0, -1000.5, 0.0),
        radius: 1000.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.05, 0.2, 0.05),
        }),
    };

    let matte_sphere = Sphere {
        center: Vec3 {
            x: 0.0,
            y: 1.0,
            z: -11.0,
        },
        radius: 2.0,
        material: Box::new(Lambertian {
            albedo: Vec3 {
                x: 0.1,
                y: 0.1,
                z: 0.9,
            },
        }),
    };

    let metal_sphere = Sphere {
        center: Vec3 {
            x: 2.5,
            y: 1.0,
            z: -7.0,
        },
        radius: 1.5,
        material: Box::new(Metal {
            albedo: Vec3 {
                x: 0.7,
                y: 0.7,
                z: 0.7,
            },
            fuzz: 0.05,
        }),
    };

    let glass_sphere = Sphere {
        center: Vec3 {
            x: -2.5,
            y: 1.0,
            z: -7.0,
        },
        radius: 1.5,
        material: Box::new(Dielectric { ref_idx: 1.4 }),
    };

    let light = Light {
        position: Vec3 {
            x: 100.0,
            y: 100.0,
            z: -5.0,
        },
        color: Vec3 {
            x: 0.4,
            y: 1.0,
            z: 0.4,
        },
    };

    let test_tri = Triangle {
        corner_a: Vec3::new(2.0, 2.0, -5.0),
        corner_b: Vec3::new(3.0, 2.0, -5.0),
        corner_c: Vec3::new(2.0, 3.0, -5.0),
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.05, 0.2, 0.05),
        }),
    };

    let lights = vec![light];

    let spheres = vec![matte_sphere, metal_sphere, earth, glass_sphere];

    let triangles = vec![test_tri];

    let scene = Scene {
        spheres: spheres,
        lights: lights,
        triangles: triangles,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
