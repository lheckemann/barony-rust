extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate glutin;
extern crate byteorder;
mod graphics;
mod display;

fn main() {
    display::main();
}
