use std::{path::Path, fs::{self, DirEntry}, io};

use exif::Tag;
use glium::glutin::event::ModifiersState;
use crate::rotation::Rotation;

pub struct State {
    pub rotation: Rotation,
    directory: String,
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

    pub fn load_img(&mut self) {
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
    }

    fn open_img<I: Iterator<Item = DirEntry>>(&mut self, mut i: I) {
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

                self.load_img();
            },
            None => return,
        };
    }

    pub fn next_img(&mut self) {
        if self.image_changed || !self.running {return;}
        println!("Next");
        
        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter();
                // println!("{:?}", i);
                i.find(|f| f.path() == Path::new(&self.image_uri));
                self.open_img(i);
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    pub fn prev_img(&mut self) {
        if self.image_changed || !self.running {return;}
        println!("Prev");
        
        match self.get_dir_cont() {
            Ok(mut files) => {
                files.sort_by(|a, b| a.path().partial_cmp(&b.path()).unwrap());
                let mut i = files.into_iter().rev();
                // println!("{:?}", i);
                i.find(|f| f.path() == Path::new(&self.image_uri));
                self.open_img(i);
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}