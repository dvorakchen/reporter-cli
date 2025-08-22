use reqwest::Client;
use scraper::Selector;

use crate::news::{
    MaterialExtractor, NewsCrawler, NewsMaterial, NewsMaterialError, NewsMaterialResult, NewsTitle,
};

pub const SOURCE_NAME: &str = "pengpai";

pub struct PengPaiNews {
    client: Client,
}

impl PengPaiNews {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for PengPaiNews {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NewsCrawler for PengPaiNews {
    async fn get_hot_news_list(&self) -> Vec<super::NewsTitle> {
        let url = "https://cache.thepaper.cn/contentapi/wwwIndex/rightSidebar";

        let expect_msg = &format!("PengPai request url {} failed", url);
        let response = self.client.get(url).send().await.expect(expect_msg);

        let json: ResponseContent = response.json().await.expect(expect_msg);

        let resp_data = json.data.hot_news;

        resp_data
            .into_iter()
            .map(PengPaiHotNews::to_hot_news)
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
struct ResponseContent {
    pub data: Data,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    #[serde(rename = "hotNews")]
    pub hot_news: Vec<PengPaiHotNews>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct PengPaiHotNews {
    #[serde(rename = "contId")]
    pub cont_id: String,
    pub name: String,
    #[serde(rename = "pic")]
    pub cover: String,
    pub videos: Option<Video>,
}

impl PengPaiHotNews {
    pub fn to_hot_news(self) -> NewsTitle {
        let mut pics = vec![self.cover];
        let mut videos = vec![];

        if let Some(video) = self.videos {
            videos.push(video.url);
            pics.push(video.cover);
        }

        NewsTitle {
            source: SOURCE_NAME.to_owned(),
            title: self.name,
            url: format!(
                "https://www.thepaper.cn/newsDetail_forward_{}",
                self.cont_id
            ),
            pics,
            videos,
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Video {
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "coverUrlFirstFrame")]
    pub cover: String,
}

pub struct PengPaiNewsMaterialExtractor {
    key: String,
    url: String,
    model: String,
}

impl PengPaiNewsMaterialExtractor {
    pub fn new(
        openai_url: impl AsRef<str>,
        openai_key: impl AsRef<str>,
        model: impl AsRef<str>,
    ) -> Self {
        Self {
            key: openai_key.as_ref().to_string(),
            url: openai_url.as_ref().to_string(),
            model: model.as_ref().to_string(),
        }
    }
    //pub fn from_ali(api_key: impl Into<String>) -> Self {
    //    Self {
    //        key: api_key.into(),
    //        url: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_owned(),
    //        model: "qwen-turbo",
    //    }
    //}
}

#[async_trait::async_trait]
impl MaterialExtractor for PengPaiNewsMaterialExtractor {
    async fn get_material(&self, hot_news: &NewsTitle) -> NewsMaterialResult {
        let http = Client::new();
        let response = http
            .get(hot_news.url.clone())
            .send()
            .await
            .map_err(|e| NewsMaterialError::NetWork(e.to_string()))?;

        let raw_content = response
            .text()
            .await
            .map_err(|e| NewsMaterialError::NetWork(e.to_string()))?;

        let body = self.get_body_inner_text(raw_content);

        let openai_result = self
            .ask_openai(&body, &self.model, &self.key, &self.url)
            .await
            .map_err(|e| NewsMaterialError::AI(e.to_string()))?;

        let mut pics = hot_news.pics.clone();
        pics.extend(openai_result.images);

        Ok(NewsMaterial {
            title: hot_news.title.clone(),
            summary: openai_result.summary.clone(),
            videos: hot_news.videos.clone(),
            pics,
            emotion: openai_result.emotion,
        })
    }

    fn get_body_inner_text(&self, raw_content: String) -> String {
        use scraper::Html;
        let document = Html::parse_document(&raw_content);
        let main = Selector::parse("main").unwrap();

        let mut mains = document.select(&main);
        let main = mains.next().unwrap();
        main.inner_html()
    }
}
