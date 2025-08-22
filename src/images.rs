use image::{ImageBuffer, ImageReader, Rgba};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::path::{Path, PathBuf};

pub(crate) fn mix_pics(input: &Vec<PathBuf>, bg_img: &PathBuf) -> Vec<PathBuf> {
    let list: Vec<PathBuf> = input
        .par_iter()
        .map(|pic| mix_background_foreground(bg_img, pic))
        .collect();

    list
}

fn blend_pixels(background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
    let alpha = foreground.0[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba([
        (background.0[0] as f32 * inv_alpha + foreground.0[0] as f32 * alpha) as u8,
        (background.0[1] as f32 * inv_alpha + foreground.0[1] as f32 * alpha) as u8,
        (background.0[2] as f32 * inv_alpha + foreground.0[2] as f32 * alpha) as u8,
        (background.0[3] as f32 * inv_alpha + foreground.0[3] as f32 * alpha) as u8,
    ])
}

fn mix_background_foreground(bg_img: &PathBuf, foreground: &PathBuf) -> PathBuf {
    let new_file_path = get_mixed_name(foreground);

    let background_img = image::open(bg_img).expect(&format!(
        "load background image failed: {}",
        bg_img.display()
    ));
    let foreground_img = ImageReader::open(&foreground)
        .expect(&format!(
            "ImageReader open foreground image failed: {}",
            foreground.display()
        ))
        .with_guessed_format()
        .expect(&format!(
            "guess foreground image format failed: {}",
            foreground.display()
        ))
        .decode()
        .expect(&format!(
            "expect foreground image failed: {}",
            foreground.display()
        ));

    let background_img = background_img.to_rgba8();
    let foreground_img = foreground_img.to_rgba8();

    let (bg_width, bg_height) = background_img.dimensions();
    let (fg_width, fg_height) = foreground_img.dimensions();

    let width_scale = bg_width as f32 / fg_width as f32;
    let height_scale = bg_height as f32 / fg_height as f32;
    let scale = width_scale.min(height_scale).min(1.0);

    let new_fg_width = (fg_width as f32 * scale) as u32;
    let new_fg_height = (fg_height as f32 * scale) as u32;

    let resized_fg = image::imageops::resize(
        &foreground_img,
        new_fg_width,
        new_fg_height,
        image::imageops::FilterType::Lanczos3,
    );

    let mut output_img = ImageBuffer::new(bg_width, bg_height);

    for (x, y, pixel) in background_img.enumerate_pixels() {
        output_img.put_pixel(x, y, *pixel);
    }

    let x_pos = (bg_width - new_fg_width) / 2;
    let y_pos = (bg_height - new_fg_height) / 2;

    for (x, y, pixel) in resized_fg.enumerate_pixels() {
        let out_x = x + x_pos;
        let out_y = y + y_pos;

        if out_x < bg_width && out_y < bg_height {
            let bg_pixel = output_img.get_pixel(out_x, out_y);
            let blended = blend_pixels(*bg_pixel, *pixel);
            output_img.put_pixel(out_x, out_y, blended);
        }
    }

    // 10. 保存结果
    output_img
        .save(&new_file_path)
        .expect(&format!("save image failed: {}", new_file_path.display()));

    new_file_path
}

fn get_mixed_name(pic: &PathBuf) -> PathBuf {
    let mut new_file_path = pic.clone();
    new_file_path.set_file_name(format!(
        "{}-mixed.png",
        new_file_path.file_stem().unwrap().to_str().unwrap()
    ));

    new_file_path
}

pub fn is_image_path(filename: &Path) -> bool {
    let image_extensions = vec!["png", "jpg", "jpeg", "gif", "webp"];

    if let Some(ext) = filename.extension()
        && let Some(ext) = ext.to_str()
    {
        return image_extensions.contains(&ext);
    }

    false
}

pub fn is_image_url(url: impl AsRef<str>) -> bool {
    // 定义一个正则表达式，匹配常见的图片文件类型
    let pattern = Regex::new(r"(?i)\.(jpeg|jpg|png|gif|bmp|webp|tiff)(\?.*)?$").unwrap();

    // 检查字符串是否匹配正则表达式
    pattern.is_match(url.as_ref())
}
