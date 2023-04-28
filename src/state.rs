use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

use crate::rotation::Rotation;
use exif::Tag;
use glium::glutin::event::ModifiersState;
use log::{info, warn};

pub struct State {
    pub rotation: Rotation,
    pub directory: String,
    pub image_uri: String,
    pub image_changed: bool,
    pub modifiers: Option<ModifiersState>,
    pub mouse_position: Option<(u32, u32)>,
    pub drag_origin: Option<(u32, u32)>,
    pub running: bool,
}

impl State {
    pub fn default() -> Self {
        Self {
            rotation: Rotation::Up,
            directory: String::from("./img/"),
            image_uri: String::from("./img/no_image.png"),
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

    pub fn load_img(&mut self) {
        let file = match fs::File::open(Path::new(&self.image_uri)) {
            Ok(f) => f,
            Err(err) => {
                warn!("{:?}", err);
                return;
            }
        };
        let mut buf_reader = io::BufReader::new(&file);
        let exif_reader = exif::Reader::new();
        match exif_reader.read_from_container(&mut buf_reader) {
            Ok(exif) => {
                if let Some(orient) = exif.fields().find(|f| f.tag == Tag::Orientation) {
                    match orient.value.get_uint(0) {
                        Some(1u32) => self.rotation = Rotation::Up,
                        Some(6u32) => self.rotation = Rotation::Right,
                        Some(3u32) => self.rotation = Rotation::Down,
                        Some(8u32) => self.rotation = Rotation::Left,
                        _ => self.rotation = Rotation::Up,
                    }
                }
            }
            Err(err) => {
                self.rotation = Rotation::Up;
                warn!("exif: {:?}", err);
            }
        };

        self.image_changed = true;
    }

    fn open_img<I: Iterator<Item = DirEntry>>(&mut self, mut i: I) {
        if let Some(new_image) = i.find(|f| match f.path().as_path().extension() {
            Some(ext) => match ext.to_ascii_lowercase().to_str() {
                Some(extension) => ["png", "jpg", "qoi", "ico", "jfif"].contains(&extension),
                None => false,
            },
            None => false,
        }) {
            self.image_uri = new_image.path().as_path().to_str().unwrap().to_string();
            info!("Opening: {:?}", self.image_uri);
            self.load_img();
        };
    }

    pub fn next_img(&mut self) {
        if self.image_changed || !self.running {}

        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter();

                i.find(|f| f.path() == Path::new(&self.image_uri));
                self.open_img(i);
            }
            Err(err) => {
                warn!("{:?}", err);
            }
        }
    }

    pub fn prev_img(&mut self) {
        if self.image_changed || !self.running {}

        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter().rev();

                i.find(|f| f.path() == Path::new(&self.image_uri));
                self.open_img(i);
            }
            Err(err) => {
                warn!("{:?}", err);
            }
        }
    }
}
