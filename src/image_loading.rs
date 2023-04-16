use std::{fs, path::Path, io};

use glium::{texture::RawImage2d};
use turbojpeg::{decompress_image};

pub fn load_image(path: &Path) -> Result<RawImage2d<'static, u8>, io::Error> {
    let image: image::RgbaImage = match decompress_image(&(fs::read(path)?)) {
        Ok(img) => img,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };

    let image_dimensions = image.dimensions();
    Ok(glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions))
}