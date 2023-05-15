use std::{
    env, fs,
    io::{self, Cursor, ErrorKind},
    path::{Path, PathBuf},
    time::Instant,
};

use glium::texture::RawImage2d;
use image::{
    error::{DecodingError, ImageFormatHint},
    Rgb, Rgba,
};
use log::{info, trace, warn, debug};
use qoi::decode_to_vec;
use turbojpeg::decompress_image;

#[derive(Debug)]
enum Image {
    Rgb(image::RgbImage),
    Rgba(image::RgbaImage),
}

impl Image {
    fn get_size(&self) -> ImageDimensions {
        match self {
            Image::Rgb(img) => img.dimensions(),
            Image::Rgba(img) => img.dimensions(),
        }
    }
}

type BoxedError = Box<dyn std::error::Error>;
type ImageDimensions = (u32, u32);
type RawImage = Vec<u8>;

pub fn load_image(path: &Path) -> Result<RawImage2d<'static, u8>, BoxedError> {
    let start = Instant::now();

    let image: Image = match fast_load(path) {
        Ok(img) => img,
        Err(err) => {
            warn!("fast load failed: {:?}", err);
            match slow_load_rgb(path) {
                Ok(img) => img,
                Err(err) => {
                    warn!("rgb slow load failed: {:?}", err);
                    match slow_load_rgba(path) {
                        Ok(img) => img,
                        Err(err) => {
                            warn!("rgba slow load failed: {:?}", err);

                            let current_exe = env::current_exe()?;
                            let parent = match current_exe.parent() {
                                Some(parent) => parent,
                                None => {
                                    return Err(Box::new(io::Error::new(
                                        ErrorKind::NotFound,
                                        "executable had no parent",
                                    )))
                                }
                            };
                            let path_str = format!("{}/img/no_image.png", parent.to_str().unwrap());
                            let path = Path::new(&path_str);
                            return load_image(path);
                        }
                    }
                }
            }
        }
    };

    info!("image decompressed: {:?}", start.elapsed());
    debug!("{:?}", image);
    info!("{:?}", image.get_size());

    texture_from_image(image)
}

fn fast_load(path: &Path) -> Result<Image, BoxedError> {
    match path.extension() {
        Some(ext) => match ext.to_ascii_lowercase().to_str() {
            Some("jpg") | Some("jfif") => Ok(Image::Rgba(decompress_image(&(fs::read(path)?))?)),
            Some("png") => {
                let file = &(fs::read(path).unwrap());
                let cursor = Cursor::new(file);
                let decoder = spng::Decoder::new(cursor);
                let (info, mut reader) = decoder.read_info()?;

                let mut out: RawImage = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut out).unwrap();

                match info.color_type {
                    spng::ColorType::Truecolor => Ok(Image::Rgb(rgb_image_from_raw(
                        info.width,
                        info.height,
                        out,
                        path.to_path_buf(),
                    )?)),
                    spng::ColorType::TruecolorAlpha => Ok(Image::Rgba(rgba_image_from_raw(
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
                    qoi::Channels::Rgb => Ok(Image::Rgb(rgb_image_from_raw(
                        header.width,
                        header.height,
                        decoded,
                        path.to_path_buf(),
                    )?)),
                    qoi::Channels::Rgba => Ok(Image::Rgba(rgba_image_from_raw(
                        header.width,
                        header.height,
                        decoded,
                        path.to_path_buf(),
                    )?)),
                }
            }
            _ => Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            ))),
        },
        _ => {
            warn!("no extension");
            Err(Box::new(io::Error::new(
                ErrorKind::Other,
                "unsupported extension",
            )))
        }
    }
}

fn slow_load_rgb(path: &Path) -> Result<Image, BoxedError> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    trace!("detected format: {:?}", reader.format());
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
    Ok(Image::Rgb(data.to_owned()))
}

fn slow_load_rgba(path: &Path) -> Result<Image, BoxedError> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    trace!("detected format: {:?}", reader.format());
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
    Ok(Image::Rgba(data.to_owned()))
}

fn rgb_image_from_raw(
    width: u32,
    height: u32,
    data: RawImage,
    ext: PathBuf,
) -> Result<image::RgbImage, image::ImageError> {
    match image::ImageBuffer::<Rgb<u8>, RawImage>::from_raw(width, height, data) {
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
    data: RawImage,
    ext: PathBuf,
) -> Result<image::RgbaImage, image::ImageError> {
    match image::ImageBuffer::<Rgba<u8>, RawImage>::from_raw(width, height, data) {
        Some(image) => Ok(image),
        None => Err(image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::PathExtension(ext),
            "Raw image bytes did not fit the image container",
        ))),
    }
}

fn texture_from_image(img: Image) -> Result<RawImage2d<'static, u8>, BoxedError> {
    match img {
        Image::Rgb(img) => {
            let image_dimensions = img.dimensions();
            Ok(glium::texture::RawImage2d::from_raw_rgb(
                img.into_raw(),
                image_dimensions,
            ))
        }
        Image::Rgba(img) => {
            let image_dimensions = img.dimensions();
            Ok(glium::texture::RawImage2d::from_raw_rgba(
                img.into_raw(),
                image_dimensions,
            ))
        }
    }
}

pub fn icon() -> Result<(RawImage, ImageDimensions), BoxedError> {
    let current_exe = env::current_exe()?;
    let parent = match current_exe.parent() {
        Some(parent) => parent,
        None => {
            return Err(Box::new(io::Error::new(
                ErrorKind::NotFound,
                "executable had no parent",
            )))
        }
    };
    let path_str = format!("{}/img/icon.ico", parent.to_str().unwrap());
    let path = Path::new(&path_str);
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    trace!("detected format: {:?}", reader.format());
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
        "kodim10",
        "kodim23",
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
            String::from("kodim10"),
            String::from("kodim23"),
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
    fn test_load_image() {
        assert!(load_image(Path::new("./test_images/0.jpg")).is_ok());
    }

    #[test]
    fn test_png_load() {
        for image in IMAGES {
            let formatted = format!("./test_images/{}.png", String::from(image));
            let path = Path::new(&formatted);
            let result = load_image(path);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_jpg_load() {
        for image in IMAGES {
            let formatted = format!("./test_images/{}.jpg", String::from(image));
            let path = Path::new(&formatted);
            let result = load_image(path);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_qoi_load() {
        for image in IMAGES {
            let formatted = format!("./test_images/{}.qoi", String::from(image));
            let path = Path::new(&formatted);
            let result = load_image(path);
            assert!(result.is_ok());
        }
    }
}
