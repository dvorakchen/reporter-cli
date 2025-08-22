use std::collections::HashMap;

use thiserror::Error;

use crate::{
    news::{
        NewsCrawler, NewsTitle, ithome,
        pengpai_news::{self, PengPaiNews},
    },
    source::SourceName,
};

pub struct HotNewsCrawler {
    sources: HashMap<SourceName, Box<dyn NewsCrawler>>,
}

#[derive(Error, Debug)]
pub enum HotNewsCrawlerError {
    #[error("Unknow source: {0}")]
    Source(String),
}

impl HotNewsCrawler {
    pub fn new() -> Self {
        let mut sources: HashMap<String, Box<dyn NewsCrawler>> = HashMap::new();

        sources.insert(
            pengpai_news::SOURCE_NAME.to_string(),
            Box::new(PengPaiNews::new()),
        );

        sources.insert(
            ithome::SOURCE_NAME.to_string(),
            Box::new(ithome::ITHomeNews::new()),
        );

        Self { sources }
    }

    pub async fn get(&self, source: &SourceName) -> Result<Vec<NewsTitle>, HotNewsCrawlerError> {
        let news_source = self
            .sources
            .get(source)
            .ok_or(HotNewsCrawlerError::Source(source.to_string()))?;

        Ok(news_source.get_hot_news_list().await)
    }
}

impl Default for HotNewsCrawler {
    fn default() -> Self {
        Self::new()
    }
}
