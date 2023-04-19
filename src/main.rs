#[macro_use]
extern crate glium;
extern crate image;
extern crate exif;

mod rotation;
use std::{path::Path, fs::{self, FileType}, io, os::windows::prelude::FileExt, time::Instant};

mod image_loading;

use exif::Tag;
use rotation::Rotation;

use glium::glutin::event::{ElementState, ModifiersState, VirtualKeyCode};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct State {
    rotation: Rotation,
    directory: String,
    image_uri: String,
    image_changed: bool,
    modifiers: Option<ModifiersState>,
    mouse_position: Option<(u32, u32)>,
    drag_origin: Option<(u32, u32)>,
    running: bool,
}

impl State {
    fn default() -> Self {
        Self {
            rotation: Rotation::UP,
            directory: String::from("C:\\Users\\Tom\\Pictures\\"),
            image_uri: String::from("C:\\Users\\Tom\\Pictures\\20230330_223017.jpg"),
            image_changed: false,
            modifiers: None,
            mouse_position: None,
            drag_origin: None,
            running: true,
        }
    }

    fn get_dir_cont(&self) -> Result<Vec<fs::DirEntry>, io::Error> {
        let files = fs::read_dir(Path::new(&self.directory))?;
        files.collect::<Result<Vec<fs::DirEntry>, io::Error>>()
    }

    fn next_img(&mut self) {
        if self.image_changed || !self.running {return;}
        println!("Next");
        
        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter();
                // println!("{:?}", i);
                i.find(|f| f.path() == Path::new(&self.image_uri));
                match i.find(|f| {
                    match f.path().as_path().extension() {
                        Some(ext) => match ext.to_str() {
                            Some(ext) => ext == "jpg",
                            None => false,
                        },
                        None => false,
                    }
                }) {
                    Some(new_image) => {
                        self.image_uri = new_image.path().as_path().to_str().unwrap().to_string();
                        println!("Opening: {:?}", self.image_uri);

                        let file = fs::File::open(Path::new(&self.image_uri)).unwrap();
                        let mut buf_reader = io::BufReader::new(&file);
                        let exif_reader = exif::Reader::new();
                        let exif = exif_reader.read_from_container(&mut buf_reader).unwrap();

                        match exif.fields().into_iter().find(|f| f.tag == Tag::Orientation) {
                            Some(orient) => {
                                println!("{:?}", orient.value);
                                match orient.value.get_uint(0) {
                                    Some(6u32) => self.rotation = Rotation::RIGHT,
                                    _ => self.rotation = Rotation::UP,
                                }
                            },
                            None => {},
                        }

                        self.image_changed = true;
                    },
                    None => return,
                }
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    fn prev_img(&mut self) {
        if self.image_changed || !self.running {return;}
        println!("Prev");
        
        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter().rev();
                // println!("{:?}", i);
                i.find(|f| f.path() == Path::new(&self.image_uri));
                match i.next() {
                    Some(new_image) => {
                        self.image_uri = new_image.path().as_path().to_str().unwrap().to_string();
                        println!("Opening: {:?}", self.image_uri);

                        let file = fs::File::open(Path::new(&self.image_uri)).unwrap();
                        let mut buf_reader = io::BufReader::new(&file);
                        let exif_reader = exif::Reader::new();
                        let exif = exif_reader.read_from_container(&mut buf_reader).unwrap();

                        match exif.fields().into_iter().find(|f| f.tag == Tag::Orientation) {
                            Some(orient) => {
                                println!("{:?}", orient.value);
                                match orient.value.get_uint(0) {
                                    Some(6u32) => self.rotation = Rotation::RIGHT,
                                    _ => self.rotation = Rotation::UP,
                                }
                            },
                            None => {},
                        }

                        self.image_changed = true;
                    },
                    None => return,
                }
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
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

    let mut state = State::default();

    let start = Instant::now();
    let image = image_loading::load_image(Path::new(&state.image_uri)).unwrap();
    println!("image loaded: {:?}", start.elapsed());
    // start = Instant::now();
    let mut image_size = (image.width, image.height);
    let mut texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
    println!("texture loaded: {:?}", start.elapsed());

    event_loop.run(move |ev, _, control_flow| {
        if state.image_changed && state.running {
            let start = Instant::now();
            let image = image_loading::load_image(Path::new(&state.image_uri)).unwrap();
            println!("image loaded: {:?}", start.elapsed());
            image_size = (image.width, image.height);
            texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
            println!("texture loaded: {:?}", start.elapsed());

            state.image_changed = false;
        }

        let uniforms = uniform! {
            p_rot: state.rotation.to_mat(display.get_framebuffer_dimensions(), image_size),
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
                    state.running = false;
                    return;
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
