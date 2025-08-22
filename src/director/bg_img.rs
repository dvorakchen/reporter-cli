use std::path::PathBuf;

use crate::news::NewsMaterial;

const DEFAULT_BG_IMG_HEAVY: &str = "bg-img-heavy.png";
const DEFAULT_BG_IMG_BUSINESS_POSITIVE: &str = "bg-img-business-positive.png";
const DEFAULT_BG_IMG_BUSINESS_NEGATIVE: &str = "bg-img-business-negative.png";
const DEFAULT_BG_IMG_OBITUARY: &str = "bg-img-obituary.png";
const DEFAULT_BG_IMG: &str = "bg-img-0.png";

/// get background image
pub(crate) fn get_bg_img(material: &NewsMaterial) -> PathBuf {
    let name = match material.emotion {
        crate::news::Emotion::Heavy => DEFAULT_BG_IMG_HEAVY,
        crate::news::Emotion::BusinessNegative => DEFAULT_BG_IMG_BUSINESS_NEGATIVE,
        crate::news::Emotion::BusinessPositive => DEFAULT_BG_IMG_BUSINESS_POSITIVE,
        crate::news::Emotion::Obituary => DEFAULT_BG_IMG_OBITUARY,
        crate::news::Emotion::General => DEFAULT_BG_IMG,
    };

    PathBuf::from(".").join("assets").join("bg-img").join(name)
}

#[cfg(test)]
mod tests {
    use crate::news::Emotion;

    use super::*;

    // `#[test]` 属性将函数标记为一个测试用例
    #[test]
    fn test_get_bg_img_for_heavy_emotion() {
        let material = NewsMaterial { 
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::Heavy 
        };
        let expected_path = PathBuf::from(".").join("assets").join("bg-img").join(DEFAULT_BG_IMG_HEAVY);
        let actual_path = get_bg_img(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bg_img_for_business_negative_emotion() {
        let material = NewsMaterial { 
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::BusinessNegative 
        };
        let expected_path = PathBuf::from(".").join("assets").join("bg-img").join(DEFAULT_BG_IMG_BUSINESS_NEGATIVE);
        let actual_path = get_bg_img(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bg_img_for_business_positive_emotion() {
        let material = NewsMaterial { 
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::BusinessPositive 
        };
        let expected_path = PathBuf::from(".").join("assets").join("bg-img").join(DEFAULT_BG_IMG_BUSINESS_POSITIVE);
        let actual_path = get_bg_img(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bg_img_for_obituary_emotion() {
        let material = NewsMaterial { 
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::Obituary 
        };
        let expected_path = PathBuf::from(".").join("assets").join("bg-img").join(DEFAULT_BG_IMG_OBITUARY);
        let actual_path = get_bg_img(&material);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_get_bg_img_for_general_emotion() {
        let material = NewsMaterial { 
            title: String::new(),
            summary: vec![],
            pics: vec![],
            videos: vec![],
            emotion: Emotion::General 
        };
        let expected_path = PathBuf::from(".").join("assets").join("bg-img").join(DEFAULT_BG_IMG);
        let actual_path = get_bg_img(&material);
        assert_eq!(actual_path, expected_path);
    }
}