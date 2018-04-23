use std::fmt;
use std::fmt::Formatter;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;

use byteorder::{LE, ReadBytesExt};

use luminance::buffer::Buffer;

#[derive(Clone, Copy, Debug)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    const BLACK : Colour = Colour {r: 0, g: 0, b: 0};
}

const PALETTE_SIZE : usize = 256;
struct VoxelModel {
    width : u32,
    height : u32,
    depth : u32,
    palette: [Colour; PALETTE_SIZE],
    data: Vec<u8>,
}

impl Debug for VoxelModel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VoxelModel {}×{}×{}", self.width, self.height, self.depth)
    }
}

impl VoxelModel {
    fn index(&self, x : u32, y: u32, z: u32) -> Result<usize, ()> {
        if x >= self.width || y >= self.height || z >= self.depth {
            Err(())
        } else {
            Ok((z + y * self.depth + x * self.height * self.depth) as usize)
        }
    }

    pub fn at(&self, x : u32, y: u32, z: u32) -> Result<Option<Colour>, ()> {
        let index = self.index(x, y, z)?;
        let value = self.data[index];
        Ok(if value == 255 { None } else { Some(self.palette[self.data[index] as usize]) })
    }

    pub fn polygonise(&self) -> Vec<Quad> {
        let mut result = Vec::new();

        for z in 0..self.depth {
            let mut this_layer = Vec::new();
            for y in 0..self.height {
                for x in 0..self.width {
                    let index = self.index(x, y, z).expect("polygonise tried to access OOB position!?");
                    let colour = self.at(x, y, z).expect("polygonise tried to access OOB position!?");
                    colour.map(|c| {
                        let quad = Quad {
                            vertices: [
                                Vertex {x: x as f32, y: y as f32, z: z as f32},
                                Vertex {x: x as f32, y: y as f32, z: z as f32},
                                Vertex {x: x as f32, y: y as f32, z: z as f32},
                                Vertex {x: x as f32, y: y as f32, z: z as f32},
                            ],
                            colour: c,
                            side: Direction::North,
                        };
                        this_layer.push(quad);
                    });
                }
            }
            for quad in this_layer {
                result.push(quad);
            }
        };

        result
    }
}

#[derive(Debug)]
struct Vertex {
    x : f32,
    y : f32,
    z : f32,
}
impl Vertex {
    const ORIGIN: Vertex = Vertex {x: 0., y: 0., z: 0.};
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    East,
    West,
    North,
    South,
}

impl Direction {
    fn translate(&self, v: &mut Vertex, amount: f32) {
        match self {
            Direction::Up => v.y += amount,
            Direction::Down => v.y -= amount,
            Direction::East => v.x += amount,
            Direction::West => v.x -= amount,
            Direction::North => v.z += amount,
            Direction::South => v.z -= amount,
        };
    }
}

fn make_quad(side: Direction, x: u32, y: u32, z: u32) -> [Vertex; 4] {
    let x = x as f32;
    let y = y as f32;
    let z = z as f32;
    let v = |x, y, z| Vertex { x: x, y: y, z: z };
    match side {
        Direction::Up    => [v(   x, 1.+y,    z), v(1.+x, 1.+y,    z), v(1.+x, 1.+y, 1.+z), v(   x, 1.+y, 1.+z)],
        Direction::Down  => [v(   x,    y,    z), v(   x,    y, 1.+z), v(1.+x,    y, 1.+z), v(1.+x,    y,    z)],
        Direction::East  => [v(1.+x,    y,    z), v(1.+x,    y, 1.+z), v(1.+x, 1.+y, 1.+z), v(1.+x, 1.+y,    z)],
        Direction::West  => [v(   x,    y,    z), v(   x, 1.+y,    z), v(   x, 1.+y, 1.+z), v(   x,    y, 1.+z)],
        Direction::North => [v(   x,    y, 1.+z), v(   x, 1.+y, 1.+z), v(1.+x, 1.+y, 1.+z), v(1.+x,    y, 1.+z)],
        Direction::South => [v(   x,    y,    z), v(1.+x,    y,    z), v(1.+x, 1.+y,    z), v(   x, 1.+y,    z)],
    }
}

#[derive(Debug)]
struct Quad {
    vertices : [Vertex; 4],
    colour : Colour,
    side : Direction,
}

#[derive(Debug)]
struct Triangle {
    vertices: [Vertex; 3],
    colour: Colour,
}

/*
struct RenderableVoxelModel {
    buffer: Buffer<f32>,
}
*/

fn load_model(stream : &mut Read) -> ::std::io::Result<VoxelModel> {
    let mut file_reader = BufReader::new(stream);
    let mut voxel_model = VoxelModel {
        width: 0, height: 0, depth: 0,
        palette: [Colour::BLACK; 256],
        data: Vec::new()
    };

    voxel_model.width = file_reader.read_u32::<LE>()?;
    voxel_model.height = file_reader.read_u32::<LE>()?;
    voxel_model.depth = file_reader.read_u32::<LE>()?;

    voxel_model.data = vec![0; (voxel_model.width * voxel_model.height * voxel_model.depth) as usize];
    file_reader.read_exact(voxel_model.data.as_mut_slice())?;

    for i in 0..PALETTE_SIZE {
        voxel_model.palette[i].r = file_reader.read_u8()? << 2;
        voxel_model.palette[i].g = file_reader.read_u8()? << 2;
        voxel_model.palette[i].b = file_reader.read_u8()? << 2;
    }

    Ok(voxel_model)
}

pub fn main() {
    let mut model_file = File::open("minotaur_head.vox");
    let model = model_file.and_then(|mut f| load_model(&mut f)).unwrap();
    let polys = model.polygonise();
    println!("{:?}", model);
    println!("{:?} quads", polys.len());
}
