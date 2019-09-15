# Rustracer
This is yet another raytracer written in Rust. Coming from C++, I have been experimenting a lot with Rust lately and thought it could be fun to write a small but capable little raytracer.
The starting point for this project was the excellent book ['Raytracing In One Weekend'](https://raytracing.github.io) by Peter Shirley as well as ssloys awesome [tinyraytracer](https://github.com/ssloy/tinyraytracer).

## Try it out!
```bash
git clone https://github.com/baurst/rustracer.git
cd rustracer
cargo build --release

# download the stanford bunny
wget http://graphics.stanford.edu/~mdfisher/Data/Meshes/bunny.obj    

# run the raytracer: adjust samples according to your needs for time & quality
./target/release/rustracer --target_file out.png --height 768 --width 1024 --samples 50
```


## Other Assets

Cube:
https://people.sc.fsu.edu/~jburkardt/data/obj/cube.obj

## To Dos:
* configure scene via yaml or json
