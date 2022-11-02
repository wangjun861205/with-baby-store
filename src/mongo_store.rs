use crate::core::{FileInfo, FileInput, FileOutput, Store};
use anyhow::Error;
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::{bson::oid::ObjectId, Collection};
use mongodb_gridfs::GridFSBucket;
use std::future::ready;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct MongoStore {
    bucket: Arc<Mutex<GridFSBucket>>,
    collection: Arc<Mutex<Collection<FileInfo>>>,
}

impl MongoStore {
    pub fn new(bucket: GridFSBucket, collection: Collection<FileInfo>) -> Self {
        Self {
            bucket: Arc::new(Mutex::new(bucket)),
            collection: Arc::new(Mutex::new(collection)),
        }
    }
}

impl Store for MongoStore {
    fn put(
        &self,
        f: FileInput,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>>>> {
        let b = self.bucket.clone();
        let c = self.collection.clone();
        return Box::pin(async move {
            let ext = infer::get(&f.bytes)
                .ok_or(Error::msg("unknown file type"))?
                .mime_type()
                .to_owned();
            let id = b
                .lock()
                .unwrap()
                .upload_from_stream(&f.name, f.bytes.as_ref(), None)
                .await
                .map_err(|e| Error::new(e))?;
            c.lock()
                .unwrap()
                .insert_one(
                    FileInfo {
                        name: f.name,
                        ext: ext,
                        owner: f.owner,
                        key: id.to_hex(),
                        create_at: Utc::now().to_rfc3339(),
                    },
                    None,
                )
                .await?;
            Ok(id.to_hex())
        });
    }

    fn get(
        &self,
        id: impl AsRef<str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, Error>>>> {
        let b = self.bucket.clone();
        let c = self.collection.clone();
        let id = id.as_ref().to_owned();
        return Box::pin(async move {
            let mut bs: Vec<u8> = Vec::new();
            b.lock()
                .unwrap()
                .open_download_stream(ObjectId::parse_str(&id).map_err(|e| Error::from(e))?)
                .await
                .map_err(|e| Error::new(e))?
                .for_each(|v| {
                    for b in v {
                        bs.push(b);
                    }
                    ready(())
                })
                .await;
            Ok(bs)
        });
    }

    fn info(
        &self,
        id: impl AsRef<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<FileInfo>, Error>>>> {
        let c = self.collection.clone();
        let id = id.as_ref().to_owned();
        return Box::pin(async move {
            let info = c
                .lock()
                .map_err(|e| Error::msg(e.to_string()))?
                .find_one(
                    Some(doc! {
                        "key": &id
                    }),
                    None,
                )
                .await
                .map_err(|e| Error::new(e))?;
            Ok(info)
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_mongo_put() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let db = client.database("with-baby-store-test");
        let bucket = GridFSBucket::new(db.clone(), None);
        let collection = db.collection("files");
        let store = MongoStore::new(bucket, collection);
        let file = FileInput {
            name: "test.txt".into(),
            bytes: b"hello world".to_vec(),
            owner: "wangjun".into(),
        };
        let id = store.put(file).await.unwrap();
        println!("{}", id);
    }

    #[tokio::test]
    async fn test_mongo_get() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let db = client.database("with-baby-store-test");
        let bucket = GridFSBucket::new(db.clone(), None);
        let collection = db.collection("files");
        let store = MongoStore::new(bucket, collection);
        let f = store.get("635bdd289395ef004c776291").await.unwrap();
        println!("{:?}", f)
    }
}
