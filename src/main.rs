#[macro_use]
extern crate glium;
extern crate image;

use std::io::Cursor;

use glium::glutin::event::{ElementState, ModifiersState, VirtualKeyCode};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Debug, PartialEq)]
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

    fn to_mat(&self, d_size: (u32, u32), i_size: (u32, u32)) -> [[f32; 2]; 2] {
        match self {
            Rotation::UP => {
                if ((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32) > 1.0 {
                    [
                        [
                            (d_size.1 * i_size.0) as f32 / (d_size.0 * i_size.1) as f32,
                            0.0,
                        ],
                        [0.0, 1.0],
                    ]
                } else {
                    [
                        [1.0, 0.0],
                        [
                            0.0,
                            (d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32,
                        ],
                    ]
                }
            }
            Rotation::RIGHT => {
                if ((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32) < 1.0 {
                    [
                        [
                            0.0,
                            (d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32,
                        ],
                        [1.0, 0.0],
                    ]
                } else {
                    [
                        [0.0, 1.0],
                        [
                            (d_size.1 * i_size.1) as f32 / (d_size.0 * i_size.0) as f32,
                            0.0,
                        ],
                    ]
                }
            }
            Rotation::DOWN => {
                if ((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32) > 1.0 {
                    [
                        [
                            (d_size.1 * i_size.0) as f32 / (d_size.0 * i_size.1) as f32,
                            0.0,
                        ],
                        [0.0, -1.0],
                    ]
                } else {
                    [
                        [1.0, 0.0],
                        [
                            0.0,
                            -((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32),
                        ],
                    ]
                }
            }
            Rotation::LEFT => {
                if ((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32) < 1.0 {
                    [
                        [
                            0.0,
                            (d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32,
                        ],
                        [-1.0, 0.0],
                    ]
                } else {
                    [
                        [0.0, 1.0],
                        [
                            -((d_size.1 * i_size.1) as f32 / (d_size.0 * i_size.0) as f32),
                            0.0,
                        ],
                    ]
                }
            }
        }
    }
}

implement_vertex!(Vertex, position, tex_coords);

struct State {
    rotation: Rotation,
    directory: String,
    image_uri: String,
    modifiers: Option<ModifiersState>,
    mouse_position: Option<(u32, u32)>,
    drag_origin: Option<(u32, u32)>,
}

impl State {
    fn default() -> Self {
        Self {
            rotation: Rotation::UP,
            directory: String::from(""),
            image_uri: String::from("C:\\Users\\Tom\\Pictures\\20221212_135107.jpg"),
            modifiers: None,
            mouse_position: None,
            drag_origin: None,
        }
    }

    fn next_img(&self) {
        println!("Next");
    }

    fn prev_img(&self) {
        println!("Prev");
    }
}

fn main() {
    use glium::glutin;
    use glium::Surface;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vertex1 = Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 0.0],
    }; //bottom left
    let vertex2 = Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 0.0],
    }; //bottom right
    let vertex3 = Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
    }; //top right
    let vertex4 = Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }; //top left
    let shape = vec![vertex1, vertex2, vertex3, vertex1, vertex4, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
    #version 410
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    uniform mat2 p_rot;

    void main() {
        v_tex_coords = tex_coords;

        vec2 tmp_pos = p_rot * position;
        gl_Position = vec4(tmp_pos.x, tmp_pos.y, 0.0, 1.0);
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

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let image = image::load(
        Cursor::new(&include_bytes!(
            "C:\\Users\\Tom\\Pictures\\20221212_135107.jpg"
        )),
        image::ImageFormat::Jpeg,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let mut state = State::default();

    event_loop.run(move |ev, _, control_flow| {
        let uniforms = uniform! {
            p_rot: state.rotation.to_mat(display.get_framebuffer_dimensions(), image_dimensions),
            tex: &texture,
        };

        let mut target = display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);

        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();

        // let next_frame_time = std::time::Instant::now() +
        //     std::time::Duration::from_nanos(16_666_667);
        // *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                glutin::event::WindowEvent::ModifiersChanged(mod_state) => {
                    state.modifiers = Some(mod_state);
                }
                glutin::event::WindowEvent::CursorMoved { position, .. } => {
                    state.mouse_position = Some((position.x as u32, position.y as u32));
                }
                _ => return,
            },
            glutin::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => match event {
                glutin::event::DeviceEvent::MouseWheel { delta } => {
                    println!("{:?}", delta);
                }
                glutin::event::DeviceEvent::Button {
                    button,
                    state: button_state,
                } => match (button, button_state) {
                    (1, ElementState::Pressed) => {
                        state.drag_origin = state.mouse_position;
                    }
                    (1, ElementState::Released) => {
                        match (state.drag_origin, state.mouse_position) {
                            (Some(start), Some(end)) => {
                                if start.0 < end.0 {
                                    state.prev_img();
                                } else {
                                    state.next_img();
                                }
                            }
                            _ => return,
                        }
                    }
                    _ => return,
                },
                glutin::event::DeviceEvent::Key(k) => {
                    match (k.virtual_keycode, k.state, state.modifiers) {
                        (Some(VirtualKeyCode::R), ElementState::Pressed, Some(mods)) => {
                            if mods.contains(ModifiersState::SHIFT) {
                                state.rotation = state.rotation.anticlockwise();
                            } else {
                                state.rotation = state.rotation.clockwise();
                            }
                        }
                        (Some(VirtualKeyCode::R), ElementState::Pressed, None) => {
                            state.rotation = state.rotation.clockwise();
                        }
                        (Some(VirtualKeyCode::Space), ElementState::Pressed, None) => {
                            state.next_img();
                        }
                        (Some(VirtualKeyCode::Right), ElementState::Pressed, None) => {
                            state.next_img();
                        }
                        (Some(VirtualKeyCode::Left), ElementState::Pressed, None) => {
                            state.prev_img();
                        }
                        _ => return,
                    }
                }
                _ => return,
            },
            _ => (),
        }
    });
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clockwise_rotations() {
        assert_eq!(Rotation::UP.clockwise(), Rotation::RIGHT);
        assert_eq!(Rotation::RIGHT.clockwise(), Rotation::DOWN);
        assert_eq!(Rotation::DOWN.clockwise(), Rotation::LEFT);
        assert_eq!(Rotation::LEFT.clockwise(), Rotation::UP);
    }

    #[test]
    fn test_anticlockwise_rotations() {
        assert_eq!(Rotation::UP.anticlockwise(), Rotation::LEFT);
        assert_eq!(Rotation::RIGHT.anticlockwise(), Rotation::UP);
        assert_eq!(Rotation::DOWN.anticlockwise(), Rotation::RIGHT);
        assert_eq!(Rotation::LEFT.anticlockwise(), Rotation::DOWN);
    }
}