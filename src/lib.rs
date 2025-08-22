use std::path::PathBuf;

use tokio::fs;

pub mod crawler;
pub mod director;
pub mod images;
pub mod news;
pub mod source;
pub mod tts;

pub async fn get_temp_dir() -> PathBuf {
    let temp_dir = PathBuf::from("temp");

    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir)
            .await
            .expect("failed to create temp dir");
    }

    temp_dir
}
