use std::{
    fmt::Display,
    str::{from_utf8, Utf8Error},
};

use crate::core::{FileInfo, FileInput, FileOutput, Store};
use actix_multipart::Multipart;
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    web::{Data, Json, Path},
    HttpResponse, ResponseError,
};
use bytes::{BufMut, BytesMut};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::with_body(self.status_code(), BoxBody::new(self.0.to_string()))
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl From<actix_multipart::MultipartError> for Error {
    fn from(e: actix_multipart::MultipartError) -> Self {
        Self(anyhow::Error::new(e))
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Self(anyhow::Error::new(e))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn get<S: Store>(
    store: Data<S>,
    id: Path<(String,)>,
) -> Result<Json<Option<FileOutput>>, Error> {
    let f = store.get(&id.0).await?;
    Ok(Json(f))
}

pub async fn info<S: Store>(
    store: Data<S>,
    id: Path<(String,)>,
) -> Result<Json<Option<FileInfo>>, Error> {
    let f = store.info(&id.0).await?;
    Ok(Json(f))
}

#[derive(Debug, Serialize)]
pub struct PutResponse {
    id: String,
}

pub async fn put<S: Store>(
    store: Data<S>,
    mut parts: Multipart,
) -> Result<Json<PutResponse>, Error> {
    let mut input = FileInput {
        name: "".into(),
        owner: "".into(),
        bytes: Vec::new(),
    };
    while let Some(parts) = parts.next().await {
        let mut field = parts?;
        let mut buffer = BytesMut::with_capacity(1024);
        while let Some(bs) = field.next().await {
            let b = bs?;
            buffer.put(b);
        }
        match field.name() {
            "name" => input.name = from_utf8(&buffer)?.to_owned(),
            "owner" => input.owner = from_utf8(&buffer)?.to_owned(),
            "bytes" => input.bytes = buffer.to_vec(),
            _ => {}
        }
    }
    let id = store.put(input).await?;
    Ok(Json(PutResponse { id }))
}
