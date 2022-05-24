use reqwest::{header::LOCATION, Body, Client, Error as ReqwestError, StatusCode};
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::Error as IoError;

#[derive(Debug)]
pub enum TinyPngError {
    Reqwest(ReqwestError),
    Io(IoError),
    Location(&'static str),
    Status(String),
}

impl From<ReqwestError> for TinyPngError {
    fn from(err: ReqwestError) -> Self {
        TinyPngError::Reqwest(err)
    }
}

impl From<IoError> for TinyPngError {
    fn from(err: IoError) -> Self {
        TinyPngError::Io(err)
    }
}

impl From<(StatusCode, String)> for TinyPngError {
    fn from(res: (StatusCode, String)) -> Self {
        TinyPngError::Status(format!("{} {}", res.0, res.1))
    }
}

#[derive(Debug)]
pub struct TinyPng {
    key: String,
    client: Client,
}

pub const REGISTER_URL: &str = "https://tinypng.com/developers";
pub const API_URL: &str = "https://api.tinify.com/shrink";

impl TinyPng {
    pub fn new<T: ToString>(key: T) -> Self {
        Self {
            key: key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn compress_file<F: AsRef<Path>, T: AsRef<Path>>(
        &self,
        from: F,
        to: T,
    ) -> Result<(u64, u64), TinyPngError> {
        let file = File::open(from).await.map_err(TinyPngError::from)?;
        let input_size = file.metadata().await.map_err(TinyPngError::from)?.len();

        let res = self
            .client
            .post(API_URL)
            .basic_auth("api", Some(self.key.as_str()))
            .body(Body::from(file))
            .send()
            .await
            .map_err(TinyPngError::from)?;

        if res.status() != 201 {
            return Err(TinyPngError::from((
                res.status(),
                res.text().await.unwrap_or_default(),
            )));
        }

        let url = res
            .headers()
            .get(LOCATION)
            .ok_or(TinyPngError::Location("Location header not found"))?
            .to_str()
            .map_err(|_| TinyPngError::Location("Location header not valid UTF-8"))?;

        let output_size = self.download_file(url, to).await?;

        Ok((input_size, output_size))
    }

    async fn download_file<P: AsRef<Path>>(&self, url: &str, path: P) -> Result<u64, TinyPngError> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TinyPngError::from)?;

        if res.status() != 200 {
            return Err(TinyPngError::from((
                res.status(),
                res.text().await.unwrap_or_default(),
            )));
        }

        let bytes = res.bytes().await.map_err(TinyPngError::from)?;
        fs::write(path, &bytes).await.map_err(TinyPngError::from)?;

        Ok(bytes.len() as u64)
    }
}
