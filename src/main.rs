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
use uuid::Uuid;

const VERSION: &str = "0.2.0";
const AUTHOR: &str = "Jaakko Leskinen <jaakko.leskinen@gmail.com>";

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
    let mut stats = Stats::new();
    let (source, target, rename_files) = parse_args();
    let in_dir = PathBuf::from(&source);
    let out_dir = PathBuf::from(&target);

    println!("Scanning images, stand by...");
    let pb = create_progress_bar(&in_dir);

    for file in WalkDir::new(&in_dir).into_iter().filter_map(|file| file.ok()) {
        stats.total_files += 1;
        pb.inc(1);

        if !is_valid_image_file(&file) {
            continue;
        }

        stats.total_images += 1;
        let (width, height) = get_image_dimensions(&file);

        if width <= 1920 {
            continue;
        }

        stats.total_hd += 1;
        process_hd_image(&file, width, height, &in_dir, &out_dir, rename_files, &mut stats);
    }

    pb.finish_with_message("done");
    print_stats(&stats);
}

struct Stats {
    total_landscape: u32,
    total_portrait: u32,
    total_hd: u32,
    total_skipped: u32,
    total_copied: u32,
    total_images: u32,
    total_files: u32,
    total_bytes: u64,
    total_suitable: u32,
    total_unsuitable: u32,
    aspect_ratios: HashSet<String>,
}

impl Stats {
    fn new() -> Self {
        Stats {
            total_landscape: 0,
            total_portrait: 0,
            total_hd: 0,
            total_skipped: 0,
            total_copied: 0,
            total_images: 0,
            total_files: 0,
            total_bytes: 0,
            total_suitable: 0,
            total_unsuitable: 0,
            aspect_ratios: HashSet::new(),
        }
    }
}

fn parse_args() -> (String, String, bool) {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    if args[1] == "-v" || args[1] == "--version" {
        println!("Nerja v{}", VERSION);
        std::process::exit(0);
    }

    let source = args[1].clone();
    let target = if args.len() >= 3 { args[2].clone() } else { String::new() };
    let rename_files = args.len() >= 4 && args[3] == "-g";

    validate_paths(&source, &target);
    (source, target, rename_files)
}

fn print_usage() {
    println!("Nerja v{} by {}", VERSION, AUTHOR);
    println!("Usage: nerja <SOURCE> [TARGET] [-g]");
    println!();
    println!("This program scans the SOURCE for *.jpg, *.jpeg or *.png images that are");
    println!("in landscape orientation, and are more than 1920 pixels wide.");
    println!();
    println!("Found images are copied recursively to the TARGET with original folder structure.");
    println!("If TARGET path is not set, Nerja will only scan and report the SOURCE folder.");
    println!();
    println!("Options:");
    println!("    SOURCE         Source path to scan for images (quote paths with spaces)");
    println!("    TARGET         Optional. Target folder to copy HD-quality landscape images");
    println!("    -g             Optional when target set. Rename target file names using random GUID");
}

fn validate_paths(source: &str, target: &str) {
    if !Path::new(source).is_dir() {
        println!("Error: Source path \"{}\" does not exist!", source);
        std::process::exit(1);
    }
    if !target.is_empty() && !Path::new(target).is_dir() {
        println!("Error: Target path \"{}\" does not exist!", target);
        std::process::exit(1);
    }
}

fn create_progress_bar(in_dir: &Path) -> ProgressBar {
    let max_files_count = WalkDir::new(in_dir).into_iter().filter_map(|file| file.ok()).count();
    ProgressBar::new(max_files_count.try_into().unwrap())
}

fn is_valid_image_file(file: &walkdir::DirEntry) -> bool {
    if !file.metadata().unwrap().is_file() {
        return false;
    }
    let file_path = file.path().display().to_string();
    let file_extension = get_extension_from_filename(&file_path);
    matches!(file_extension, Some("jpg") | Some("jpeg") | Some("png"))
}

fn get_image_dimensions(file: &walkdir::DirEntry) -> (u32, u32) {
    let imagefilename = file.path().display().to_string();
    match size(imagefilename) {
        Ok(dim) => (dim.width as u32, dim.height as u32),
        Err(_) => (0, 0),
    }
}

fn process_hd_image(file: &walkdir::DirEntry, width: u32, height: u32, in_dir: &Path, out_dir: &Path, rename_files: bool, stats: &mut Stats) {
    if width == 0 || height == 0 {
        return;
    }

    if width > height {
        stats.total_landscape += 1;
        let aspect_ratio = calculate_aspect_ratio(width, height);
        stats.aspect_ratios.insert(aspect_ratio.clone());

        let widescreen_suitable = is_widescreen_suitable(&aspect_ratio);
        if widescreen_suitable {
            stats.total_suitable += 1;
        } else {
            stats.total_unsuitable += 1;
        }

        if !out_dir.as_os_str().is_empty() {
            copy_image(file, in_dir, out_dir, widescreen_suitable, rename_files, stats);
        }
    } else if width < height {
        stats.total_portrait += 1;
    }
}

fn calculate_aspect_ratio(width: u32, height: u32) -> String {
    let mut gcd_cache: HashMap<String, usize> = HashMap::new();
    let r = gcd_cached(width as usize, height as usize, &mut gcd_cache);
    format!("{}:{}", width / r as u32, height / r as u32)
}

fn is_widescreen_suitable(aspect_ratio: &str) -> bool {
    let parts: Vec<&str> = aspect_ratio.split(':').collect();
    let ratio = parts[0].parse::<f64>().unwrap() / parts[1].parse::<f64>().unwrap();
    ratio >= 1.6 && ratio <= 2.7
}

fn copy_image(file: &walkdir::DirEntry, in_dir: &Path, out_dir: &Path, widescreen_suitable: bool, rename_files: bool, stats: &mut Stats) {
    let from = file.path();
    let mut to = out_dir.to_path_buf();
    let subdir = if widescreen_suitable { "widescreen" } else { "normal" };
    to = to.join(subdir);

    if rename_files {
        to = generate_unique_filename(to, file);
    } else {
        to = to.join(from.strip_prefix(in_dir).unwrap());
    }

    let to_dir = to.parent().unwrap();
    if !to_dir.is_dir() {
        fs::create_dir_all(to_dir).expect("destination path creation failed");
    }

    if to.exists() {
        stats.total_skipped += 1;
    } else {
        match fs::copy(from, to) {
            Ok(bytes_copied) => {
                stats.total_copied += 1;
                stats.total_bytes += bytes_copied;
            },
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn generate_unique_filename(mut to: PathBuf, file: &walkdir::DirEntry) -> PathBuf {
    use std::fs::File;
    use std::io::{BufReader, Read};
    use blake3;

    let original_extension = file.path().extension().and_then(OsStr::to_str).unwrap_or("");

    let checksum = {
        let file = File::open(file.path()).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 65536]; // Increased buffer size for better performance
        loop {
            let count = reader.read(&mut buffer).expect("Failed to read file");
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize().to_hex()
    };
    
    let new_filename = format!("{}.{}", checksum, original_extension);
    to = to.join(new_filename);
    if !to.exists() {
        return to;
    }

    let mut attempts = 0;
    loop {
        let new_filename = format!("{}.{}", Uuid::new_v4().to_string(), original_extension);
        to = to.with_file_name(new_filename);
        if !to.exists() || attempts > 10 {
            break;
        }
        attempts += 1;
    }
    if attempts > 10 {
        println!("warning: all generated UUID filenames for this source already exists: {}", file.path().display());
    }
    to
}

fn print_stats(stats: &Stats) {
    println!("┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ SUMMARY                                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ HD Images:        {:5} ({:5} landscape, {:5} portrait)               │", stats.total_hd, stats.total_landscape, stats.total_portrait);
    println!("│ Wide Screen:      {:5} suitable, {:5} unsuitable                      │", stats.total_suitable, stats.total_unsuitable);
    println!("│ Processing:       {:5} skipped, {:5} copied (HD landscape)            │", stats.total_skipped, stats.total_copied);
    println!("│ Total:            {:5} files, {:5} images, {:12} bytes         │", stats.total_files, stats.total_images, stats.total_bytes);
    println!("└─────────────────────────────────────────────────────────────────────────┘");
}
