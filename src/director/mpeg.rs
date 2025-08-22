use std::{path::{Path, PathBuf}, process::Command};

use thiserror::Error;

use crate::news::NewsMaterial;

#[derive(Error, Debug)]
pub enum MpegToolError {
    #[error("init tool failed: {0}")]
    Init(String),
    #[error("Failed to handle voice: {0}")]
    Voice(String),
    #[error("Failed to to compose video, voice, subtitle: {0}")]
    Compose(String),
    #[error("Failed to edit post video")]
    EditPostVideo(String),
}

const DEFAULT_FONT: &str = "assets/fonts/SmileySans-Oblique.ttf";

pub struct FFmpeg {
    font_path: &'static str,
}

impl FFmpeg {
    pub fn new() -> Self {
        Self {
            font_path: DEFAULT_FONT,
        }
    }
}

impl FFmpeg {
    pub async fn compose_images(
        &self,
        file_list_path: &PathBuf,
        bg_img: &PathBuf,
        output: &PathBuf,
    ) -> Result<(), MpegToolError> {
        let ffmpeg = Command::new("ffmpeg")
.arg("-y")
        .arg("-loop").arg("1")
        .arg("-i").arg(bg_img)
        .arg("-f").arg("concat")
        .arg("-i").arg(file_list_path)
        .arg("-filter_complex")
        .arg("[0:v]scale=720:1280[bg];[1:v]scale=720:-1[img];[bg][img]overlay=(W-w)/2:(H-h)/2:shortest=1[out]")
        .arg("-map").arg("[out]")
        .arg("-c:v").arg("libx264")
        .arg("-r").arg("30")
        .arg("-pix_fmt").arg("yuv420p")
        .arg(output)
            .output().expect("failed to execute ffmpeg");

        if !ffmpeg.status.success() {
            eprintln!("ffmpeg stderr: {}", String::from_utf8_lossy(&ffmpeg.stderr));
            return Err(MpegToolError::Compose(
                "failed to compose_images".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn compose_all(
        &self,
        video_input: &PathBuf,
        voice_input: &PathBuf,
        subtitle_input: &PathBuf,
        bgm_inpu: &PathBuf,
        output: &Path,
    ) -> Result<(), MpegToolError> {
        let s = subtitle_input.display().to_string().replace('\\', "/");

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(video_input)
            .arg("-i")
            .arg(voice_input)
            .arg("-i")
            .arg(bgm_inpu)
            .arg("-filter_complex")
            .arg(&format!(
                "[2:a]volume=0.4[a2]; [1:a][a2]amix=inputs=2:weights=1|0.3:duration=first[a]; [0:v]subtitles='{}':force_style='Fontsize=10,Alignment=2,MarginV=70'[v]",
                s))
            .arg("-map")
            .arg("[v]")
            .arg("-map")
            .arg("[a]")
            .arg("-c:v")
            .arg("libx264")
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("192k")
            .arg("-shortest")
            .arg("-y")
            .arg(output)
            .output().expect("failed to execute ffmpeg");

        if !ffmpeg.status.success() {
            eprintln!("ffmpeg stderr: {}", String::from_utf8_lossy(&ffmpeg.stderr));
            return Err(MpegToolError::Compose(
                "failed to compose_images".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn edit_post_video(
        &self,
        video_input: &PathBuf,
        material: &NewsMaterial,
        author: &str,
        output: &PathBuf,
    ) -> Result<(), MpegToolError> {
        let cover_subtitle = material
            .title
            .chars()
            .collect::<Vec<_>>()
            .chunks(8)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        let content_subtitle = material
            .title
            .chars()
            .collect::<Vec<_>>()
            .chunks(17)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        let mut vf = format!(
            "drawtext=expansion=normal:text='{}':fontcolor=yellow:fontsize=100:fontfile={}:x=(w-text_w)/2:y=200:bordercolor=black:borderw=6:enable='between(t,0,0.001)',\
            drawtext=expansion=normal:text='{}':fontcolor=yellow:fontsize=50:fontfile={}:x=(w-text_w)/2:y=150:bordercolor=black:borderw=6:enable='between(t,0.001,9999)'",
            cover_subtitle, self.font_path, content_subtitle, self.font_path,
        );

        if !author.is_empty() {
            vf.push_str(&format!(
            ",drawtext=expansion=normal:text='@{}':fontcolor=#0095d7:fontsize=45:fontfile={}:x=(w-text_w)/2:y=1200:bordercolor=yellow:borderw=6:enable='between(t,0,9999)'", author, self.font_path));
        }

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(video_input)
            .arg("-vf")
            .arg(vf)
            .arg("-c:a")
            .arg("copy")
            .arg("-y")
            .arg(&output)
            .output()
            .expect("failed to execute ffmpeg");

        /*
        ffmpeg -i post-video.mp4
        -vf "drawtext=expansion=normal:text='欢迎观看欢迎观\n看欢迎观看':fontcolor=yellow:fontsize=120:fontfile=SmileySans-Oblique.ttf:x=(w-text_w)/2:y=300:bordercolor=black:borderw=6:enable='between(t,0,0.001)',\
        drawtext=expansion=normal:text='欢迎观看欢迎观\n看欢迎观看':fontcolor=yellow:fontsize=84:fontfile=SmileySans-Oblique.ttf:x=(w-text_w)/2:y=150:bordercolor=black:borderw=6:enable='between(t,0.001,9999)'"
        -c:a copy -y output.mp4
                */

        if !ffmpeg.status.success() {
            eprintln!("ffmpeg stderr: {}", String::from_utf8_lossy(&ffmpeg.stderr));
            return Err(MpegToolError::Compose(
                "failed to compose_images".to_string(),
            ));
        }

        Ok(())
    }
}
