use anyhow::Error;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub mime: String,
    pub owner: String,
    pub key: String,
    pub create_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInput {
    pub name: String,
    pub bytes: Vec<u8>,
    pub owner: String,
}

#[derive(Debug, Serialize)]
pub struct FileOutput {
    pub name: String,
    pub ext: String,
    pub owner: String,
    pub key: String,
    pub create_at: String,
    pub bytes: Vec<u8>,
}

pub trait Store {
    fn put(&self, file: FileInput) -> Pin<Box<dyn Future<Output = Result<String, Error>>>>;
    fn get(
        &self,
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Pin<Box<dyn Stream<Item = Vec<u8>>>>, Error>>>>;
    fn info(
        &self,
        id: impl AsRef<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<FileInfo>, Error>>>>;
}
