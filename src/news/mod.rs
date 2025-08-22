pub mod ithome;
pub mod pengpai_news;

use crate::source::SourceName;

use openai::chat::ChatCompletion;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsTitle {
    pub source: SourceName,
    pub title: String,
    pub url: String,
    pub pics: Vec<String>,
    pub videos: Vec<String>,
}

impl NewsTitle {
    /// extracts the material from this news
    pub async fn get_news_material<T>(&self, material_extractor: &T) -> NewsMaterialResult
    where
        T: MaterialExtractor,
    {
        material_extractor.get_material(self).await
    }
}

pub type NewsMaterialResult = Result<NewsMaterial, NewsMaterialError>;

#[derive(Error, Debug)]
pub enum NewsMaterialError {
    #[error("NetWork: {0}")]
    NetWork(String),
    #[error("AI: {0}")]
    AI(String),
    #[error("Material: {0}")]
    Material(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Emotion {
    General,
    Heavy,
    BusinessNegative,
    BusinessPositive,
    Obituary,
}
#[derive(Debug)]
pub struct NewsMaterial {
    pub title: String,
    pub summary: Vec<String>,
    pub pics: Vec<String>,
    pub videos: Vec<String>,
    pub emotion: Emotion,
}

/// crawler, indicats how to get the news title list
#[async_trait::async_trait]
pub trait NewsCrawler: Sync + Send {
    async fn get_hot_news_list(&self) -> Vec<NewsTitle>;
}

/// Extractor, responsible for extractor news material: news summary, pictures, videos.
#[async_trait::async_trait]
pub trait MaterialExtractor: Sync + Send {
    async fn get_material(&self, hot_news: &NewsTitle) -> NewsMaterialResult;

    fn get_prompt(&self) -> &'static str {
        let prompt = include_str!("./prompt.md");
        prompt
    }

    fn get_body_inner_text(&self, raw_content: String) -> String;

    async fn ask_openai(
        &self,
        news_content: &str,
        model: &str,
        key: &str,
        url: &str,
    ) -> Result<OpenAIResult, String> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.append(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", key)).unwrap(),
        );
        headers.append(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let body = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": self.get_prompt()
                },
                {
                    "role": "user",
                    "content": news_content
                }
            ]
        })
        .to_string();

        let response = client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .expect("Failed to send OpenAI request");

        let data: ChatCompletion = response.json().await.expect("OpenAI JSON failed");
        let content = data
            .choices
            .first()
            .as_ref()
            .unwrap()
            .message
            .content
            .as_ref()
            .unwrap()
            .trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_end_matches("```")
            .trim()
            .to_string();

        log::debug!("raw_json: {}", content);
        let res: OpenAIResult = serde_json::from_str(&content).unwrap();

        Ok(res)
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct OpenAIResult {
    pub summary: Vec<String>,
    pub images: Vec<String>,
    pub emotion: Emotion,
}
