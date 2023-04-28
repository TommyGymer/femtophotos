use std::{
    fs,
    io::{self, Cursor, ErrorKind},
    path::{Path, PathBuf},
    time::Instant,
};

use glium::texture::RawImage2d;
use image::{
    error::{DecodingError, ImageFormatHint},
    Rgb, Rgba,
};
use qoi::decode_to_vec;
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
            println!("fast load failed: {:?}", err);
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
            Some("jpg") | Some("jfif") => Ok(Image::RGBA(decompress_image(&(fs::read(path)?))?)),
            Some("png") => {
                let file = &(fs::read(path).unwrap());
                let cursor = Cursor::new(file);
                let decoder = spng::Decoder::new(cursor);
                let (info, mut reader) = decoder.read_info()?;

                let mut out: Vec<u8> = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut out).unwrap();

                match info.color_type {
                    spng::ColorType::Truecolor => Ok(Image::RGB(rgb_image_from_raw(
                        info.width,
                        info.height,
                        out,
                        path.to_path_buf(),
                    )?)),
                    spng::ColorType::TruecolorAlpha => Ok(Image::RGBA(rgba_image_from_raw(
                        info.width,
                        info.height,
                        out,
                        path.to_path_buf(),
                    )?)),
                    _ => panic!(
                        "not implemented grayscale png; this should probably return an error"
                    ),
                }
            }
            Some("qoi") => {
                let file = &(fs::read(path).unwrap());
                let (header, decoded) = decode_to_vec(file)?;

                match header.channels {
                    qoi::Channels::Rgb => Ok(Image::RGB(rgb_image_from_raw(
                        header.width,
                        header.height,
                        decoded,
                        path.to_path_buf(),
                    )?)),
                    qoi::Channels::Rgba => Ok(Image::RGBA(rgba_image_from_raw(
                        header.width,
                        header.height,
                        decoded,
                        path.to_path_buf(),
                    )?)),
                }
            }
            _ => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::Other,
                    "unsupported extension",
                )))
            }
        },
        _ => {
            println!("no extension");
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            )));
        }
    }
}

fn slow_load_rgb(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    println!("detected format: {:?}", reader.format());
    let decoded = reader.decode()?;
    let data = match decoded.as_rgb8() {
        Some(data) => data,
        None => {
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            )))
        }
    };
    Ok(Image::RGB(data.to_owned()))
}

fn slow_load_rgba(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    println!("detected format: {:?}", reader.format());
    let decoded = reader.decode()?;
    let data = match decoded.as_rgba8() {
        Some(data) => data,
        None => {
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            )))
        }
    };
    Ok(Image::RGBA(data.to_owned()))
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
            Ok(glium::texture::RawImage2d::from_raw_rgb(
                img.into_raw(),
                image_dimensions,
            ))
        }
        Image::RGBA(img) => {
            let image_dimensions = img.dimensions();
            Ok(glium::texture::RawImage2d::from_raw_rgba(
                img.into_raw(),
                image_dimensions,
            ))
        }
    }
}

pub fn icon() -> Result<(Vec<u8>, (u32, u32)), Box<dyn std::error::Error>> {
    let path = Path::new("./img/icon.ico");
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    println!("detected format: {:?}", reader.format());
    let decoded = reader.decode()?;
    let data = match decoded.as_rgba8() {
        Some(data) => data,
        None => {
            return Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            )))
        }
    };

    Ok((data.as_raw().to_vec(), data.dimensions()))
}

#[cfg(test)]
mod image_loading_tests {
    use super::*;

    const IMAGES: [&str; 8] = [
        "0",
        "dice",
        "kodium10",
        "kodium23",
        "qoi_logo",
        "testcard",
        "testcard_rgba",
        "wikipedia_008",
    ];

    #[test]
    fn test_images_const() {
        let images = [
            String::from("0"),
            String::from("dice"),
            String::from("kodium10"),
            String::from("kodium23"),
            String::from("qoi_logo"),
            String::from("testcard"),
            String::from("testcard_rgba"),
            String::from("wikipedia_008"),
        ];
        let mut i = 0;
        for image in IMAGES {
            assert_eq!(images.get(i).unwrap(), &image);
            i += 1;
        }
    }

    #[test]
    fn test_load_image()  {
        assert!(load_image(Path::new("./test_images/0.jpg")).is_ok());
    }

    // #[test]
    // fn bench_png_load() {
    //     for image in IMAGES {
    //         let formatted = format!("./test_images/{:?}.png", String::from(image));
    //         let path = Path::new(&formatted);
    //         let result = load_image(path);
    //         assert!(result.is_ok());
    //     }
    // }

    // #[test]
    // fn test_jpg_load() -> Result<(), Box<dyn std::error::Error>> {
    //     for image in IMAGES {
    //         let _ = load_image(Path::new(&format!("C:\\Users\\Tom\\Documents\\GitHub\\femtophotos\\test_images\\{:?}.jpg", String::from(image))))?;
    //         assert!(true);
    //     }
    //     Ok(())
    // }

    // #[test]
    // fn test_qoi_load() -> Result<(), Box<dyn std::error::Error>> {
    //     for image in IMAGES {
    //         let _ = load_image(Path::new(&format!("C:\\Users\\Tom\\Documents\\GitHub\\femtophotos\\test_images\\{:?}.qoi", String::from(image))))?;
    //         assert!(true);
    //     }
    //     Ok(())
    // }
}
