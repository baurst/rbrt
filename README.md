# Rustracer
This is yet another raytracer written in Rust. Coming from C++, Rusts ownership concepts (and easy way of managing dependencies and cross-platform builds) fascinated me and so I thought it could be fun to write a lightweight but capable raytracer to get to know the language better.
The inspiration for this project came from the excellent book ['Raytracing In One Weekend'](https://raytracing.github.io) by Peter Shirley as well as ssloys awesome [tinyraytracer](https://github.com/ssloy/tinyraytracer).

## Try it out!
```bash
git clone https://github.com/baurst/rustracer.git
cd rustracer
cargo build --release

# download the stanford bunny
wget http://graphics.stanford.edu/~mdfisher/Data/Meshes/bunny.obj    

# run the raytracer: adjust samples according to your needs
./target/release/rustracer --target_file out.png --height 768 --width 1024 --samples 50 --config scenes/example_scene.yaml
```

## Coordinate System
The raytracer uses a right-handed coordinate system, with negative z pointing towards the scene.

## Configuring the scene
For an example scene configuration check out scenes/example_scene.yaml.
Triangle meshes can be loaded by providing a path to an .obj file, spheres can also be added to the config yaml.
For each scene element, a material definition needs to be specified in the config yaml, for example like so: 
* for metal: "material: metal; albedo: (0.8, 0.8, 0.8); roughness: 0.005"
* for a matte, lambertian material: "material: lambertian; albedo: (0.02,0.2,0.02)"
* for a transparent dielectric material: "material: dielectric; ref_idx: 1.8"

All of these features can be seen in the example_scene.yaml file in the config folder.
