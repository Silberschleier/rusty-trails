#![feature(vec_into_raw_parts)]

use std::{io, thread, time};
use std::path::{PathBuf, Path};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::ffi::CString;
use std::os::raw::c_char;
use rayon::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::convert::TryInto;
use glob::glob;
use clap::{crate_version, Clap};

mod image;


extern {
    #[link(name = "dngwrite", kind = "static")]
    fn buildDNG(image_data: *mut ::std::os::raw::c_ushort, width: u32, height: u32, out_file: *const c_char, exif_raw_file: *const c_char);
}

/// Stacks Startrails from RAW images into a RAW-DNG.
#[derive(Clap)]
#[clap(version = crate_version ! ())]
struct Opts {
    /// Input file pattern. Use with quotation marks, e.g.: "directory/*.CR2"
    input: String,
    /// Path of the resulting DNG
    #[clap(default_value = "startrails.dng")]
    output: String,
    /// Stacking mode
    #[clap(short = "m", possible_values = & ["falling", "raising", "normal"])]
    mode: String,
}

#[derive(Copy, Clone)]
enum CometMode {
    Falling,
    Raising,
    Normal,
}


fn main() -> io::Result<()> {
    let opts = Opts::parse();

    let num_threads = num_cpus::get();
    println!("System has {} cores and {} threads. Using {} worker threads.", num_cpus::get_physical(), num_threads, num_threads);

    let mode = match opts.mode.as_str() {
        "falling" => CometMode::Falling,
        "raising" => CometMode::Raising,
        _ => CometMode::Normal
    };

    let mut entries = vec![];
    for entry in glob(&opts.input).expect("Failed to match glob") {
        let e = entry.unwrap();
        if e.is_file() {
            entries.push(e);
        }
    }

    println!("Processing {} files in target folder.", entries.len());

    let result = Arc::new(Mutex::new(vec![]));
    let done = Arc::new(AtomicBool::new(false));
    let mut thread_handles = vec![];

    // Setup CLI progressbar
    let m = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");
    let pb_decode = Arc::new(Mutex::new(m.add(ProgressBar::new(entries.len() as u64))));
    let pb_blend = Arc::new(Mutex::new(m.add(ProgressBar::new(entries.len() as u64))));
    pb_decode.lock().unwrap().set_style(sty.clone());
    pb_decode.lock().unwrap().set_message("Decode files");
    pb_blend.lock().unwrap().set_style(sty);
    pb_blend.lock().unwrap().set_message("Blend images");

    let pb_thread = thread::spawn(move || {
        m.join().unwrap();
    });


    for _ in 0..num_threads {
        let q = Arc::clone(&result);
        let d = Arc::clone(&done);
        let pb = Arc::clone(&pb_blend);
        thread_handles.push(thread::spawn(move || {
            queue_worker(q, d, pb);
        }));
    }

    entries.sort();
    entries.par_iter()
        .zip(0..entries.len())
        .for_each(|(e, i)| process_image(e, Arc::clone(&result), Arc::clone(&pb_decode), i, entries.len(), mode));
    pb_decode.lock().unwrap().finish();

    done.store(true, Ordering::Relaxed);
    for t in thread_handles {
        t.join().unwrap_or(());
    }
    pb_blend.lock().unwrap().finish();
    pb_thread.join().unwrap_or(());

    let mut data = result.lock().unwrap();
    let raw_image = data.pop().unwrap();

    write_dng(raw_image, Path::new(&opts.output), Path::new(entries.first().unwrap()));
    Ok(())
}


fn queue_worker(queue: Arc<Mutex<Vec<image::Image>>>, done: Arc<AtomicBool>, pb: Arc<Mutex<ProgressBar>>) {
    loop {
        let mut q = queue.lock().unwrap();
        if q.len() <= 1 {
            if done.load(Ordering::Relaxed) { return; } else {
                // Queue is empty but work is not done yet => Wait.
                drop(q);
                thread::sleep(time::Duration::from_millis(20));
                continue;
            }
        }

        let v1 = q.pop().unwrap();
        let v2 = q.pop().unwrap();
        drop(q);

        let res = v1.merge(v2);
        queue.lock().unwrap().push(res);
        pb.lock().unwrap().inc(1);
    }
}


fn process_image(entry: &PathBuf, queue: Arc<Mutex<Vec<image::Image>>>, pb: Arc<Mutex<ProgressBar>>, index: usize, num_images: usize, mode: CometMode) {
    let intensity = match mode {
        CometMode::Falling => 1.0 - index as f32 / num_images as f32,
        CometMode::Raising => index as f32 / num_images as f32,
        CometMode::Normal => 1.0,
    };

    let img = image::Image::load_from_raw(entry.as_path(), intensity).unwrap();
    queue.lock().unwrap().push(img);
    pb.lock().unwrap().inc(1);
}


fn write_dng(img: image::Image, out_file: &Path, exif_raw_file: &Path) {
    let (ptr, len, _cap) = img.raw_image_data.into_raw_parts();
    println!("Result images has size {} with height {} and width {}. Writing to '{}'...", len, img.height, img.width, out_file.display());

    unsafe {
        buildDNG(ptr, img.width.try_into().unwrap(), img.height.try_into().unwrap(),
                 CString::new(out_file.as_os_str().to_str().unwrap()).unwrap().as_ptr(),
                 CString::new(exif_raw_file.as_os_str().to_str().unwrap()).unwrap().as_ptr()
        );
    }
}
