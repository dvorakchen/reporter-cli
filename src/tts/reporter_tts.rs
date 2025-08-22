use crate::get_temp_dir;

use super::get_wav_len;
use super::{TTSError, TTSFile, TTSService};
use async_trait::async_trait;
use reqwest::Url;

pub struct ReporterTTS {
    url: String,
}

impl ReporterTTS {
    pub fn new(url: String) -> Self {
        ReporterTTS { url }
    }
}

#[async_trait]
impl TTSService for ReporterTTS {
    async fn tts(&self, text_list: &[String]) -> Result<Vec<TTSFile>, TTSError> {
        if text_list.is_empty() {
            return Err(TTSError::EmptyText);
        }

        let client = reqwest::Client::new();
        let mut tts_files = Vec::with_capacity(text_list.len());

        for text in text_list {
            // 使用 Url::parse_with_params 自动处理 URL 编码
            let request_url =
                Url::parse_with_params(&format!("{}/tts", self.url), &[("text", text)])
                    .map_err(|e| TTSError::Network(format!("Failed to build URL: {}", e)))?;


            let response = client
                .post(request_url)
                .send()
                .await
                .map_err(|e| TTSError::Network(e.to_string()))?;

            if !response.status().is_success() {
                return Err(TTSError::Network(format!(
                    "HTTP request failed with status: {}",
                    response.status()
                )));
            }

            let temp = get_temp_dir().await;

            let file_path = temp.join(format!("tts_{}.wav", uuid::Uuid::new_v4()));
            let mut file = tokio::fs::File::create(&file_path)
                .await
                .map_err(TTSError::IO)?;
            let content = response
                .bytes()
                .await
                .map_err(|e| TTSError::Network(e.to_string()))?;

            tokio::io::copy(&mut content.as_ref(), &mut file)
                .await
                .map_err(TTSError::IO)?;

            let duration = get_wav_len(&file_path).await?;

            tts_files.push(TTSFile {
                path: file_path,
                text: text.clone(),
                duration,
            });
        }

        Ok(tts_files)
    }
}
