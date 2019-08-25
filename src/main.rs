extern crate clap;
extern crate rustracer_lib;

use rustracer_lib::vec3::Vec3;
use clap::{App, Arg};
use std::fs::OpenOptions;

fn main() {
    let app = App::new("rustracer")
        .version("0.1")
        .author("baurst")
        .about("lighweight raytracer written in rust")
        .arg(
            Arg::with_name("target_file")
                .help("file that will be created witht he rendered output")
                .default_value("dbg_out.png")
                //.required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("height")
                .help("target image resolution height")
                .default_value("600")
                .index(2),
        )
        .arg(
            Arg::with_name("width")
                .help("target image resolution width")
                .default_value("800")
                .index(3),
        )
        .arg(
            Arg::with_name("samples")
                .help("number of samples to draw per pixel")
                .default_value("5")
                .index(4),
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

    let sphere = rustracer_lib::Sphere {
        center: Vec3 {
            x: 0.0,
            y: 1.0,
            z: -5.0,
        },
        radius: 1.0,
        color: Vec3 {
            x: 0.4,
            y: 1.0,
            z: 0.4,
        },
    };

    let sphere2 = rustracer_lib::Sphere {
        center: Vec3 {
            x: 1.0,
            y: 0.0,
            z: -5.0,
        },
        radius: 1.5,
        color: Vec3 {
            x: 0.9,
            y: 0.0,
            z: 0.0,
        },
    };

    let sphere3 = rustracer_lib::Sphere {
        center: Vec3 {
            x: -1.0,
            y: -1.0,
            z: -4.0,
        },
        radius: 0.5,
        color: Vec3 {
            x: 0.2,
            y: 0.2,
            z: 0.9,
        },
    };

    let light = rustracer_lib::Light {
        position: Vec3 {
            x: 0.0,
            y: 0.0,
            z: -5.0,
        },
        color: Vec3 {
            x: 0.4,
            y: 1.0,
            z: 0.4,
        },
    };

    let lights = vec![light];

    let spheres = vec![sphere, sphere2, sphere3];

    let scene = rustracer_lib::Scene {
        spheres: spheres,
        lights: lights,
    };

    let img_buf = rustracer_lib::render_scene(height, width, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
