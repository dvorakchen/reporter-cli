use std::collections::HashMap;

use thiserror::Error;

use crate::{
    news::{
        MaterialExtractor, NewsMaterial, NewsMaterialError, NewsTitle,
        ithome::{self, ITHomeNewsMaterialExtractor},
        pengpai_news,
    },
    source::SourceName,
};

#[derive(Error, Debug)]
pub enum MaterialRetrieverError {
    #[error("source: {0}")]
    Source(String),
    #[error("material: {0}")]
    Material(#[from] NewsMaterialError),
}

pub struct MaterialRetriever {
    sources: HashMap<SourceName, Box<dyn MaterialExtractor>>,
}

impl MaterialRetriever {
    pub fn new(openai_url: String, openai_key: String, model: String) -> Self {
        let mut sources: HashMap<String, Box<dyn MaterialExtractor>> = HashMap::new();

        sources.insert(
            pengpai_news::SOURCE_NAME.to_string(),
            Box::new(pengpai_news::PengPaiNewsMaterialExtractor::new(
                &openai_url,
                &openai_key,
                &model,
            )),
        );

        sources.insert(
            ithome::SOURCE_NAME.to_string(),
            Box::new(ITHomeNewsMaterialExtractor::new(
                &openai_url,
                &openai_key,
                &model,
            )),
        );

        Self { sources }
    }

    pub async fn get_material(
        &self,
        news_title: &NewsTitle,
    ) -> Result<NewsMaterial, MaterialRetrieverError> {
        let news_source =
            self.sources
                .get(&news_title.source)
                .ok_or(MaterialRetrieverError::Source(
                    news_title.source.to_string(),
                ))?;

        let result = news_source.get_material(news_title).await?;

        Ok(result)
    }
}
