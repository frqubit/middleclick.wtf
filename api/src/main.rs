use odht::{Config, HashTableOwned, FxHashFn};
use std::sync::{Arc, OnceLock};
use sha2::{Sha256, Digest};
use tokio::sync::Mutex;
use infer::MatcherType;
use thiserror::Error;
use rand::prelude::*;
use log::{info, error};
use actix_web::{
    web, App, HttpResponse,
    HttpServer, Responder,
    post, get,
    middleware::Logger, ResponseError,
    body::{MessageBody, BodySize}
};


/****************************************
* Data
****************************************/
type HashIdTableType = Arc<Mutex<HashTableOwned<HashIds>>>;
static HASHID_TABLE: OnceLock<HashIdTableType> = OnceLock::new();
fn hashid_table() -> &'static HashIdTableType {
    HASHID_TABLE.get_or_init(|| {
        if let Ok(serialized) = std::fs::read("hashids.db") {
            let table = HashTableOwned::<HashIds>::from_raw_bytes(&serialized).unwrap();
            return Arc::new(Mutex::new(table));
        }

        Arc::new(Mutex::new(HashTableOwned::with_capacity(1000, 95)))
    })
}

async fn save_hashid_table() {
    let table = hashid_table().lock().await;
    let serialized = table.raw_bytes();

    std::fs::write("hashids.db", serialized).unwrap();
}

/****************************************
* Helpers
****************************************/
fn generate_hashid() -> String {
    let mut thread_rng = rand::thread_rng();


    (0..10)
        .map(|_| thread_rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .collect::<String>()
}

/****************************************
* Routes
****************************************/
#[post("/upload")]
async fn upload(
    file: web::Bytes
) -> Result<impl Responder, UploadError> {
    let filename = generate_hashid();

    if let BodySize::Sized(size) = file.size() {
        if size > 25_000_000 {
            error!("File too big: {}", size);
            return Err(UploadError::FileTooBig);
        }
    } else {
        error!("File has invalid size");
        return Err(UploadError::InternalServerError);
    }

    let file_type = infer::get(&file)
        .ok_or(UploadError::UnknownFileType)?;

    if file_type.matcher_type() != MatcherType::Image {
        error!("File is not an image");
        return Err(UploadError::NotAnImage);
    }

    let extension = file_type.extension();

    let hashid_table = hashid_table();
    let mut table = hashid_table.lock().await;

    let mut hasher = Sha256::new();
    hasher.update(&file);
    let hash: [u8; 32] = hasher.finalize().into();

    if let Some(k) = table.get(&hash) {
        info!("File already exists: {}", k);

        return Ok(HttpResponse::Ok().body(
            format!("{}.{}", k, extension)
        ));
    }

    table.insert(&hash, &filename.clone());

    tokio::spawn(save_hashid_table());

    let filename = format!("{}.{}", filename, extension);

    info!("Uploading file: {}", filename);

    tokio::fs::write(
        format!("/var/www/middleclick.wtf/images/{}", filename).as_str(),
        file
    ).await.unwrap();

    info!("Uploaded file: {}", filename);

    Ok(HttpResponse::Ok().body(filename))
}

#[get("/images/{filename}")]
async fn get_image(
    filename: web::Path<String>
) -> Result<impl Responder, DownloadError> {
    let filename = filename.into_inner();

    info!("Downloading file: {}", filename);

    let file = tokio::fs::read(
        format!("/var/www/middleclick.wtf/images/{}", filename).as_str()
    ).await;

    if let Ok(file) = file {
        info!("Downloaded file: {}", filename);
        Ok(HttpResponse::Ok().body(file))
    } else {
        error!("File not found: {}", filename);
        Err(DownloadError::NotFound)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(upload)
            .service(get_image)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::PayloadConfig::new(25_000_000))
    }).bind("0.0.0.0:8080")?
        .run()
        .await
}

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Not an image")]
    NotAnImage,
    #[error("File is too big")]
    FileTooBig,
    #[error("Internal server error")]
    InternalServerError
}

impl ResponseError for UploadError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            UploadError::UnknownFileType => actix_web::http::StatusCode::BAD_REQUEST,
            UploadError::NotAnImage => actix_web::http::StatusCode::BAD_REQUEST,
            UploadError::FileTooBig => actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,
            UploadError::InternalServerError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("File not found")]
    NotFound,
    #[error("Internal server error")]
    InternalServerError
}

impl ResponseError for DownloadError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            DownloadError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            DownloadError::InternalServerError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/****************************************
* Data tables
****************************************/
struct HashIds;

impl Config for HashIds {
    type Key = [u8; 32];
    type Value = String;

    type EncodedKey = [u8; 32];
    type EncodedValue = [u8; 10];

    type H = FxHashFn;

    #[inline]
    fn encode_key(key: &Self::Key) -> Self::EncodedKey {
        *key
    }

    #[inline]
    fn encode_value(value: &Self::Value) -> Self::EncodedValue {
        if value.len() != 10 {
            panic!("Value must be 10 bytes long");
        }

        if !value.is_ascii() {
            panic!("Value must be ascii");
        }

        let mut encoded = [0; 10];
        encoded.copy_from_slice(value.as_bytes());
        encoded
    }

    #[inline]
    fn decode_key(key: &Self::EncodedKey) -> Self::Key {
        *key
    }

    #[inline]
    fn decode_value(value: &Self::EncodedValue) -> Self::Value {
        String::from_utf8_lossy(value).to_string()
    }
}
