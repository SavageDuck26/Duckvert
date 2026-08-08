mod converter;

use converter::{convert, get_ext};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const MESSAGE: &str = "
All supported formats:
- Image: jpg, jpeg, png, bmp, tiff, tif, gif, webp, ico
- Video: mp4, mkv, mov, avi, ogg, webm, flv, wmv, m4v, mpeg, mpg, ts, 3gp
- Audio: mp3, wav, flac, aac, m4a, ogg, opus, aiff, wma, amr
- Document: txt, pdf, md, doc, docx, rtf, odt, xls, xlsx, ppt, pptx

Note: Audio/video conversions require ffmpeg to be installed.
";

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().trim_matches('"').to_string()
}

/// Crypt-style path input: one path per line, blank line to finish.
/// Non-existent paths are reported and skipped.
fn get_input_paths() -> Vec<PathBuf> {
    println!("Enter paths (One per line, blank line to finish):");
    let mut all_input = String::new();
    loop {
        let mut line = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut line)
            .expect("Failed to read path input from stdin");
        if bytes_read == 0 {
            break;
        }
        let trimmed = line.trim().trim_matches('"').to_string();
        if trimmed.is_empty() {
            if all_input.is_empty() {
                continue;
            }
            break;
        }
        all_input.push_str(&trimmed);
        all_input.push('\n');
    }

    let mut paths = Vec::new();
    for line in all_input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = PathBuf::from(trimmed);
        if path.exists() {
            paths.push(path);
            println!("Added: {trimmed}");
        } else {
            println!("Path does not exist: {trimmed}. Skipping.");
        }
    }
    if paths.is_empty() {
        println!("No valid paths entered.");
    }
    paths
}

/// All converted files go into one shared output directory.
/// Pressing Enter falls back to the first selected path's directory.
/// Non-existent output paths are created.
fn get_output_dir(first: &Path) -> PathBuf {
    let default = if first.is_dir() {
        first.to_path_buf()
    } else {
        first.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    println!(
        "Enter output directory (or press Enter to use {}):",
        default.display()
    );
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read output directory from stdin");
    let trimmed = input.trim().trim_matches('"').to_string();
    if trimmed.is_empty() {
        println!("Using output directory: {}", default.display());
        return default;
    }
    let path = PathBuf::from(&trimmed);
    if path.is_dir() {
        println!("Output directory: {}", path.display());
        return path;
    }
    if let Err(e) = fs::create_dir_all(&path) {
        println!(
            "Could not create output directory '{}' ({e}). Using {}.",
            trimmed,
            default.display()
        );
        return default;
    }
    println!("Output directory: {}", path.display());
    path
}

/// True when `path` lives inside `base` (canonicalized compare). Used to
/// avoid re-converting files already written into the shared output folder.
fn is_within(path: &Path, base: &Path) -> bool {
    let p = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let b = fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    p.starts_with(&b)
}

/// Recursively converts every supported file under `folder` into `output_dir`.
/// Returns (converted, processed).
fn batch_convert(folder: &Path, target_fmt: &str, output_dir: &Path) -> (u32, u32) {
    let mut success = 0u32;
    let mut processed = 0u32;
    let mut total_files = 0u32;

    for entry in WalkDir::new(folder) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                println!("Failed to read entry in {}: {e}", folder.display());
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        // Skip files already written into the output folder (unless the
        // walked folder itself IS the output folder).
        if !is_within(folder, output_dir) && is_within(path, output_dir) {
            continue;
        }
        total_files += 1;
        let ext = match get_ext(path) {
            Some(e) => e,
            None => {
                println!(
                    "Skipping unsupported extension: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                continue;
            }
        };
        processed += 1;
        println!(
            "Converting {} from {} to {target_fmt}...",
            path.display(),
            ext
        );
        if convert(path, target_fmt, Some(output_dir)).is_some() {
            success += 1;
        } else {
            println!("Failed conversion: {}", path.display());
        }
    }

    if total_files == 0 {
        println!("No files found in folder: {}", folder.display());
    } else {
        println!(
            "Folder done: {success}/{processed} converted to {target_fmt} in {}",
            output_dir.display()
        );
    }
    (success, processed)
}

/// Converts a single file into the shared `output_dir`. Returns (converted, processed).
fn convert_file(path: &Path, target_fmt: &str, output_dir: &Path) -> (u32, u32) {
    let ext = match get_ext(path) {
        Some(e) => e,
        None => {
            println!("Unsupported file type: {}", path.display());
            return (0, 0);
        }
    };

    println!(
        "Converting {} from {} to {target_fmt}...",
        path.display(),
        ext
    );
    match convert(path, target_fmt, Some(output_dir)) {
        Some(out) => {
            println!("Converted: {}", out.display());
            (1, 1)
        }
        None => {
            println!("Failed conversion: {}", path.display());
            (0, 1)
        }
    }
}

fn main() {
    println!("Created by SavageDuck26"); // Please keep :D
    println!("{MESSAGE}");

    let paths = get_input_paths();
    if paths.is_empty() {
        return;
    }

    let target_format = read_line("Enter target format (e.g., pdf, jpg, mp3): ");
    let output_dir = get_output_dir(&paths[0]);

    let mut total_success = 0u32;
    let mut total_processed = 0u32;

    for path in &paths {
        println!();
        if path.is_dir() {
            let (s, p) = batch_convert(path, &target_format, &output_dir);
            total_success += s;
            total_processed += p;
        } else {
            let (s, p) = convert_file(path, &target_format, &output_dir);
            total_success += s;
            total_processed += p;
        }
    }

    println!(
        "\nAll done: {total_success}/{total_processed} converted successfully into {}.",
        output_dir.display()
    );
}
