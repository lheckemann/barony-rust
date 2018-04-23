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

        let add_quad = |layer : &mut Vec<Quad>, dir : Direction, x, y, z| {
            let colour = self.at(x, y, z).expect("polygonise tried to access OOB position!?");
            colour.map(|c| {
                // Get the voxel "in front" of this one
                let neigh_pos = dir.step(x, y, z);
                let neigh = self.at(neigh_pos[0], neigh_pos[1], neigh_pos[2]);
                // Only add the quad if it doesn't have another voxel blocking its view
                if neigh.unwrap_or(None).is_some() { return; }
                let quad = Quad {
                    vertices: make_quad(dir, x, y, z),
                    colour: c,
                    side: dir,
                };
                layer.push(quad);
            });
        };

        for x in 0..self.width {
            let mut east_layer = Vec::new();
            let mut west_layer = Vec::new();
            for y in 0..self.height {
                for z in 0..self.depth {
                    add_quad(&mut east_layer, Direction::East, x, y, z);
                    add_quad(&mut west_layer, Direction::West, x, y, z);
                }
            }
            result.extend(east_layer);
            result.extend(west_layer);
        }
        for y in 0..self.height {
            let mut up_layer = Vec::new();
            let mut down_layer = Vec::new();
            for x in 0..self.width {
                for z in 0..self.depth {
                    add_quad(&mut up_layer, Direction::Up, x, y, z);
                    add_quad(&mut down_layer, Direction::Down, x, y, z);
                }
            }
            result.extend(up_layer);
            result.extend(down_layer);
        }
        for z in 0..self.depth {
            let mut north_layer = Vec::new();
            let mut south_layer = Vec::new();
            for y in 0..self.height {
                for x in 0..self.width {
                    add_quad(&mut north_layer, Direction::North, x, y, z);
                    add_quad(&mut south_layer, Direction::South, x, y, z);
                }
            }
            result.extend(north_layer);
            result.extend(south_layer);
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

#[derive(Debug, Clone, Copy)]
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
    fn step(&self, x: u32, y: u32, z: u32) -> [u32; 3] {
        match self {
            Direction::Up => [x, y+1, z],
            Direction::Down => [x, y.wrapping_sub(1), z],
            Direction::East => [x+1, y, z],
            Direction::West => [x.wrapping_sub(1), y, z],
            Direction::North => [x, y, z+1],
            Direction::South => [x, y, z.wrapping_sub(1)],
        }
    }
}

fn make_quad(side: Direction, x: u32, y: u32, z: u32) -> [Vertex; 4] {
    let x1 = x + 1;
    let y1 = y + 1;
    let z1 = z + 1;
    let v = |x, y, z| Vertex { x: x as f32, y: y as f32, z: z as f32};
    match side {
        Direction::Up    => [v( x, y1,  z), v(x1, y1,  z), v(x1, y1, z1), v( x, y1, z1)],
        Direction::Down  => [v( x,  y,  z), v( x,  y, z1), v(x1,  y, z1), v(x1,  y,  z)],
        Direction::East  => [v(x1,  y,  z), v(x1,  y, z1), v(x1, y1, z1), v(x1, y1,  z)],
        Direction::West  => [v( x,  y,  z), v( x, y1,  z), v( x, y1, z1), v( x,  y, z1)],
        Direction::North => [v( x,  y, z1), v( x, y1, z1), v(x1, y1, z1), v(x1,  y, z1)],
        Direction::South => [v( x,  y,  z), v(x1,  y,  z), v(x1, y1,  z), v( x, y1,  z)],
    }
}

#[derive(Debug)]
struct Quad {
    vertices : [Vertex; 4],
    colour : Colour,
    side : Direction,
}

/*
#[derive(Debug)]
struct Triangle {
    vertices: [Vertex; 3],
    colour: Colour,
}

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
    let model_file = File::open("minotaur_head.vox");
    let model = model_file.and_then(|mut f| load_model(&mut f)).unwrap();
    let polys = model.polygonise();
    println!("{:?}", model);
    println!("{} quads", polys.len());
}
