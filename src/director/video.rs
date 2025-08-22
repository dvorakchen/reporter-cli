use rayon::prelude::*;
use reqwest::Client;
use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

use crate::{
    director::mpeg::{FFmpeg, MpegToolError},
    get_temp_dir,
    images::{is_image_path, is_image_url},
    news::NewsMaterial,
};

const IMAGE_INTERNAL_DURATION: Duration = Duration::from_secs(3);

pub type VideoEditorResult<T> = Result<T, VideoEditorError>;

#[derive(Error, Debug)]
pub enum VideoEditorError {
    #[error("tts has not set")]
    NoSet,
    #[error("network request error: {0}")]
    NetWork(String),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Image error: {0}")]
    Image(String),
    #[error("duration empty or video editor has not set")]
    Duration,
    #[error("video editor tool error:  {0}")]
    Tool(#[from] MpegToolError),
}

pub struct JuniorEditor;

impl JuniorEditor {
    pub async fn do_edit(
        material: &NewsMaterial,
        dur: Duration,
        bg_img: &PathBuf,
    ) -> VideoEditorResult<PathBuf> {
        let pic_urls = Self::get_need_pics(&material.pics, dur);
        let pic_files = Self::save_pics(pic_urls).await?;
        // edit pics
        let mixed_pic_files = crate::images::mix_pics(&pic_files, bg_img);

        let path = Self::compose_pics(&mixed_pic_files, bg_img).await?;

        pic_files.par_iter().for_each(|pic| {
            log::debug!("pic: {}", pic.display());
            _ = std::fs::remove_file(pic);
        });

        Ok(path)
    }
}

impl JuniorEditor {
    fn get_need_pics(pics: &[String], dur: Duration) -> Vec<String> {
        let mut pic_need = (dur.as_secs() / 2).max(1) as usize;
        if dur.subsec_millis() > 0 {
            pic_need += 1;
        }

        let pic_count = pics.len();

        if pic_need <= pic_count {
            pics[..pic_need].to_vec()
        } else {
            let (full_cycles, remainder) = (pic_need / pic_count, pic_need % pic_count);

            let mut list = Vec::with_capacity(pic_need);

            for _ in 0..full_cycles {
                list.extend_from_slice(pics);
            }

            if remainder > 0 {
                list.extend_from_slice(&pics[..remainder]);
            }

            list
        }
    }

    async fn save_pics(pic_links: Vec<String>) -> VideoEditorResult<Vec<PathBuf>> {
        let id = Uuid::new_v4().to_string();

        let mut pic_files = vec![];

        let mut handle_list = vec![];

        for (index, pic_url) in pic_links.iter().filter(|pic| is_image_url(pic)).enumerate() {
            let id = id.clone();
            let pic_url = pic_url.clone();
            let temp_dir = get_temp_dir().await;

            let handle =
                tokio::spawn(
                    async move { Self::download_pic(index, &pic_url, &id, &temp_dir).await },
                );

            handle_list.push(handle);
        }

        for handle in handle_list {
            if let Ok(res) = handle.await {
                match res {
                    Ok(path) => {
                        pic_files.push(path);
                    }
                    Err(e) => log::error!("Failed to download pic: {:?}", e),
                }
            } else {
                log::error!("Failed to await handle download pic");
            }
        }

        pic_files.retain(|pic| is_image_path(pic.as_path()));

        Ok(pic_files)
    }

    async fn download_pic(
        index: usize,
        pic_url: &str,
        id: &str,
        temp_dir: &Path,
    ) -> VideoEditorResult<PathBuf> {
        let http = Client::new();
        let response = http
            .get(pic_url)
            .send()
            .await
            .map_err(|e| VideoEditorError::NetWork(e.to_string()))?;

        let format = {
            let content_type = response
                .headers()
                .get("content-type")
                .ok_or(VideoEditorError::NetWork(
                    "cannot find image format".to_owned(),
                ))?
                .to_str()
                .map_err(|_| {
                    VideoEditorError::NetWork(
                        "Failed to get header of response content-type".to_owned(),
                    )
                })?;

            content_type
                .split('/')
                .nth(1)
                .ok_or(VideoEditorError::NetWork(
                    "malformed content-type".to_owned(),
                ))?
                .to_string()
        };

        let bytes = response
            .bytes()
            .await
            .map_err(|e| VideoEditorError::NetWork(e.to_string()))?;

        let file_path = temp_dir.join(format!("{}{:03}.{}", id, index, format));
        fs::write(&file_path, bytes).await?;

        Ok(file_path)
    }

    async fn compose_pics(pics: &Vec<PathBuf>, bg_img: &PathBuf) -> VideoEditorResult<PathBuf> {
        if pics.is_empty() {
            return Err(VideoEditorError::Image("no pics to compose".to_owned()));
        }

        let temp_dir = get_temp_dir().await;
        let output_path = PathBuf::from(format!(
            "{}/{}-composed.mp4",
            temp_dir.display(),
            Uuid::new_v4()
        ));

        let file_list_path = Self::build_file_list(pics).await?;

        FFmpeg::new()
            .compose_images(&file_list_path, bg_img, &output_path)
            .await?;

        _ = fs::remove_file(file_list_path).await;

        for pic in pics {
            _ = fs::remove_file(pic).await;
        }
        Ok(output_path)
    }

    async fn build_file_list(pics: &Vec<PathBuf>) -> VideoEditorResult<PathBuf> {
        if pics.is_empty() {
            return Err(VideoEditorError::Image("no pics to compose".to_owned()));
        }

        let temp_dir = get_temp_dir().await;
        let file_list_path = PathBuf::from(format!(
            "{}/{}-file-list.txt",
            temp_dir.display(),
            Uuid::new_v4()
        ));

        let mut file_list_content = String::new();
        for pic in pics {
            let file_name = pic
                .file_name()
                .ok_or(VideoEditorError::Image(format!(
                    "Failed to get filename from {}",
                    pic.display()
                )))?
                .to_str()
                .ok_or(VideoEditorError::Image(format!(
                    "Failed to get filename from {}",
                    pic.display()
                )))?;

            file_list_content.push_str(&format!("file '{}'\n", file_name));
            file_list_content.push_str(&format!(
                "duration {}\n\n",
                IMAGE_INTERNAL_DURATION.as_secs()
            ));
        }

        file_list_content.push_str(&format!(
            "file '{}'\n",
            pics.first()
                .expect("The pics for build file_list should not be empyt")
                .file_name()
                .ok_or(VideoEditorError::Image(
                    "Failed to get filename from pics.get(0)".to_owned()
                ))?
                .to_str()
                .ok_or(VideoEditorError::Image(
                    "Failed to get filename from pics.get(0)".to_owned()
                ))?
        ));
        file_list_content.push_str("duration 2\n\n");

        fs::write(&file_list_path, file_list_content).await?;

        Ok(file_list_path)
    }
}
