mod converter;

use converter::{convert, get_ext};
use std::io::{self, Write};
use std::path::Path;

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

fn batch_convert(folder: &Path, target_fmt: &str) {
    if !folder.is_dir() {
        println!("Not a folder: {}", folder.display());
        return;
    }

    let folder_name = folder
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let output_dir = folder
        .parent()
        .unwrap_or(folder)
        .join(format!("{folder_name}-duckvert"));

    let entries: Vec<_> = match std::fs::read_dir(folder) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect(),
        Err(e) => {
            println!("Failed to read folder: {e}");
            return;
        }
    };

    if entries.is_empty() {
        println!("No files found in folder: {}", folder.display());
        return;
    }

    let mut success = 0u32;
    let mut processed = 0u32;

    for entry in &entries {
        let path = entry.path();
        if get_ext(&path).is_none() {
            println!(
                "Skipping unsupported extension: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            continue;
        }
        processed += 1;
        println!(
            "Converting {} from {} to {target_fmt}...",
            path.display(),
            get_ext(&path).unwrap()
        );
        if convert(&path, target_fmt, Some(&output_dir)).is_some() {
            success += 1;
        } else {
            println!("Failed conversion: {}", path.display());
        }
    }

    println!(
        "Batch conversion complete: {success}/{processed} converted to {target_fmt} in {}",
        output_dir.display()
    );
}

fn main() {
    println!("Created by SavageDuck26"); // Please keep :D
    println!("{MESSAGE}");

    let file_path = read_line("Enter file path or folder path to convert: ");
    let target_format = read_line("Enter target format (e.g., pdf, jpg, mp3): ");

    let path = Path::new(&file_path);

    if path.is_dir() {
        batch_convert(path, &target_format);
        return;
    }

    let ext = get_ext(path);
    if ext.is_none() {
        println!("Unsupported file type.");
        return;
    }

    println!(
        "Converting {} from {} to {target_format}...",
        path.display(),
        ext.unwrap()
    );

    match convert(path, &target_format, None) {
        Some(out) => println!("Converted: {}", out.display()),
        None => println!("Failed conversion: {}", path.display()),
    }
}
