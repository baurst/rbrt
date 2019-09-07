extern crate clap;
extern crate rustracer_lib;

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

    let earth = Sphere {
        center: Vec3::new(0.0, -1000.5, 0.0),
        radius: 1000.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.05, 0.2, 0.05),
        }),
    };

    let matte_sphere: Box<dyn Intersectable + Send + Sync> = Box::new(Sphere {
        center: Vec3::new(0.0, 1.0, -11.0),
        radius: 2.0,
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.1, 0.1, 0.9),
        }),
    });

    let metal_sphere = Sphere {
        center: Vec3::new(2.5, 1.0, -7.0),
        radius: 1.5,
        material: Box::new(Metal {
            albedo: Vec3::new(0.7, 0.7, 0.7),
            fuzz: 0.05,
        }),
    };

    let glass_sphere = Sphere {
        center: Vec3::new(-2.5, 1.0, -7.0),
        radius: 1.5,
        material: Box::new(Dielectric { ref_idx: 1.4 }),
    };

    let light = Light {
        position: Vec3::new(100.0, 100.0, -5.0),
        color: Vec3::new(0.4, 1.0, 0.4),
    };

    let test_tri: Box<dyn Intersectable + Send + Sync> = Box::new(Triangle {
        corner_a: Vec3::new(3.0, 2.0, -4.0),
        corner_b: Vec3::new(4.0, 2.0, -4.0),
        corner_c: Vec3::new(3.0, 3.0, -4.0),
        material: Box::new(Lambertian {
            albedo: Vec3::new(0.5, 0.2, 0.2),
        }),
    });

    let lights = vec![light];

    let elements: Vec<Box<dyn Intersectable>> = vec![matte_sphere, test_tri];
    /*
        , Box::new(metal_sphere), Box::new(earth), Box::new(glass_sphere), Box::new(test_tri)];
    */
    let triangles = vec![test_tri];

    let scene = Scene {
        elements: elements,
        lights: lights,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
