extern crate clap;
extern crate rustracer_lib;
use clap::{App, Arg};

use rustracer_lib::blueprints::{
    create_scene_from_scene_blueprint, load_blueprints_from_yaml_file,
};

use rustracer_lib::cam::Camera;

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
                .help("YAML file that specifies the scene layout and camera specification.")
                .default_value("scenes/example_scene.yaml"),
        )
        .arg(
            Arg::with_name("samples")
                .short("s")
                .long("samples")
                .help("number of rays per pixel")
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

    let scene_bp = load_blueprints_from_yaml_file(&config_file);
    let cam = Camera::new(
        scene_bp.camera_blueprint.camera_position,
        scene_bp.camera_blueprint.camera_look_at,
        scene_bp.camera_blueprint.camera_up,
        height,
        width,
        scene_bp.camera_blueprint.camera_focal_length_mm,
    );

    let scene = create_scene_from_scene_blueprint(scene_bp);

    let img_buf = rustracer_lib::render_scene(cam, num_samples, scene);

    img_buf.save(target_image_path).unwrap();
}
