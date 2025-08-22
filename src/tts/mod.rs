pub mod reporter_tts;

use hound::WavReader;
use std::{io, path::PathBuf, time::Duration};
use thiserror::Error;
use uuid::Uuid;

use crate::get_temp_dir;

#[async_trait::async_trait]
pub trait TTSService: Sync + Send {
    async fn tts(&self, text_list: &[String]) -> Result<Vec<TTSFile>, TTSError>;
}

#[derive(Debug)]
pub struct TTSFile {
    // the audio file path
    pub path: PathBuf,
    // text
    pub text: String,
    // this audio duartion
    pub duration: Duration,
}

#[derive(Error, Debug)]
pub enum TTSError {
    #[error("cannot pass empty test")]
    EmptyText,
    #[error("network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Audio error: {0}")]
    Audio(#[from] hound::Error),
}

pub struct TTS;

impl TTS {}

/// Get the audio playback duration in seconds.
pub async fn get_wav_len(path: &PathBuf) -> Result<Duration, TTSError> {
    let reader = WavReader::open(path)?;

    let seconds = reader.duration() as f64 / reader.spec().sample_rate as f64;
    let duration = Duration::from_secs_f64(seconds);

    Ok(duration)
}

pub async fn compose_audio(tts_files: &Vec<TTSFile>) -> PathBuf {
    let spec = {
        let first = tts_files.first().unwrap();
        let reader = WavReader::open(&first.path).expect("Wav open file failed");
        reader.spec()
    };

    let silence = {
        let silence_duration = 0.3;
        let num_samples = (spec.sample_rate as f32 * silence_duration) as usize;
        vec![0; num_samples * spec.channels as usize]
    };

    let mut compose_wav = vec![];

    for tts_file in tts_files {
        let mut reader = WavReader::open(&tts_file.path).unwrap();
        let samples: Vec<i32> = reader.samples::<i16>().map(|s| s.unwrap() as i32).collect();

        compose_wav.extend(samples.clone());
        compose_wav.extend(silence.clone());
    }

    let final_wav = get_temp_dir()
        .await
        .join(format!("{}-final.wav", Uuid::new_v4()));

    let mut writer = hound::WavWriter::create(&final_wav, spec).expect("Wav create failed");

    for sample in compose_wav {
        writer.write_sample(sample).unwrap();
    }

    final_wav
}
