extern crate clap;
extern crate rbrt_lib;
use clap::{Arg, Command};

use rbrt_lib::blueprints::{create_scene_from_scene_blueprint, load_blueprints_from_yaml_file};

use rbrt_lib::cam::Camera;

fn main() {
    let app = Command::new("rbrt")
        .version("0.1")
        .author("baurst")
        .about("a lighweight raytracer written in rust")
        .arg(
            Arg::new("target_file")
                .short('t')
                .long("target_file")
                .help("file that will be created witht he rendered output")
                .default_value("dbg_out.png"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .help("target image resolution height")
                .default_value("600")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .help("target image resolution width")
                .default_value("800")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("YAML file that specifies the scene layout and camera specification.")
                .default_value("scenes/example_scene.yaml"),
        )
        .arg(
            Arg::new("samples")
                .short('s')
                .long("samples")
                .help("number of rays per pixel")
                .default_value("5")
                .value_parser(clap::value_parser!(u32)),
        );
    let matches = app.get_matches();

    let target_image_path = matches
        .get_one::<String>("target_file")
        .expect("Please provide a valid target file path!");

    let height = matches
        .get_one::<u32>("height")
        .expect("Please provide valid height!");
    let width = matches
        .get_one::<u32>("width")
        .expect("Please provide valid width!");
    let num_samples = matches
        .get_one::<u32>("samples")
        .expect("Please provide valid number of samples per pixel!");
    let config_file = matches
        .get_one::<String>("config")
        .expect("Please specify a valid scene layout yaml file!");

    let scene_bp = load_blueprints_from_yaml_file(config_file);
    let cam = Camera::new(
        scene_bp.camera_blueprint.camera_position,
        scene_bp.camera_blueprint.camera_look_at,
        scene_bp.camera_blueprint.camera_up,
        *height,
        *width,
        scene_bp.camera_blueprint.camera_focal_length_mm,
    );

    let scene = create_scene_from_scene_blueprint(scene_bp);

    let img_buf = rbrt_lib::render_scene(cam, *num_samples, scene);

    println!("Saving rendered image to {}", target_image_path);

    img_buf.save(target_image_path).unwrap_or_else(|_| {
        panic!(
            "Unable to save target img to {}! Maybe the directory does not exist?",
            target_image_path
        )
    });
}
