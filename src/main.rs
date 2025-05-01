use std::io::Cursor;
use axum::{
    
    body::{Body, Bytes},
    extract::{Query, Multipart},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use image::{DynamicImage, ImageFormat};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/new", post(image_process));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}


struct ImageHandling {
    img: DynamicImage,
}

impl ImageHandling {
    fn new(buffer: &[u8]) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory(buffer)?;
        Ok(ImageHandling { img })
    }

    fn blur(&mut self) {
        self.img = self.img.blur(10.0)
    }
}

async fn image_process(
    multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let image_data = extract_image_data(multipart).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let mut img_handler = ImageHandling::new(&image_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    img_handler.blur();
    
    let buffer = convert_to_buffer(img_handler.img, ImageFormat::Jpeg)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(buffer))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}

async fn extract_image_data(mut multipart: Multipart) -> Result<Vec<u8>, String> {


    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        if field.name() == Some("image") {
            let data = field.bytes().await.map_err(|e| e.to_string())?;
            return Ok(data.to_vec());
        }
    }
    
    Err("No image field found in multipart form".to_string())
}

fn convert_to_buffer(image: DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buffer: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut buffer), format)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}