use crate::core::{FileInfo, FileInput, Store};
use anyhow::Error;
use chrono::Utc;
use futures::Stream;
use mongodb::bson::doc;
use mongodb::{bson::oid::ObjectId, Collection};
use mongodb_gridfs::GridFSBucket;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct MongoStore {
    bucket: GridFSBucket,
    collection: Collection<FileInfo>,
}

impl MongoStore {
    pub fn new(bucket: GridFSBucket, collection: Collection<FileInfo>) -> Self {
        Self { bucket, collection }
    }
}

impl Store for MongoStore {
    fn put(
        &self,
        f: FileInput,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>>>> {
        let mut b = self.bucket.clone();
        let c = self.collection.clone();
        return Box::pin(async move {
            let mime = infer::get(&f.bytes)
                .ok_or(Error::msg("unknown file type"))?
                .mime_type()
                .to_owned();
            let id = b
                .upload_from_stream(&f.name, f.bytes.as_ref(), None)
                .await
                .map_err(|e| Error::new(e))?;
            c.insert_one(
                FileInfo {
                    name: f.name,
                    mime: mime,
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
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Pin<Box<dyn Stream<Item = Vec<u8>>>>, Error>>>> {
        let b = self.bucket.clone();
        let id = id.to_owned();
        Box::pin(async move {
            let s = b
                .open_download_stream(ObjectId::parse_str(&id).map_err(|e| Error::from(e))?)
                .await
                .map_err(|e| Error::new(e))?;
            let res: Pin<Box<dyn Stream<Item = Vec<u8>>>> = Box::pin(s);
            Ok(res)
        })
    }

    fn info(
        &self,
        id: impl AsRef<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<FileInfo>, Error>>>> {
        let c = self.collection.clone();
        let id = id.as_ref().to_owned();
        return Box::pin(async move {
            let info = c
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
    use futures::StreamExt;
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
        let mut f = store.get("635bdd289395ef004c776291").await.unwrap();
        let mut l = Vec::new();
        while let Some(mut item) = f.next().await {
            l.append(&mut item)
        }
        println!("{:?}", l)
    }
}
