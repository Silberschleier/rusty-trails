use std::{fs, io};
use std::cmp::max;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use rayon::prelude::*;
use std::ops::Deref;


fn main() -> io::Result<()> {
    let mut entries = fs::read_dir("./Lapse_001")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    println!("Found {} files in target folder. Reading...", entries.len());
    let result = Arc::new(Mutex::new(vec![]));

    entries.sort();
    entries.par_iter()
        .filter(|e| !e.as_path().is_dir())
        .map(|e| process_image(e))
        .for_each(|x| add_vectors(Arc::clone(&result), x));

    let data = result.lock().unwrap();
    println!("Result images has size {}. Writing to out.ppm...", data.len());

    write_ppm(data.deref());
    Ok(())
}

fn add_vectors(v1: Arc<Mutex<Vec<u16>>>, v2: Vec<u16>) {
    let mut v = v1.lock().unwrap();
    if v.is_empty() {
        v.extend(v2.iter().clone())
    } else {
        //v.iter().zip(v2).zip(0..v.len()).for_each(|((x, y), i)| v[i] = x + y).collect();
        /*for (mut x, y) in v.par_iter().zip(v2) {
            x = max(x, &y);
        }*/
        v.par_iter().zip(v2).map(|(mut x, y)| x = max(x,&y)).collect()
    }
}

fn process_image(entry: &PathBuf) -> Vec<u16> {
    let image = rawloader::decode_file(entry.as_path()).unwrap();
    if let rawloader::RawImageData::Integer(data) = image.data {
        println!("{}: {} {}, width: {}\t height {}", entry.display(), image.clean_make, image.clean_model, image.width, image.height);
        data
    } else {
        vec![0]
    }
}

fn write_ppm(data: &Vec<u16>) {
    // Write out the image as a grayscale PPM
    let mut f = BufWriter::new(File::create("out.ppm").unwrap());
    let preamble = format!("P6 {} {} {}\n", 5568, 3708, 65535).into_bytes();
    f.write_all(&preamble).unwrap();

    for pix in data {
        // Do an extremely crude "demosaic" by setting R=G=B
        let pixhigh = (pix>>8) as u8;
        let pixlow  = (pix&0x0f) as u8;
        f.write_all(&[pixhigh, pixlow, pixhigh, pixlow, pixhigh, pixlow]).unwrap()
    }
}
