pub mod bg_img;
pub mod bgm;
pub mod material;
pub mod mpeg;
pub mod subtitle;
pub mod video;

use crate::{
    director::{
        bg_img::get_bg_img,
        bgm::get_bgm,
        material::MaterialRetrieverError,
        mpeg::{FFmpeg, MpegToolError},
        subtitle::{SingleSubtitle, SubtitleError},
        video::{JuniorEditor, VideoEditorError},
    },
    get_temp_dir,
    news::{NewsMaterial, NewsMaterialError, NewsTitle},
    tts::{TTSError, TTSService, get_wav_len, reporter_tts::ReporterTTS},
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

pub type DirectorResult<T> = Result<T, DirectorError>;

#[derive(Error, Debug)]
pub enum DirectorError {
    #[error("fail occurred: {0}")]
    Source(String),
    #[error("wav error: {0}")]
    WavReader(#[from] hound::Error),
    #[error("material retriever: {0}")]
    MaterialRetriever(#[from] MaterialRetrieverError),
    #[error("tts error: {0}")]
    TTS(#[from] TTSError),
    #[error("subtitle error: {0}")]
    Subtitle(#[from] SubtitleError),
    #[error("file error")]
    IO(#[from] std::io::Error),
    #[error("video editor error: {0}")]
    VideoEditor(#[from] VideoEditorError),
    #[error("video editor tool error: {0}")]
    MpegTool(#[from] MpegToolError),
    //  #[error("wav error: {0}")]
    //  WavReader(#[from] hound::Error),
}

#[derive(Serialize)]
pub struct NewsShortVideo {
    pub title: String,
    pub path: PathBuf,
}

pub struct Director {
    author: String,
    tts_url: String,
    openai_url: String,
    openai_key: String,
    model: String,
}

impl Director {
    pub fn new(
        author: String,
        tts_url: String,
        openai_url: String,
        openai_key: String,
        model: String,
    ) -> Self {
        Self {
            author,
            tts_url,
            openai_url,
            openai_key,
            model,
        }
    }

    pub async fn shot_single(&self, news_title: NewsTitle, output: &Path) -> DirectorResult<NewsShortVideo> {
        /*
           1. get material
           2. generate voice
           3. generate subtitle
           4. generate video without voice
           5. generate BGM
           6. edit video -- add primary title .etc
           7. compose all
        */

        let material = self.get_material(&news_title).await?;

        if material.pics.is_empty() {
            return Err(DirectorError::MaterialRetriever(
                MaterialRetrieverError::Material(NewsMaterialError::Material(
                    "pics not found".to_string(),
                )),
            ));
        }

        let voice_subtitle = self.generate_voice(&material).await?;

        let subtitle_path = self.generate_subtitle(&voice_subtitle.tts_files).await?;

        let video_path = self
            .generate_video(&material, &voice_subtitle.voice_path)
            .await?;

        let bgm_path = get_bgm(&material);
        let posted_video_path = self.edit_post_video(&video_path, &material).await?;

        self
            .compose_all(
                &posted_video_path,
                &voice_subtitle.voice_path,
                &subtitle_path,
                &bgm_path,
                output
            )
            .await?;

        _ = fs::remove_file(video_path).await;
        _ = fs::remove_file(posted_video_path).await;
        _ = fs::remove_file(voice_subtitle.voice_path).await;
        _ = fs::remove_file(subtitle_path).await;

        Ok(NewsShortVideo {
            title: material.title.clone(),
            path: output.to_path_buf(),
        })
    }

    pub async fn get_material(&self, news_title: &NewsTitle) -> DirectorResult<NewsMaterial> {
        let retriever = material::MaterialRetriever::new(
            self.openai_url.clone(),
            self.openai_key.clone(),
            self.model.clone(),
        );

        let result = retriever.get_material(news_title).await?;
        Ok(result)
    }

    async fn generate_voice(&self, material: &NewsMaterial) -> DirectorResult<VoiceSubtitle> {
        let tts = ReporterTTS::new(self.tts_url.clone());
        let tts_files = tts.tts(&material.summary).await?;

        let compose_path = crate::tts::compose_audio(&tts_files).await;

        let subtitles: Vec<SingleSubtitle> = tts_files
            .into_iter()
            .map(|tts| {
                _ = std::fs::remove_file(&tts.path);
                SingleSubtitle {
                    text: tts.text.clone(),
                    duration: tts.duration,
                }
            })
            .collect();

        Ok(VoiceSubtitle {
            voice_path: compose_path,
            tts_files: subtitles,
        })
    }

    async fn generate_subtitle(&self, tts_files: &[SingleSubtitle]) -> DirectorResult<PathBuf> {
        let path = subtitle::SrtSubtitle::write_subtitle(tts_files).await?;
        Ok(path)
    }

    async fn generate_video(
        &self,
        material: &NewsMaterial,
        voice_path: &PathBuf,
    ) -> DirectorResult<PathBuf> {
        let time = get_wav_len(voice_path).await?;

        let bg_img = get_bg_img(material);

        let path = JuniorEditor::do_edit(material, time, &bg_img).await?;
        Ok(path)
    }

    async fn edit_post_video(
        &self,
        video_path: &PathBuf,
        material: &NewsMaterial,
    ) -> DirectorResult<PathBuf> {
        let output = get_temp_dir().await.join(format!("{}.mp4", Uuid::new_v4()));

        FFmpeg::new()
            .edit_post_video(video_path, material, &self.author, &output)
            .await?;

        Ok(output)
    }
    async fn compose_all(
        &self,
        video: &PathBuf,
        dubbing: &PathBuf,
        subtitle: &PathBuf,
        bgm: &PathBuf,
        output: &Path,
    ) -> DirectorResult<()> {
        FFmpeg::new()
            .compose_all(video, dubbing, subtitle, bgm, output)
            .await?;

        Ok(())
    }
}

struct VoiceSubtitle {
    voice_path: PathBuf,
    tts_files: Vec<SingleSubtitle>,
}
