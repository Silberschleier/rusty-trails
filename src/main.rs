#![feature(vec_into_raw_parts)]

use std::{io, thread};
use std::path::{PathBuf, Path};
use std::sync::{Arc, Mutex};
use std::ffi::CString;
use std::os::raw::c_char;
use rayon::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::convert::TryInto;
use glob::glob;
use clap::{crate_version, Clap};
use crate::image::ImagePrototype;

mod image;
mod merger;


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
    /// Pattern of dark frames. Use with quotation marks, e.g.: "directory/darks/*.CR2"
    #[clap(short = "d", long = "darks", default_value="")]
    darks: String,
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

    let darks = match_files(opts.darks);
    let mut lights = match_files(opts.input);
    println!("Light frames: {},  Dark frames: {},  Mode: {}", lights.len(), darks.len(), opts.mode);

    // Setup CLI progressbar
    let m = MultiProgress::new();
    let sty_load = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.green/cyan}] {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");
    let sty_blend = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .progress_chars("##-");
    let pb_decode = Arc::new(Mutex::new(m.add(ProgressBar::new((lights.len() + darks.len() - 1) as u64))));
    let pb_blend_lights = m.add(ProgressBar::new(lights.len() as u64 - 1));
    let pb_blend_darks = m.add(ProgressBar::new(darks.len() as u64));
    pb_decode.lock().unwrap().set_style(sty_load);
    pb_decode.lock().unwrap().set_message("Decode files");
    pb_blend_lights.set_style(sty_blend.clone());
    pb_blend_darks.set_style(sty_blend);
    pb_blend_lights.set_message("Blend Lights");
    pb_blend_darks.set_message("Blend Darks");

    let pb_thread = thread::spawn(move || {
        m.join().unwrap();
    });

    // Prepare mergers
    let mut light_merger = merger::Merger::new(merger::MergeAction::Max, pb_blend_lights);
    let mut dark_merger = merger::Merger::new(merger::MergeAction::Add, pb_blend_darks);
    light_merger.spawn_workers(num_threads);
    dark_merger.spawn_workers(num_threads);


    // Load darks
    darks.par_iter()
        .map(|p| ImagePrototype {
            path: PathBuf::from(p),
            intensity: 1.0 / darks.len() as f32,
            image_type: image::ImageType::Dark
        })
        .for_each(|p| load_image(&p, dark_merger.get_queue(), Arc::clone(&pb_decode)));

    dark_merger.finish();


    // Load lights
    lights.sort();
    lights.par_iter()
        .zip(0..lights.len())
        .map(|(p, i)| ImagePrototype {
            path: PathBuf::from(p),
            intensity: determine_intensity(mode, i, lights.len()),
            image_type: image::ImageType::Light
        })
        .for_each(|p| load_image(&p, light_merger.get_queue(), Arc::clone(&pb_decode)));


    pb_decode.lock().unwrap().finish();

    let master_dark = dark_merger.finish_and_join();
    let merged_light = light_merger.finish_and_join().unwrap();
    pb_thread.join().unwrap_or(());

    let raw_image = match master_dark {
        Some(dark) => { println!("Subtracting master dark."); merged_light.subtract(dark)},
        None => merged_light
    };

    write_dng(raw_image, Path::new(&opts.output), Path::new(lights.first().unwrap()));
    Ok(())
}

fn match_files(pattern: String) -> Vec<PathBuf> {
    let mut entries = vec![];

    for entry in glob(&pattern).expect("Failed to match glob") {
        let e = entry.unwrap();
        if e.is_file() {
            entries.push(e);
        }
    }

    entries
}

fn determine_intensity(mode: CometMode, index: usize, num_images: usize) -> f32 {
    match mode {
        CometMode::Falling => 1.0 - index as f32 / num_images as f32,
        CometMode::Raising => index as f32 / num_images as f32,
        CometMode::Normal => 1.0,
    }
}

fn load_image(proto: &ImagePrototype, queue: Arc<Mutex<Vec<image::Image>>>, pb: Arc<Mutex<ProgressBar>>) {
    let img = image::Image::load_from_raw(proto.path.as_path(), proto.intensity).unwrap().apply_intensity();
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
