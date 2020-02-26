use std::{fs, io};
use std::cmp::max;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use rayon::prelude::*;
use std::ops::Deref;
use std::{thread, time};
use std::borrow::BorrowMut;
use std::time::Duration;


fn main() -> io::Result<()> {
    let mut entries = fs::read_dir("./Lapse_002")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    println!("Found {} files in target folder. Reading...", entries.len());
    let result = Arc::new(Mutex::new(vec![]));
    let mut done = Arc::new(Box::new(true));

    let mut thread_handles = vec![];
    for _ in 0..12 {
        let mut q = Arc::clone(&result);
        let d = Arc::clone(&done);
        thread_handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            queue_worker(q, d);
        }));
    }

    entries.sort();
    entries.par_iter()
        .filter(|e| !e.as_path().is_dir())
        .for_each(|e| process_image(e, Arc::clone(&result)));

    //done.deref().borrow_mut().deref() = true;
    for t in thread_handles {
        t.join();
    }

    let data = result.lock().unwrap();
    println!("Result images has size {}. Writing to out.ppm...", data.len());

    write_ppm(data.first().unwrap());
    Ok(())
}

fn queue_worker(queue: Arc<Mutex<Vec<Vec<u16>>>>, done: Arc<Box<bool>>) {
    loop {
        let mut q = queue.lock().unwrap();
        if q.len() <= 1 {
            if **done { return }
            else {
                // Queue is empty but work is not done yet => Wait.
                drop(q);
                thread::sleep(time::Duration::from_millis(20));
                continue;
            }
        }

        let v1 = q.pop().unwrap();
        let v2 = q.pop().unwrap();
        drop(q);

        let res = v1.par_iter().zip(v2).map(|(x, y)| *max(x, &y)).collect();
        queue.lock().unwrap().push(res);
    }
}

fn process_image(entry: &PathBuf, queue: Arc<Mutex<Vec<Vec<u16>>>>) {
    let image = rawloader::decode_file(entry.as_path()).unwrap();
    if let rawloader::RawImageData::Integer(data) = image.data {
        queue.lock().unwrap().push(data);
        println!("{}: {} {}, width: {}\t height {}", entry.display(), image.clean_make, image.clean_model, image.width, image.height);
    } else {
        eprintln!("Image {} is in non-integer format.", entry.display());
    }
}

fn write_ppm(data: &Vec<u16>) {
    //let width = 5568; let height = 3708;
    let width = 4312; let height = 2876;
    // Write out the image as a grayscale PPM
    let mut f = BufWriter::new(File::create("out.ppm").unwrap());
    let preamble = format!("P6 {} {} {}\n", width, height, 65535).into_bytes();
    f.write_all(&preamble).unwrap();

    for pix in data {
        // Do an extremely crude "demosaic" by setting R=G=B
        let pixhigh = (pix>>8) as u8;
        let pixlow  = (pix&0x0f) as u8;
        f.write_all(&[pixhigh, pixlow, pixhigh, pixlow, pixhigh, pixlow]).unwrap()
    }
}
