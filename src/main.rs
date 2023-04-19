#[macro_use]
extern crate glium;
extern crate image;
extern crate exif;

mod rotation;
mod image_loading;
mod state;
use rfd::FileDialog;
use state::State;

use std::{path::Path, time::Instant, ffi::OsString, env};
use glium::{glutin::{event::{ElementState, ModifiersState, VirtualKeyCode}, window::Icon}, texture::SrgbTexture2d, Display};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

fn load_texture(display: &Display, state: &State) -> Result<(SrgbTexture2d, (u32, u32)), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let image = image_loading::load_image(Path::new(&state.image_uri))?;
    let image_size = (image.width, image.height);
    println!("image loaded: {:?}", start.elapsed());
    let texture = glium::texture::SrgbTexture2d::new(display, image)?;
    println!("texture loaded: {:?}", start.elapsed());
    Ok((texture, image_size))
}

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    dbg!(&args);

    use glium::glutin;
    use glium::Surface;

    let event_loop = glutin::event_loop::EventLoop::new();
    let icon = match image_loading::icon() {
        Ok((data, (width, height))) => Some(Icon::from_rgba(data, width, height).unwrap()),
        Err(_) => None,
    };
    let wb = glutin::window::WindowBuilder::new().with_title("FemtoPhotos: ").with_transparent(true).with_window_icon(icon);
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

    let mut state = State::default();
    state.load_img();

    let (mut texture, mut image_size) = match load_texture(&display, &state) {
        Ok(res) => res,
        Err(err) => panic!("{:?}", err),
    };

    event_loop.run(move |ev, _, control_flow| {
        if state.image_changed && state.running {
            (texture, image_size) = match load_texture(&display, &state) {
                Ok(res) => res,
                Err(err) => panic!("{:?}", err),
            };

            display.gl_window().window().set_title(&format!("FemtoPhotos: {}", Path::new(&state.image_uri).file_name().unwrap().to_str().unwrap()));

            state.image_changed = false;
        }

        let uniforms = uniform! {
            p_rot: state.rotation.to_mat(display.get_framebuffer_dimensions(), image_size),
            tex: &texture,
        };

        let mut target = display.draw();
        target.clear_color(0.2, 0.2, 0.2, 0.2);

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

        // println!("{:?}", ev);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    state.running = false;
                    return;
                },
                glutin::event::WindowEvent::ModifiersChanged(mod_state) => {
                    if mod_state.is_empty() {
                        state.modifiers = None;
                    } else {
                        state.modifiers = Some(mod_state);
                    }
                },
                glutin::event::WindowEvent::CursorMoved { position, .. } => {
                    state.mouse_position = Some((position.x as u32, position.y as u32));
                },
                glutin::event::WindowEvent::CursorLeft { .. } => {
                    state.mouse_position = None;
                },
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
                        },
                        (Some(VirtualKeyCode::R), ElementState::Pressed, None) => {
                            state.rotation = state.rotation.clockwise();
                        },
                        (Some(VirtualKeyCode::Space), ElementState::Pressed, None) => {
                            state.next_img();
                        },
                        (Some(VirtualKeyCode::Right), ElementState::Pressed, None) => {
                            state.next_img();
                        },
                        (Some(VirtualKeyCode::Left), ElementState::Pressed, None) => {
                            state.prev_img();
                        },
                        (Some(VirtualKeyCode::S,), ElementState::Released, None) => {
                            println!("Saving");

                            let file = FileDialog::new()
                                    .set_directory(Path::new(&state.directory))
                                    .set_file_name(&state.image_uri)
                                    .add_filter("JPG", &["jpg", "JPG", "jpeg", "JPEG"])
                                    .add_filter("PNG", &["png", "PNG"])
                                    .add_filter("QOI", &["qoi", "QOI"])
                                    .save_file();

                            println!("{:?}", file);
                        },
                        _ => return, //println!("returned {:?}", k),
                    }
                }
                _ => return,
            },
            _ => (),
        }
    });
}
