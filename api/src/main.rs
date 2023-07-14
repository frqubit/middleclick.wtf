use infer::MatcherType;
use thiserror::Error;
use rand::prelude::*;
use log::{info, error};
use actix_web::{
    web, App, HttpResponse,
    HttpServer, Responder, post,
    middleware::Logger, ResponseError,
    body::{MessageBody, BodySize}
};


#[post("/upload")]
async fn upload(
    file: web::Bytes
) -> Result<impl Responder, UploadError> {
    let mut thread_rng = rand::thread_rng();
    let filename = (0..10)
        .map(|_| thread_rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .collect::<String>();

    if let BodySize::Sized(size) = file.size() {
        if size > 25_000_000 {
            info!("File too big: {}", size);
            return Err(UploadError::FileTooBig);
        }
    } else {
        return Err(UploadError::InternalServerError);
    }

    let file_type = infer::get(&file)
        .ok_or(UploadError::UnknownFileType)?;

    if file_type.matcher_type() != MatcherType::Image {
        return Err(UploadError::NotAnImage);
    }

    let extension = file_type.extension();

    let filename = format!("{}.{}", filename, extension);

    tokio::fs::write(
        format!("/var/www/middleclick.wtf/images/{}", filename).as_str(),
        file
    ).await.unwrap();

    Ok(HttpResponse::Ok().body(filename))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(upload)
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
