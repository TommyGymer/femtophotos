#[macro_use]
extern crate glium;
extern crate image;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
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

    uniform mat4 matrix;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = matrix * vec4(position, 0.0, 1.0);
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

    use std::io::Cursor;
    let image = image::load(Cursor::new(&include_bytes!("C:\\Users\\Tom\\Pictures\\20221212_135107.jpg")),
                            image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ],
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
                _ => return,
            },
            _ => (),
        }
    });
}
