mod core;
mod handlers;
mod mongo_store;
use actix_web::{
    middleware::Logger,
    web::{get, post, Data},
};
use dotenv;
use env_logger;
use mongo_store::MongoStore;
use mongodb::Client;
use mongodb_gridfs::GridFSBucket;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let client = Client::with_uri_str(
        dotenv::var("MONGODB_URL").expect("MONGODB_URL environment variable not exists"),
    )
    .await
    .unwrap();
    actix_web::HttpServer::new(move || {
        let bucket = GridFSBucket::new(client.database("with-baby-store"), None);
        let collection = client.database("with-baby-store").collection("files");
        let store = Data::new(MongoStore::new(bucket, collection));
        actix_web::App::new()
            .wrap(Logger::default())
            .app_data(store)
            .route("/{id}", get().to(handlers::get::<MongoStore>))
            .route("/", post().to(handlers::put::<MongoStore>))
            .route("/{id}/info", get().to(handlers::info::<MongoStore>))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
