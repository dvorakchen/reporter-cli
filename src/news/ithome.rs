use reqwest::Client;
use scraper::Selector;

use crate::news::{
    MaterialExtractor, NewsCrawler, NewsMaterial, NewsMaterialError, NewsMaterialResult, NewsTitle,
};

pub const SOURCE_NAME: &str = "ithome";

pub struct ITHomeNews {
    client: Client,
}

impl ITHomeNews {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for ITHomeNews {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NewsCrawler for ITHomeNews {
    async fn get_hot_news_list(&self) -> Vec<super::NewsTitle> {
        let url = "https://www.ithome.com/block/rank.html";

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "accept-language",
            "en,zh-CN;q=0.9,zh;q=0.8".parse().unwrap(),
        );
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("pragma", "no-cache".parse().unwrap());
        headers.insert("priority", "u=0, i".parse().unwrap());
        headers.insert(
            "sec-ch-ua",
            "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\""
                .parse()
                .unwrap(),
        );
        headers.insert("sec-ch-ua-mobile", ".unwrap()0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
        headers.insert("sec-fetch-dest", "document".parse().unwrap());
        headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
        headers.insert("sec-fetch-site", "none".parse().unwrap());
        headers.insert("sec-fetch-user", ".unwrap()1".parse().unwrap());
        headers.insert("upgrade-insecure-requests", "1".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());

        let response = self.client.get(url).headers(headers).send().await.unwrap();
        let raw_html = response.text().await.unwrap();

        let html = scraper::Html::parse_fragment(&raw_html);

        let id_1 = if let Some(id_1) = html.select(&Selector::parse("#d-1").unwrap()).nth(0) {
            id_1
        } else {
            return vec![];
        };

        let mut list = vec![];
        for a_tag in id_1.select(&Selector::parse("a").unwrap()) {
            let title = a_tag.attr("title").unwrap_or("");
            let url = a_tag.attr("href").unwrap_or("");
            if title.is_empty() || url.is_empty() {
                continue;
            }

            list.push(NewsTitle {
                source: SOURCE_NAME.to_owned(),
                title: title.to_owned(),
                url: url.to_owned(),
                pics: vec![],
                videos: vec![],
            });
        }

        list
    }
}

pub struct ITHomeNewsMaterialExtractor {
    key: String,
    url: String,
    model: String,
}

impl ITHomeNewsMaterialExtractor {
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
impl MaterialExtractor for ITHomeNewsMaterialExtractor {
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
        let main = Selector::parse("#paragraph").unwrap();

        let mut mains = document.select(&main);
        let main = mains.next().unwrap();
        main.inner_html()
    }
}
