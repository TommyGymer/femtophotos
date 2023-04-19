use std::{fs, path::Path, io, time::Instant};

use glium::{texture::RawImage2d};
use turbojpeg::{decompress_image};

pub fn load_image(path: &Path) -> Result<RawImage2d<'static, u8>, io::Error> {
    let start = Instant::now();
    let image: image::RgbaImage = match decompress_image(&(fs::read(path)?)) {
        Ok(img) => img,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
    };

    println!("image decompressed: {:?}", start.elapsed());

    let image_dimensions = image.dimensions();
    Ok(glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions))
}