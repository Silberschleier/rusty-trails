use std::path::Path;
use std::unimplemented;
use std::fs::File;
use std::io::BufReader;
use std::cmp::max;
use std::sync::{Arc, Mutex};


pub struct Image {
    pub raw_image_data: Vec<u16>,
    exif: Arc<Mutex<exif::Exif>>,
    pub height: usize,
    pub width: usize,
    intensity: f32
}


impl Image {
    pub fn load_from_raw(path: &Path, intensity: f32) -> Result<Image, &str> {
        let exif = load_exif(path).unwrap();
        let image = rawloader::decode_file(path).unwrap();

        if let rawloader::RawImageData::Integer(data) = image.data {
            assert_eq!(data.len(), image.width * image.height, "Mismatch between raw data-size and image resolution.");
            Ok(Image {
                raw_image_data: data,
                exif: Arc::new(Mutex::new(exif)),
                height: image.height,
                width: image.width,
                intensity
            })
        } else {
            unimplemented!("Can't parse RAWs with non-integer data, yet.");
        }
    }

    pub fn merge(&self, other: Image) -> Image {
        let res = self.raw_image_data.iter()
            .zip(other.raw_image_data)
            .map(|(x, y)| max(*x, y))
            .collect();

        Image {
            raw_image_data: res,
            exif: self.exif.clone(),
            height: self.height,
            width: self.width,
            intensity: 1.0
        }
    }
}


fn load_exif(path: &Path) -> Result<exif::Exif, exif::Error> {
    let file = File::open(path)?;
    let exif = exif::Reader::new().read_from_container(
        &mut BufReader::new(&file))?;

    Ok(exif)
}
