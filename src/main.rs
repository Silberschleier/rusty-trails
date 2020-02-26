use std::{fs, io, thread, time};
use std::cmp::max;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use rayon::prelude::*;


fn main() -> io::Result<()> {
    let num_threads = num_cpus::get();
    println!("System has {} logical cores. Using the same number of worker threads", num_threads);

    let mut entries = fs::read_dir("./Lapse_001")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    println!("Found {} files in target folder. Reading...", entries.len());

    let result = Arc::new(Mutex::new(vec![]));
    let done = Arc::new(AtomicBool::new(false));
    let mut thread_handles = vec![];

    for _ in 0..num_threads {
        let q = Arc::clone(&result);
        let d = Arc::clone(&done);
        thread_handles.push(thread::spawn(move || {
            queue_worker(q, d);
        }));
    }

    entries.sort();
    entries.par_iter()
        .filter(|e| !e.as_path().is_dir())
        .for_each(|e| process_image(e, Arc::clone(&result)));

    done.store(true, Ordering::Relaxed);
    for t in thread_handles {
        t.join().unwrap_or(());
    }

    let data = result.lock().unwrap();
    let raw_image = data.first().unwrap();
    println!("Result images has size {}. Writing to out.ppm...", raw_image.len());

    write_ppm(raw_image);
    Ok(())
}

fn queue_worker(queue: Arc<Mutex<Vec<Vec<u16>>>>, done: Arc<AtomicBool>) {
    loop {
        let mut q = queue.lock().unwrap();
        if q.len() <= 1 {
            if done.load(Ordering::Relaxed) { return }
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

        let res = v1.iter().zip(v2).map(|(x, y)| *max(x, &y)).collect();
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
    let width = 5568; let height = 3708;
    //let width = 4312; let height = 2876;
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
