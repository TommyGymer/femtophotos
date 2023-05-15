#![windows_subsystem = "windows"]

#[macro_use]
extern crate glium;
extern crate exif;
extern crate image;

mod image_loading;
mod image_saving;
mod rotation;
mod state;
use image_saving::save_image;
use rfd::FileDialog;
use state::State;

use core::fmt;
use glium::{
    glutin::{
        event::{ElementState, ModifiersState, VirtualKeyCode},
        window::Icon,
    },
    texture::SrgbTexture2d,
    Blend, Display, DrawParameters,
};
use log::{debug, info, trace, warn, LevelFilter};
use std::{env, ffi::OsString, path::Path, thread};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

fn load_texture(
    display: &Display,
    state: &State,
) -> Result<(SrgbTexture2d, (u32, u32)), Box<dyn std::error::Error>> {
    // let start = Instant::now();
    let image = image_loading::load_image(Path::new(&state.image_uri))?;
    let image_size = (image.width, image.height);
    // println!("image loaded: {:?}", start.elapsed());
    let texture = glium::texture::SrgbTexture2d::new(display, image)?;
    // println!("texture loaded: {:?}", start.elapsed());
    Ok((texture, image_size))
}

#[derive(Debug, Clone)]
struct LogFileError {
    err_str: String,
}

impl fmt::Display for LogFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LogFileError: {}", self.err_str)
    }
}

fn attempt_log_file() -> Result<(), LogFileError> {
    let current_exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return Err(LogFileError {
                err_str: err.to_string(),
            })
        }
    };
    let parent = match current_exe.parent() {
        Some(parent) => parent,
        None => {
            return Err(LogFileError {
                err_str: String::from("executable had no parent"),
            })
        }
    };
    let dir_str = match parent.to_str() {
        Some(dir_str) => dir_str,
        None => {
            return Err(LogFileError {
                err_str: String::from("executable parent directory path was not a string"),
            })
        }
    };
    match simple_logging::log_to_file(format!("{}/latest.log", dir_str), LevelFilter::Trace) {
        Ok(()) => Ok(()),
        Err(err) => Err(LogFileError {
            err_str: err.to_string(),
        }),
    }
}

fn main() {
    match attempt_log_file() {
        Ok(()) => info!("Logging to latest.log"),
        Err(err) => warn!("Logging to stdout: {:?}", err),
    }

    info!("dir: {:?}", env::current_dir());
    info!("exe: {:?}", env::current_exe());

    let args: Vec<OsString> = env::args_os().collect();
    info!("{:?}", &args);

    use glium::glutin;
    use glium::Surface;

    let event_loop = glutin::event_loop::EventLoop::new();
    let icon = match image_loading::icon() {
        Ok((data, (width, height))) => Some(Icon::from_rgba(data, width, height).unwrap()),
        Err(_) => None,
    };
    let wb = glutin::window::WindowBuilder::new()
        .with_title("FemtoPhotos: ")
        .with_transparent(true)
        .with_window_icon(icon);
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
    if args.len() > 1 {
        state.image_uri = args.get(1).unwrap().to_str().unwrap().to_string();
        state.directory = Path::new(&state.image_uri)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }
    state.load_img();

    let mut texture: SrgbTexture2d;
    let mut image_size: (u32, u32);

    (texture, image_size) = match load_texture(&display, &state) {
        Ok(res) => res,
        Err(err) => {
            info!("{:?}", err);
            panic!("{:?}", err);
        }
    };

    info!("First texture loaded");

    display.gl_window().window().set_title(&format!(
        "FemtoPhotos: {}",
        Path::new(&state.image_uri)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    ));

    state.image_changed = false;

    info!("Render loop started");

    event_loop.run(move |ev, _, control_flow| {
        // let next_frame_time =
        //     std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        // *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        *control_flow = glutin::event_loop::ControlFlow::Wait;

        if !state.running {
            return;
        }

        state.needs_redraw = true;

        // let str_ev = format!("{:?}", ev);

        // println!("{:?}", ev);
        match ev {
            glutin::event::Event::MainEventsCleared | glutin::event::Event::RedrawEventsCleared => {
                state.needs_redraw = false;
            }
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    debug!("Close requested");
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    state.running = false;
                    state.needs_redraw = false;
                }
                glutin::event::WindowEvent::ModifiersChanged(mod_state) => {
                    if mod_state.is_empty() {
                        state.modifiers = None;
                    } else {
                        state.modifiers = Some(mod_state);
                    }
                    state.needs_redraw = false;
                }
                glutin::event::WindowEvent::CursorMoved { position, .. } => {
                    state.mouse_position = Some((position.x as u32, position.y as u32));
                    state.needs_redraw = false;
                }
                glutin::event::WindowEvent::CursorLeft { .. } => {
                    state.mouse_position = None;
                    state.needs_redraw = false;
                }
                glutin::event::WindowEvent::Touch(touch) => match touch.phase {
                    glutin::event::TouchPhase::Started => {
                        state.needs_redraw = false;
                        state.drag_origin = Some((touch.location.x as u32, touch.location.y as u32))
                    }
                    glutin::event::TouchPhase::Ended => {
                        if let (Some(start), Some(end)) = (state.drag_origin, state.mouse_position)
                        {
                            trace!(
                                "mouse: start@{} end@{} prev:{} next:{}",
                                start.0,
                                end.0,
                                start.0 > end.0 + 10,
                                end.0 > start.0 + 10
                            );
                            if start.0 > end.0 + 10 {
                                state.prev_img();
                            } else if end.0 > start.0 + 10 {
                                state.next_img();
                            }
                        }
                    }
                    glutin::event::TouchPhase::Moved => {
                        state.needs_redraw = false;
                        state.mouse_position =
                            Some((touch.location.x as u32, touch.location.y as u32))
                    }
                    _ => {
                        state.needs_redraw = false;
                        state.drag_origin = None;
                        state.mouse_position = None;
                    }
                },
                glutin::event::WindowEvent::DroppedFile(_)
                | glutin::event::WindowEvent::HoveredFile(_)
                | glutin::event::WindowEvent::HoveredFileCancelled
                | glutin::event::WindowEvent::ReceivedCharacter(_)
                | glutin::event::WindowEvent::KeyboardInput { .. }
                | glutin::event::WindowEvent::Ime(_)
                | glutin::event::WindowEvent::CursorEntered { .. }
                | glutin::event::WindowEvent::MouseWheel { .. }
                | glutin::event::WindowEvent::MouseInput { .. }
                | glutin::event::WindowEvent::TouchpadPressure { .. }
                | glutin::event::WindowEvent::AxisMotion { .. }
                | glutin::event::WindowEvent::Occluded(_)
                | glutin::event::WindowEvent::Moved { .. } => {
                    state.needs_redraw = false;
                }
                _ => (),
            },
            glutin::event::Event::DeviceEvent {
                device_id: _,
                event,
            } => match event {
                glutin::event::DeviceEvent::MouseWheel { .. } => {
                    state.needs_redraw = false;
                    // println!("{:?}", delta);
                }
                glutin::event::DeviceEvent::Button {
                    button,
                    state: button_state,
                } => match (button, button_state) {
                    (1, ElementState::Pressed) => {
                        state.needs_redraw = false;
                        state.drag_origin = state.mouse_position;
                    }
                    (1, ElementState::Released) => {
                        if let (Some(start), Some(end)) = (state.drag_origin, state.mouse_position)
                        {
                            trace!(
                                "touch: start@{} end@{} prev:{} next:{}",
                                start.0,
                                end.0,
                                start.0 > end.0 + 10,
                                end.0 > start.0 + 10
                            );
                            if start.0 > end.0 + 10 {
                                state.prev_img();
                            } else if end.0 > start.0 + 10 {
                                state.next_img();
                            }
                        }
                    }
                    _ => {
                        state.needs_redraw = false;
                    } //_ => println!("{:?}", event),
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
                        (Some(VirtualKeyCode::S), ElementState::Released, None) => {
                            state.needs_redraw = false;
                            let file = FileDialog::new()
                                .set_directory(Path::new(&state.directory))
                                .set_file_name(
                                    Path::new(&state.image_uri)
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap(),
                                )
                                .add_filter("JPG", &["jpg", "JPG", "jpeg", "JPEG"])
                                .add_filter("PNG", &["png", "PNG"])
                                .add_filter("QOI", &["qoi", "QOI"])
                                .save_file();

                            info!("Saving to {:?}", file);

                            let buf: image_saving::RGBAImageData =
                                texture.read_to_pixel_buffer().read_as_texture_2d().unwrap();
                            let size = (texture.width(), texture.height());

                            thread::spawn(move || {
                                let data: Vec<u8> = flatten(buf.data);

                                save_image(data, size.0, size.1, file.unwrap().as_path());
                            });
                        }
                        _ => {
                            state.needs_redraw = false;
                        }
                    }
                }
                _ => {
                    state.needs_redraw = false;
                }
            },
            glutin::event::Event::NewEvents(cause) => {
                // println!("{:?}", cause);
                if cause == glutin::event::StartCause::Poll {
                    state.needs_redraw = false;
                }
            }
            glutin::event::Event::Suspended
            | glutin::event::Event::Resumed
            | glutin::event::Event::LoopDestroyed => {
                state.needs_redraw = false;
            }
            _ => {
                // println!("{:?}", ev);
            }
        }

        if state.needs_redraw && state.running {
            // println!("{}", str_ev);

            if state.image_changed {
                (texture, image_size) = match load_texture(&display, &state) {
                    Ok(res) => res,
                    Err(err) => panic!("{:?}", err),
                };

                display.gl_window().window().set_title(&format!(
                    "FemtoPhotos: {}",
                    Path::new(&state.image_uri)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                ));

                state.image_changed = false;
            }

            let uniforms = uniform! {
                p_rot: state.rotation.to_mat(display.get_framebuffer_dimensions(), image_size),
                tex: &texture,
            };

            let mut target = display.draw();
            target.clear_color(0.2, 0.2, 0.2, 1.0);

            target
                .draw(
                    &vertex_buffer,
                    indices,
                    &program,
                    &uniforms,
                    &DrawParameters {
                        blend: Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();

            target.finish().unwrap();
            state.needs_redraw = false;
        }
    });
}

#[no_mangle]
#[inline(never)]
fn flatten(data: Vec<(u8, u8, u8, u8)>) -> Vec<u8> {
    let size = data.capacity();
    let mut result = data;
    unsafe {
        result.set_len(size * 4);
        #[allow(clippy::unsound_collection_transmute)]
        std::mem::transmute(result)
    }
}
