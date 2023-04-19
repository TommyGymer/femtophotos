use std::{path::{Path, PathBuf}, time::Instant, io::{ErrorKind, self, Cursor}, fs};

use glium::{texture::RawImage2d};
use image::{error::{ImageFormatHint, DecodingError}, Rgb, Rgba};
use turbojpeg::decompress_image;

enum Image {
    RGB(image::RgbImage),
    RGBA(image::RgbaImage),
}

pub fn load_image(path: &Path) -> Result<RawImage2d<'static, u8>, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let image: Image = match fast_load(path) {
        Ok(img) => img,
        Err(err) => {
            println!("fast load failed: {:#?}", err);
            match slow_load_rgb(path) {
                Ok(img) => img,
                Err(err) => {
                    println!("rgb slow load failed: {:?}", err);
                    slow_load_rgba(path)?
                }
            }
        }
    };

    println!("image decompressed: {:?}", start.elapsed());

    texture_from_image(image)
}

fn fast_load(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    match path.extension() {
        Some(ext) => match ext.to_ascii_lowercase().to_str() {
            Some("jpg") => Ok(Image::RGBA(decompress_image(&(fs::read(path)?))?)),
            Some("png") => {
                let file = &(fs::read(path).unwrap());
                let cursor = Cursor::new(file);
                let decoder = spng::Decoder::new(cursor);
                let (info, mut reader) = decoder.read_info()?;

                let mut out: Vec<u8> = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut out).unwrap();

                match info.color_type {
                    spng::ColorType::Truecolor => Ok(Image::RGB(rgb_image_from_raw(info.width, info.height, out, path.to_path_buf())?)),
                    spng::ColorType::TruecolorAlpha => Ok(Image::RGBA(rgba_image_from_raw(info.width, info.height, out, path.to_path_buf())?)),
                    _ => panic!("not implemented grayscale png; this should probably return an error"),
                }
            }
            _ => panic!("not implemented"),
        },
        _ => {
            println!("no extension");
            return Err(Box::new(io::Error::new(ErrorKind::Other, "unsupported extension")));
        }
    }
}

fn slow_load_rgb(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    println!("detected format: {:?}", reader.format());
    Ok(Image::RGB(reader
        .decode()?
        .as_rgb8()
        .unwrap()
        .to_owned()))
}

fn slow_load_rgba(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    println!("detected format: {:?}", reader.format());
    Ok(Image::RGBA(reader
        .decode()?
        .as_rgba8()
        .unwrap()
        .to_owned()))
}

fn rgb_image_from_raw(
    width: u32,
    height: u32,
    data: Vec<u8>,
    ext: PathBuf,
) -> Result<image::RgbImage, image::ImageError> {
    match image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, data) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            "Raw image bytes did not fit the image container",
        ))),
    }
}

fn rgba_image_from_raw(
    width: u32,
    height: u32,
    data: Vec<u8>,
    ext: PathBuf,
) -> Result<image::RgbaImage, image::ImageError> {
    match image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, data) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            "Raw image bytes did not fit the image container",
        ))),
    }
}

fn texture_from_image(img: Image) -> Result<RawImage2d<'static, u8>, Box<dyn std::error::Error>> {
    match img {
        Image::RGB(img) => {
            let image_dimensions = img.dimensions();
            Ok(glium::texture::RawImage2d::from_raw_rgb(img.into_raw(), image_dimensions))
        },
        Image::RGBA(img) => {
            let image_dimensions = img.dimensions();
            Ok(glium::texture::RawImage2d::from_raw_rgba(img.into_raw(), image_dimensions))
        },
    }
}