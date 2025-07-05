use axum::{
    body::Bytes,
    extract::Multipart,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use image::{DynamicImage, ImageFormat};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "image_transformer_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/transform", post(transform_image_handler))
        .layer(
            TraceLayer::new_for_http()
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR))
        )
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

/// Handler for the /transform endpoint.
/// Accepts multipart/form-data with two fields:
/// - "image": The image file (PNG or JPG).
/// - "size": An optional string like "800x600".
/// - "quality": An optional float for lossy compression quality (0.0 to 100.0).
async fn transform_image_handler(mut multipart: Multipart) -> Result<Response, AppError> {
    let mut image_data: Option<Bytes> = None;
    let mut size_str: Option<String> = None;
    let mut quality: Option<f32> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "image" => {
                image_data = Some(field.bytes().await?);
            }
            "size" => {
                size_str = Some(field.text().await?);
            }
            "quality" => {
                let quality_str = field.text().await?;
                quality = quality_str.parse::<f32>().ok();
                if let Some(q) = quality {
                    if q < 0.0 || q > 100.0 {
                        return Err(AppError::new(StatusCode::BAD_REQUEST, "Quality must be between 0.0 and 100.0"));
                    }
                }
            }
            _ => { /* Ignore other fields */ }
        }
    }

    let image_bytes = image_data.ok_or_else(|| {
        AppError::new(StatusCode::BAD_REQUEST, "Image data not provided in 'image' field")
    })?;

    let webp_bytes = tokio::task::spawn_blocking(move || {
        process_image(image_bytes, size_str, quality)
    })
    .await??;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/webp")],
        webp_bytes,
    ).into_response())
}

fn process_image(image_bytes: Bytes, size_str: Option<String>, quality: Option<f32>) -> Result<Vec<u8>, AppError> {
    let image_format = image::guess_format(&image_bytes)
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "Could not determine image format"))?;

    if ![ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::WebP].contains(&image_format) {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "Input image must be PNG, JPG, or WebP"));
    }

    let mut img = image::load_from_memory(&image_bytes)
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to decode image: {}", e)))?;

    if let Some(s) = size_str {
        let (width, height) = parse_size(&s)?;
        img = img.resize(width, height, image::imageops::FilterType::Lanczos3);
    }

    encode_to_webp(img, quality.unwrap_or(100.0))
}

fn encode_lossy_webp(img: DynamicImage, quality: f32) -> Result<Vec<u8>, AppError> {
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    
    let encoder = webp::Encoder::new(&*img, webp::PixelLayout::Rgba, width, height);
    let encoded = encoder.encode(quality);
    
    if encoded.is_empty() {
        return Err(AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR, 
            "Failed to encode image to WebP format"
        ));
    }
    
    Ok(encoded.to_vec())
}

fn encode_to_webp(img: DynamicImage, quality: f32) -> Result<Vec<u8>, AppError> {
    encode_lossy_webp(img, quality)
}

fn parse_size(size_str: &str) -> Result<(u32, u32), AppError> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "Invalid size format. Use 'WIDTHxHEIGHT'"));
    }
    let width = parts[0].parse::<u32>()
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "Invalid width value"))?;
    let height = parts[1].parse::<u32>()
        .map_err(|_| AppError::new(StatusCode::BAD_REQUEST, "Invalid height value"))?;
    Ok((width, height))
}

struct AppError {
    status_code: StatusCode,
    message: String,
}

impl AppError {
    fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self { status_code, message: message.into() }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status_code.is_server_error() {
            tracing::error!(status = %self.status_code, error = %self.message);
        } else if self.status_code.is_client_error() {
            tracing::warn!(status = %self.status_code, error = %self.message);
        }

        (self.status_code, self.message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}
