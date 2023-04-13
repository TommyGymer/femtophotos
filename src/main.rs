#[macro_use]
extern crate glium;
extern crate image;

use std::io::Cursor;

use glium::glutin::event::{VirtualKeyCode, ElementState, ModifiersState};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Debug)]
enum Rotation {
    UP,
    RIGHT,
    DOWN,
    LEFT,
}

impl Rotation {
    fn clockwise(&self) -> Rotation {
        match self {
            Rotation::UP => Rotation::RIGHT,
            Rotation::RIGHT => Rotation::DOWN,
            Rotation::DOWN => Rotation::LEFT,
            Rotation::LEFT => Rotation::UP,
        }
    }

    fn anticlockwise(&self) -> Rotation {
        match self {
            Rotation::UP => Rotation::LEFT,
            Rotation::RIGHT => Rotation::UP,
            Rotation::DOWN => Rotation::RIGHT,
            Rotation::LEFT => Rotation::DOWN,
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Rotation::UP => 0u32,
            Rotation::RIGHT => 1u32,
            Rotation::DOWN => 2u32,
            Rotation::LEFT => 3u32,
        }
    }
}

implement_vertex!(Vertex, position, tex_coords);

fn main() {
    use glium::glutin;
    use glium::Surface;

    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vertex1 = Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] };  //bottom left
    let vertex2 = Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] };   //bottom right
    let vertex3 = Vertex { position: [ 1.0, 1.0], tex_coords: [1.0, 1.0] };     //top right
    let vertex4 = Vertex { position: [ -1.0, 1.0], tex_coords: [0.0, 1.0] };   //top left
    let shape = vec![vertex1, vertex2, vertex3, vertex1, vertex4, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    
    let vertex_shader_src = r#"
    #version 410
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    uniform uint rot;
    uniform double i_aspr;
    uniform double d_aspr;

    void main() {
        v_tex_coords = tex_coords;
        if (rot == 0) {
            if (d_aspr > i_aspr) {
                gl_Position = vec4(position.x / (d_aspr / i_aspr), position.y, 0.0, 1.0);
            } else {
                gl_Position = vec4(position.x, position.y * (d_aspr / i_aspr), 0.0, 1.0);
            }
        } else if (rot == 1) {
            if (d_aspr < (1 / i_aspr)) {
                gl_Position = vec4(position.y, position.x * (d_aspr * i_aspr), 0.0, 1.0);
            } else {
                gl_Position = vec4(position.y / (d_aspr * i_aspr), position.x, 0.0, 1.0);
            }
        } else if (rot == 2) {
            if (d_aspr > i_aspr) {
                gl_Position = vec4(position.x / (d_aspr / i_aspr), -position.y, 0.0, 1.0);
            } else {
                gl_Position = vec4(position.x, -position.y * (d_aspr / i_aspr), 0.0, 1.0);
            }
        } else {
            if (d_aspr < (1 / i_aspr)) {
                gl_Position = vec4(-position.y, position.x * (d_aspr * i_aspr), 0.0, 1.0);
            } else {
                gl_Position = vec4(-position.y / (d_aspr * i_aspr), position.x, 0.0, 1.0);
            }
        }
    }
    "#;

    let fragment_shader_src = r#"
    #version 410
    in vec2 v_tex_coords;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = texture(tex, v_tex_coords);
    }
    "#;

    let image = image::load(Cursor::new(&include_bytes!("C:\\Users\\Tom\\Pictures\\20221212_135107.jpg")),
                            image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let image_aspr: f64 = image.width as f64 / image.height as f64;

    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut rotation = Rotation::UP;

    event_loop.run(move |ev, _, control_flow| {
        let display_aspr: f64 = display.get_framebuffer_dimensions().0 as f64 / display.get_framebuffer_dimensions().1 as f64;
        
        // if (display_aspr < image_aspr) {
        //     println!("d_aspr < i_aspr");
        // } else if (display_aspr > image_aspr) {
        //     println!("d_aspr > i_aspr");
        // } else {
        //     println!("d_aspr = i_aspr");
        // }

        let uniforms = uniform! {
            rot: rotation.to_u32(),
            i_aspr: image_aspr,
            d_aspr: display_aspr,
            tex: &texture,
        };

        let mut target = display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();

        target.finish().unwrap();

        // let next_frame_time = std::time::Instant::now() +
        //     std::time::Duration::from_nanos(16_666_667);
        // *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::ModifiersChanged(state) => {
                    println!("{:?}", state);
                    return;
                },
                _ => return,
            },
            glutin::event::Event::DeviceEvent { device_id, event } => match event {
                glutin::event::DeviceEvent::MouseMotion { delta } => {
                    println!("{:?}", delta);
                    return;
                },
                glutin::event::DeviceEvent::MouseWheel { delta } => {
                    println!("{:?}", delta);
                    return;
                },
                glutin::event::DeviceEvent::Button { button, state } => {
                    println!("{:?}: {:#?}", button, state);
                    return;
                },
                glutin::event::DeviceEvent::Key(k) => {
                    match k.virtual_keycode {
                        Some(VirtualKeyCode::R) => {
                                if (k.state == ElementState::Pressed) {
                                    if (k.modifiers.contains(ModifiersState::SHIFT)) {
                                        rotation = rotation.anticlockwise();
                                    } else {
                                        rotation = rotation.clockwise();
                                    }
                                    println!("rotated!");
                                    println!("{:?}", rotation);
                                }
                                println!("{:?}", k);
                            },
                        _ => println!("{:?}", k),
                    }
                    return;
                },
                _ => return,
            }
            _ => (),
        }
    });
}
