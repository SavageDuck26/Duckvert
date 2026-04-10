use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ::image as img_crate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Image,
    Audio,
    Video,
    Document,
}

pub fn categorize(ext: &str) -> Option<FileCategory> {
    match ext {
        // Images
        "jpg" | "jpeg" | "png" | "bmp" | "tiff" | "tif" | "gif" | "webp" | "ico" => {
            Some(FileCategory::Image)
        }
        // Audio
        "mp3" | "wav" | "flac" | "aac" | "m4a" | "opus" | "aiff" | "wma" | "amr" => {
            Some(FileCategory::Audio)
        }
        // Video  (ogg can be audio OR video)
        "mp4" | "mkv" | "mov" | "avi" | "ogg" | "webm" | "flv" | "wmv" | "m4v" | "mpeg"
        | "mpg" | "ts" | "3gp" => Some(FileCategory::Video),
        // Documents
        "txt" | "pdf" | "md" | "doc" | "docx" | "rtf" | "odt" | "xls" | "xlsx" | "ppt"
        | "pptx" => Some(FileCategory::Document),
        _ => None,
    }
}


pub fn convert(
    src: &Path,
    target_fmt: &str,
    output_dir: Option<&Path>,
) -> Option<PathBuf> {
    let mut attempts: HashSet<(String, String)> = HashSet::new();
    convert_inner(src, target_fmt, output_dir, &mut attempts)
}

fn convert_inner(
    src: &Path,
    target_fmt: &str,
    output_dir: Option<&Path>,
    attempts: &mut HashSet<(String, String)>,
) -> Option<PathBuf> {
    let src_ext = get_ext(src)?;
    let target = target_fmt.to_ascii_lowercase();

    let key = (src_ext.clone(), target.clone());
    if attempts.contains(&key) {
        return None;
    }
    attempts.insert(key);

    if src_ext == target {
        println!("Source is already in {target} format: {}", src.display());
        return Some(src.to_path_buf());
    }

    let src_cat = categorize(&src_ext);
    let tgt_cat = categorize(&target);

    // Image -> Image
    if src_cat == Some(FileCategory::Image) && tgt_cat == Some(FileCategory::Image) {
        return convert_image(src, &target, output_dir);
    }

    // Audio -> Audio  (ffmpeg)
    if src_cat == Some(FileCategory::Audio) && tgt_cat == Some(FileCategory::Audio) {
        return convert_via_ffmpeg(src, &target, output_dir);
    }

    // Video -> Audio  (ffmpeg)
    if src_cat == Some(FileCategory::Video) && tgt_cat == Some(FileCategory::Audio) {
        return convert_via_ffmpeg(src, &target, output_dir);
    }

    // Video -> Video  (ffmpeg)
    if src_cat == Some(FileCategory::Video) && tgt_cat == Some(FileCategory::Video) {
        return convert_via_ffmpeg(src, &target, output_dir);
    }

    // Audio -> Video  (ffmpeg, e.g. mp3 -> ogg)
    if src_cat == Some(FileCategory::Audio) && tgt_cat == Some(FileCategory::Video) {
        return convert_via_ffmpeg(src, &target, output_dir);
    }

    // txt -> pdf
    if src_ext == "txt" && target == "pdf" {
        return convert_text_to_pdf(src, output_dir);
    }

    // pdf -> txt
    if src_ext == "pdf" && target == "txt" {
        return convert_pdf_to_text(src, output_dir);
    }

    // Image -> PDF
    if src_cat == Some(FileCategory::Image) && target == "pdf" {
        return convert_image_to_pdf(src, output_dir);
    }

    // PDF -> Image
    if src_ext == "pdf" && tgt_cat == Some(FileCategory::Image) {
        return convert_pdf_to_image(src, &target, output_dir);
    }

    // Try intermediate conversions
    convert_via_intermediate(src, &target, output_dir, attempts)
}
// ===========================================================================
pub fn get_ext(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let ext = if ext == "jpeg" { "jpg".to_string() } else { ext };
    if categorize(&ext).is_some() {
        Some(ext)
    } else {
        None
    }
}

fn output_path(src: &Path, target_fmt: &str, output_dir: Option<&Path>) -> PathBuf {
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let name = format!("{stem}_converted.{target_fmt}");
    match output_dir {
        Some(dir) => {
            fs::create_dir_all(dir).ok();
            dir.join(name)
        }
        None => src.with_file_name(name),
    }
}

fn has_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
// ===========================================================================
fn convert_image(src: &Path, target_fmt: &str, output_dir: Option<&Path>) -> Option<PathBuf> {
    let out = output_path(src, target_fmt, output_dir);
    match img_crate::open(src) {
        Ok(img) => {
            let img = if matches!(target_fmt, "jpg" | "jpeg") {
                img_crate::DynamicImage::ImageRgb8(img.to_rgb8())
            } else {
                img
            };
            match img.save(&out) {
                Ok(()) => {
                    println!("Image saved to {}", out.display());
                    Some(out)
                }
                Err(e) => {
                    println!("Image conversion failed: {e}");
                    None
                }
            }
        }
        Err(e) => {
            println!("Failed to open image: {e}");
            None
        }
    }
}
// ===========================================================================
fn convert_via_ffmpeg(src: &Path, target_fmt: &str, output_dir: Option<&Path>) -> Option<PathBuf> {
    if !has_ffmpeg() {
        println!("ffmpeg not found — install ffmpeg for audio/video conversion.");
        return None;
    }
    let out = output_path(src, target_fmt, output_dir);
    let result = Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(src)
        .arg(&out)
        .output();

    match result {
        Ok(o) if o.status.success() => {
            println!("Converted via ffmpeg: {}", out.display());
            Some(out)
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            println!("ffmpeg failed: {stderr}");
            None
        }
        Err(e) => {
            println!("ffmpeg execution error: {e}");
            None
        }
    }
}
// ===========================================================================
fn convert_text_to_pdf(src: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let out = output_path(src, "pdf", output_dir);
    let text = match fs::read_to_string(src) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to read text file: {e}");
            return None;
        }
    };

    use printpdf::*;

    let (doc, page1, layer1) = PdfDocument::new("Converted", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 280.0_f32; // mm from bottom
    let line_height = 4.5_f32;
    let font_size = 11.0;

    for line in text.lines() {
        if y < 15.0 {
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(page).get_layer(layer);
            y = 280.0;
        }
        current_layer.use_text(line, font_size, Mm(15.0), Mm(y), &font);
        y -= line_height;
    }

    match doc.save(&mut std::io::BufWriter::new(fs::File::create(&out).ok()?)) {
        Ok(()) => {
            println!("PDF saved to {}", out.display());
            Some(out)
        }
        Err(e) => {
            println!("Text to PDF conversion failed: {e}");
            None
        }
    }
}
// ===========================================================================
fn convert_pdf_to_text(src: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let out = output_path(src, "txt", output_dir);
    let doc = match lopdf::Document::load(src) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to open PDF: {e}");
            return None;
        }
    };

    let mut text = String::new();
    let pages = doc.get_pages();
    let mut page_ids: Vec<_> = pages.iter().collect();
    page_ids.sort_by_key(|(num, _)| *num);

    for (_num, &obj_id) in &page_ids {
        if let Ok(content) = doc.extract_text(&[obj_id.0 as u32]) {
            text.push_str(&content);
            text.push('\n');
        }
    }

    match fs::write(&out, &text) {
        Ok(()) => {
            println!("Text saved to {}", out.display());
            Some(out)
        }
        Err(e) => {
            println!("PDF to text conversion failed: {e}");
            None
        }
    }
}
// ===========================================================================
fn convert_image_to_pdf(src: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let out = output_path(src, "pdf", output_dir);
    let img = match img_crate::open(src) {
        Ok(i) => i.to_rgb8(),
        Err(e) => {
            println!("Failed to open image: {e}");
            return None;
        }
    };

    use printpdf::*;

    let (w, h) = img.dimensions();
    let dpi = 150.0;
    let page_w = Mm(w as f32 / dpi * 25.4);
    let page_h = Mm(h as f32 / dpi * 25.4);

    let (doc, page, layer) = PdfDocument::new("Image", page_w, page_h, "Layer 1");
    let current_layer = doc.get_page(page).get_layer(layer);

    let pdf_image = Image::from(ImageXObject {
        width: Px(w as usize),
        height: Px(h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: img.into_raw(),
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    });

    pdf_image.add_to_layer(current_layer, ImageTransform::default());

    match doc.save(&mut std::io::BufWriter::new(fs::File::create(&out).ok()?)) {
        Ok(()) => {
            println!("Image converted to PDF: {}", out.display());
            Some(out)
        }
        Err(e) => {
            println!("Image to PDF conversion failed: {e}");
            None
        }
    }
}
// ===========================================================================
fn convert_pdf_to_image(src: &Path, target_fmt: &str, output_dir: Option<&Path>) -> Option<PathBuf> {
    // Try ffmpeg as a best-effort fallback
    if has_ffmpeg() {
        return convert_via_ffmpeg(src, target_fmt, output_dir);
    }
    println!("PDF to image conversion requires ffmpeg (or an external tool like poppler).");
    None
}
// ===========================================================================
fn convert_via_intermediate(
    src: &Path,
    target_fmt: &str,
    output_dir: Option<&Path>,
    attempts: &mut HashSet<(String, String)>,
) -> Option<PathBuf> {
    let src_ext = get_ext(src)?;
    let src_cat = categorize(&src_ext)?;
    let _tgt_cat = categorize(target_fmt)?;

    let intermediates: &[&str] = match src_cat {
        FileCategory::Image => &["png", "jpg", "bmp"],
        FileCategory::Audio => &["wav", "mp3", "flac"],
        FileCategory::Video => &["mp4", "mkv"],
        FileCategory::Document => &["txt", "pdf"],
    };

    for &mid in intermediates {
        if mid == src_ext || mid == target_fmt {
            continue;
        }
        if let Some(mid_path) = convert_inner(src, mid, output_dir, attempts) {
            if let Some(final_path) = convert_inner(&mid_path, target_fmt, output_dir, attempts) {
                return Some(final_path);
            }
        }
    }
    println!("Conversion from {src_ext} to {target_fmt} is not supported.");
    None
}
