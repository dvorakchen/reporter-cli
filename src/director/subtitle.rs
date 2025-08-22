use std::{io, path::PathBuf, time::Duration};
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

use crate::get_temp_dir;

pub type SubtitleResult = Result<PathBuf, SubtitleError>;

/// one single subtitle
pub struct SingleSubtitle {
    pub text: String,
    pub duration: Duration,
}

#[derive(Error, Debug)]
pub enum SubtitleError {
    #[error("file error: {0}")]
    IO(#[from] io::Error),
    #[error("subtitle has not set")]
    NoSet,
}

// #[async_trait::async_trait]
// pub trait Subtitle: Sync + Send {
//     async fn write_subtitle(&self, subtitles: &Vec<SingleSubtitle>) -> SubtitleResult;
// }

pub struct SrtSubtitle;

impl SrtSubtitle {
    pub async fn write_subtitle(subtitles: &[SingleSubtitle]) -> SubtitleResult {
        let mut content = String::with_capacity(subtitles.len() * 13 * 10);

        let mut index = 1usize;
        let mut time = Duration::from_secs(0);
        let gap_time = Duration::from_millis(200);

        for subtitle in subtitles {
            let (single, end_time) =
                Self::gen_single_subtitle(&subtitle.text, index, time, subtitle.duration);

            content.push_str(&single);
            time = end_time + gap_time;
            index += 1;
        }

        let mut path = get_temp_dir().await;

        path = path.join(format!("{}.srt", Uuid::new_v4()));

        fs::write(&path, content).await?;

        Ok(path)
    }

    fn gen_single_subtitle(
        text: &str,
        index: usize,
        start: Duration,
        time: Duration,
    ) -> (String, Duration) {
        let mut content = String::new();

        content.push_str(&index.to_string());
        content.push('\n');

        let start_time = Self::to_subtitle_time_string(&start);
        content.push_str(&start_time);
        content.push_str(" --> ");

        let end_time = start + time;
        let end_time_str = Self::to_subtitle_time_string(&end_time);

        content.push_str(&end_time_str);
        content.push('\n');
        content.push_str(text);
        content.push_str("\n\n");

        (content, end_time)
    }

    fn to_subtitle_time_string(time: &Duration) -> String {
        let total_secs = time.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = time.subsec_millis();

        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::fs;


    #[tokio::test]
    async fn test_write_subtitle() {
        let subtitles = vec![
            SingleSubtitle {
                text: "Hello, World!".to_string(),
                duration: Duration::from_secs(2),
            },
            SingleSubtitle {
                text: "This is a test.".to_string(),
                duration: Duration::from_secs(3),
            },
        ];

        let result = SrtSubtitle::write_subtitle(&subtitles).await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).await.unwrap();
        let expected_content = "1\n00:00:00,000 --> 00:00:02,000\nHello, World!\n\n2\n00:00:02,200 --> 00:00:05,200\nThis is a test.\n\n";
        
        assert_eq!(content, expected_content);

        // 清理测试生成的文件
        fs::remove_file(&path).await.unwrap();
    }

    #[test]
    fn test_gen_single_subtitle() {
        let text = "Test subtitle text";
        let index = 1;
        let start = Duration::from_secs(10);
        let time = Duration::from_secs(5);
        
        let (content, end_time) = SrtSubtitle::gen_single_subtitle(text, index, start, time);
        
        let expected_content = "1\n00:00:10,000 --> 00:00:15,000\nTest subtitle text\n\n";
        assert_eq!(content, expected_content);
        
        let expected_end_time = Duration::from_secs(15);
        assert_eq!(end_time, expected_end_time);
    }

    #[test]
    fn test_to_subtitle_time_string_basic() {
        let duration = Duration::from_secs(62) + Duration::from_millis(500);
        let result = SrtSubtitle::to_subtitle_time_string(&duration);
        assert_eq!(result, "00:01:02,500");
    }
    
    #[test]
    fn test_to_subtitle_time_string_long() {
        let duration = Duration::from_secs(3665) + Duration::from_millis(123);
        let result = SrtSubtitle::to_subtitle_time_string(&duration);
        assert_eq!(result, "01:01:05,123");
    }

    #[test]
    fn test_to_subtitle_time_string_zero() {
        let duration = Duration::from_secs(0);
        let result = SrtSubtitle::to_subtitle_time_string(&duration);
        assert_eq!(result, "00:00:00,000");
    }
}