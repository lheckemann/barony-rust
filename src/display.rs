use std::f32::consts::{FRAC_PI_6, FRAC_PI_3};

use cgmath;
use cgmath::{Matrix4, Vector3, Vector4, PerspectiveFov};

use glium;
use glium::Surface;

use glutin;
use glutin::Event;
use glutin::WindowEvent;

use graphics;

struct State {
    exit : bool,
    t : u32,
}

fn handle_event(ev: glutin::Event, state: &mut State) {
    match ev {
        glutin::Event::WindowEvent { event, .. } => { match event {
            WindowEvent::Closed => { state.exit = true; },
            _ => (),
        }},
        _ => ()
    }
}

const VERTEX_SHADER_SRC: &'static str = r#"
    #version 330

    uniform uint time;
    uniform mat4 transproj;

    in vec3 position;
    in vec3 colour;
    out vec3 col;

    void main() {
        gl_Position = transproj * vec4(position, 1.0);
        col = colour * (1.0/256);
    }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 col;
    out vec4 color;

    void main() {
        color = vec4(col, 1.0);
    }
"#;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    colour: [u8; 3],
}

pub fn main() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let w = 1920;
    let h = 1080;
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(w, h)
        .with_min_dimensions(w, h)
        .with_max_dimensions(w, h)
        .with_title("Hello, world");
    let context = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let mut state = State {
        exit: false,
        t: 0,
    };

    implement_vertex!(Vertex, position, colour);
    let minotaur = graphics::minotaur();
    let mut minotaur_verts = Vec::new();
    for quad in minotaur.polygonise() {
        let colour: [u8; 3] = [quad.colour.r, quad.colour.g, quad.colour.b];
        let vert = |v: graphics::Vertex| Vertex {
            position: [v.x, v.y, v.z],
            colour: colour,
        };
        minotaur_verts.push(vert(quad.vertices[0]));
        minotaur_verts.push(vert(quad.vertices[1]));
        minotaur_verts.push(vert(quad.vertices[2]));
        minotaur_verts.push(vert(quad.vertices[2]));
        minotaur_verts.push(vert(quad.vertices[3]));
        minotaur_verts.push(vert(quad.vertices[0]));
    };
    println!("{} verts", minotaur_verts.len());
    let minotaur_buffer = glium::VertexBuffer::immutable(&display, &minotaur_verts).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();
    let proj : Matrix4<f32> = (PerspectiveFov {
        fovy: cgmath::Deg(120.).into(),
        aspect: (w as f32)/(h as f32),
        near: 0.1,
        far: 1000.
    }).into();
    let proj = proj * Matrix4::from_nonuniform_scale(1., 1., -1.);
    let scale : Matrix4<f32> = Matrix4::from_scale(9.);
    let trans1 : Matrix4<f32> = Matrix4::from_translation(Vector3::new(0., 0., 200.));
    let mut rot : Matrix4<f32> = cgmath::One::one();
    let dims = Vector3::new(minotaur.width as f32, minotaur.height as f32, minotaur.depth as f32);
    let trans2 : Matrix4<f32> = Matrix4::from_translation(dims * -0.5);

    rot = rot * Matrix4::from_angle_x(cgmath::Deg(90.));
    rot = rot * Matrix4::from_angle_z(cgmath::Deg(-90.));

    let mut draw_params: glium::DrawParameters = Default::default();
    draw_params.depth.test = glium::draw_parameters::DepthTest::IfLess;
    draw_params.depth.write = true;
    let draw_params = draw_params;

    let s = |t : f32| ((t.sin() + 1.0) / 2.0);
    while !state.exit {
        let mut target = display.draw();
        let t = (state.t as f32) / 80.;
        //target.clear_color(s(t), s(t + FRAC_PI_3), s(t + (2. * FRAC_PI_3)), 1.0);
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        rot = rot * Matrix4::from_angle_x(cgmath::Rad(0.02));
        rot = rot * Matrix4::from_angle_y(cgmath::Rad(0.03));
        rot = rot * Matrix4::from_angle_z(cgmath::Rad(0.05));
        let uniforms = uniform! {
            time: state.t,
            transproj: cgmath::conv::array4x4(proj * trans1 * rot * scale * trans2),
        };
        target.draw(&minotaur_buffer, &indices, &program, &uniforms, &draw_params).unwrap();
        target.finish().unwrap();
        events_loop.poll_events(|ev| handle_event(ev, &mut state));
        state.t += 1;
    };
}
