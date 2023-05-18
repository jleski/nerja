extern crate walkdir;
use walkdir::WalkDir;
use imagesize::size;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use indicatif::ProgressBar;
use std::env::args;

fn get_extension_from_filename(filename: &str) -> Option<&str> {    
    Path::new(filename)
    .extension()
    .and_then(OsStr::to_str)
}

fn gcd (a:usize, b: usize) -> usize{
    if b == 0 {
        return a
    }
    return gcd (b, a % b)
}

fn gcd_cached (a: usize, b: usize, cache: &mut HashMap<String, usize>) -> usize {
    let cache_key = format!("{}x{}", a, b);
    if cache.contains_key(&cache_key) {
        return cache[&cache_key];
    }
    let r = gcd(a,b);
    cache.insert(cache_key, r);
    return r;
}

fn main() {
    let mut total_landscape = 0;
    let mut total_portrait = 0;
    let mut total_hd = 0;
    let mut total_skipped = 0;
    let mut total_copied = 0;
    let mut total_images = 0;
    let mut total_files = 0;
    let mut total_bytes: u64 = 0;
    let mut total_suitable = 0;
    let mut total_unsuitable = 0;
    let mut source: String = "".to_owned();
    let mut target: String = "".to_owned();
    let mut out_dir: PathBuf = PathBuf::new();
    let mut gcd_cache: HashMap<String, usize> = HashMap::new();
    let mut aspect_ratios: HashSet<String> = HashSet::new();
    if args().count() < 2 {
        println!("Usage: nerja <SOURCE> [TARGET]

This program scans the SOURCE for *.jpg, *.jpeg or *.png images that are
in landscape orientation, and are more than 1920 pixels wide.

Found images are copied recursively to the TARGET with original folder structure.
If TARGET path is not set, Nerja will only scan and report the SOURCE folder.

Options:
    SOURCE      Source path to scan for images (quote paths with spaces).
    TARGET      Optional. Target folder to copy HD-quality landscape images.
");
        return
    }
    if args().count() >= 2 {
        source = args().nth(1).unwrap();
        println!("Scan source: \t\"{}\"", source);
        if !Path::new(source.as_str()).is_dir() {
            println!("Error: Source path \"{}\" does not exist!", source);
            return
        }
    }
    if args().count() >= 3 {
        target = args().nth(2).unwrap();
        println!("Target path: \t\"{}\"", target);
        if !Path::new(target.as_str()).is_dir() {
            println!("Error: Target path \"{}\" does not exist!", target);
            return
        }
        out_dir = PathBuf::from(target.clone());
    }
    let in_dir = PathBuf::from(source);
    println!("Scanning images, stand by...");
    let max_files_count = WalkDir::new(&in_dir).into_iter().filter_map(|file| file.ok()).count();
    let pb = ProgressBar::new(max_files_count.try_into().unwrap());
    for file in WalkDir::new(&in_dir).into_iter().filter_map(|file| file.ok()) {
        total_files += 1;
        pb.inc(1);
        let imagefilename = file.path().display().to_string();
        let file_extension = get_extension_from_filename(&*imagefilename);
        if file.metadata().unwrap().is_file() && (file_extension == Some("jpg") || file_extension == Some("jpeg") || file_extension == Some("png")){
            total_images += 1;
            let (width, height) = match size(imagefilename.to_string()) {
                Ok(dim) => (dim.width, dim.height),
                Err(_) => (0, 0),
            };
            if width > 1920 {
                total_hd += 1;
                if width > 0 && height > 0 {
                    if width > height {
                        let mut widescreen_suitable = false;
                        let width_ceil = (((width/100) as f64).round() as usize) * 100;
                        let height_ceil = (((height/100) as f64).round() as usize) * 100;
                        total_landscape += 1;
                        let r = gcd_cached(width_ceil, height_ceil, &mut gcd_cache);
                        aspect_ratios.insert(format!("{}:{}",width_ceil/r,height_ceil/r));
                        let aspect_ratio_decimal = (width_ceil / r) as f64 / (height_ceil/r) as f64;
                        if aspect_ratio_decimal >= 1.6 && aspect_ratio_decimal <= 1.9 {
                            total_suitable += 1;
                            widescreen_suitable = true;
                        } else {
                            total_unsuitable += 1;
                        }
                        if target.is_empty() {
                            // report only
                            continue
                        }
                        let from = file.path();
                        let path_to_copy = from.strip_prefix(&in_dir)
                            .expect("path is not part of the prefix");
                        let mut to = out_dir.clone();
                        if widescreen_suitable {
                            to = to.join("widescreen")
                        } else {
                            to = to.join("normal")
                        }
                        //let to = out_dir.join(path_to_copy);
                        to = to.join(path_to_copy);
                        let to_dir = to.parent().expect("target path must be in some directory");
                        if !Path::new(to_dir).is_dir() {
                            fs::create_dir_all(to_dir).expect("destination path creation failed");
                        }
                        if std::path::Path::new(to.as_os_str()).exists() {
                            total_skipped += 1;
                        } else {
                            let result = fs::copy(from,to.clone());
                            match result {
                                Ok(bytes_copied) => {
                                    total_copied += 1;
                                    total_bytes += bytes_copied
                                },
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                    } else if width < height {
                        total_portrait += 1;
                    }
                }
            }
        }
    }
    pb.finish_with_message("done");
    println!("TOTAL {} HD images, {} landscape and {} portrait", total_hd, total_landscape, total_portrait);
    println!("WIDE SCREEN+ SUITABLE {} images and {} unsuitable images", total_suitable, total_unsuitable);
    println!("SKIPPED {}, COPIED {} HD landscape images", total_skipped, total_copied);
    println!("total of {} files, {} images, {} bytes", total_files, total_images, total_bytes);
}
