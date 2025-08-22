use std::path::PathBuf;

use crate::news::NewsMaterial;

const DEFAULT_BGM: &str = "general_bgm_01.mp3";
const DEFAULT_HEAVY_BGM: &str = "heavy _bgm_01.mp3";
const DEFAULT_BUSINESS_POSITIVE_BGM: &str = "business_positive_bgm_01.mp3";
const DEFAULT_BUSINESS_NEGATIVE_BGM: &str = "business_negative_bgm.mp3";
const DEFAULT_OBITUARY_BGM: &str = "obituary_bgm_01.mp3";

pub(crate) fn get_bgm(material: &NewsMaterial) -> PathBuf {
    log::debug!("get_bgm emotion: {:?}", material);

    let file = match material.emotion {
        crate::news::Emotion::Heavy => DEFAULT_HEAVY_BGM,
        crate::news::Emotion::General => DEFAULT_BGM,
        crate::news::Emotion::BusinessPositive => DEFAULT_BUSINESS_POSITIVE_BGM,
        crate::news::Emotion::BusinessNegative => DEFAULT_BUSINESS_NEGATIVE_BGM,
        crate::news::Emotion::Obituary => DEFAULT_OBITUARY_BGM,
    };

    PathBuf::from(".").join("assets").join("bgm").join(file)
}

#[cfg(test)]
mod tests {
    use crate::news::Emotion;

    use super::*;

    #[test]
    fn test_get_bgm_for_heavy_emotion() {
        let material = NewsMaterial {
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::Heavy,
        };
        let expected_path = PathBuf::from(".").join("assets").join("bgm").join(DEFAULT_HEAVY_BGM);
        let actual_path = get_bgm(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bgm_for_general_emotion() {
        let material = NewsMaterial {
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::General,
        };
        let expected_path = PathBuf::from(".").join("assets").join("bgm").join(DEFAULT_BGM);
        let actual_path = get_bgm(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bgm_for_business_positive_emotion() {
        let material = NewsMaterial {
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::BusinessPositive,
        };
        let expected_path = PathBuf::from(".").join("assets").join("bgm").join(DEFAULT_BUSINESS_POSITIVE_BGM);
        let actual_path = get_bgm(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bgm_for_business_negative_emotion() {
        let material = NewsMaterial {
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::BusinessNegative,
        };
        let expected_path = PathBuf::from(".").join("assets").join("bgm").join(DEFAULT_BUSINESS_NEGATIVE_BGM);
        let actual_path = get_bgm(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bgm_for_obituary_emotion() {
        let material = NewsMaterial {
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::Obituary,
        };
        let expected_path = PathBuf::from(".").join("assets").join("bgm").join(DEFAULT_OBITUARY_BGM);
        let actual_path = get_bgm(&material);
        assert_eq!(actual_path, expected_path);
    }
}