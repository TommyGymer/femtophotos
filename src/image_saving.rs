use std::{borrow::Cow, path::Path, fs::{self, File}, io::{BufWriter, Write}};

use glium::texture::Texture2dDataSink;
use qoi::encode_to_vec;
use turbojpeg::compress_image;

pub struct RGBAImageData {
    pub data: Vec<(u8, u8, u8, u8)>,
    pub width: u32,
    pub height: u32,
}

impl Texture2dDataSink<(u8, u8, u8, u8)> for RGBAImageData {
    fn from_raw(data: Cow<'_, [(u8, u8, u8, u8)]>, width: u32, height: u32) -> Self {
        RGBAImageData {
            data: data.into_owned(),
            width,
            height,
        }
    }
}

pub fn save_image(data: Vec<u8>, width: u32, height: u32, path: &Path) {
    match path.extension() {
        Some(ext) => match ext.to_ascii_lowercase().to_str() {
            Some("jpg") => {
                let image = image::RgbaImage::from_vec(width, height, data).unwrap();
                let jpg = compress_image(&image, 100, turbojpeg::Subsamp::None).unwrap();
                fs::write(path, &jpg).unwrap();
                println!("image saved at {:?}", path);
            },
            Some("png") => {
                let ref mut buf = BufWriter::new(File::create(path).unwrap());
                
                let mut encoder = png::Encoder::new(buf, width, height);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));

                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(&data).unwrap();
                println!("image saved at {:?}", path);
            },
            Some("qoi") => {
                let encoded = encode_to_vec(data, width, height).unwrap();
                File::create(path).unwrap().write_all(&encoded).unwrap();
                println!("image saved at {:?}", path);
            }
            _ => return,
        },
        None => todo!(),
    };
}