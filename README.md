# Image Transformer API

A high-performance REST API for image transformation and format conversion, built with Rust. This service allows you to resize images and convert them to WebP format for optimal web delivery.

## Features

- **Image Resizing**: Resize images to specified dimensions using high-quality Lanczos3 filtering
- **Format Conversion**: Convert PNG, JPEG, and WebP images to optimized WebP format
- **Quality Control**: Adjust compression quality for lossy WebP encoding (0.0-100.0)
- **High Performance**: Built with Rust for maximum performance and memory safety
- **Large File Support**: Handles files up to 100MB
- **CORS Enabled**: Ready for cross-origin requests from web applications

## Technology Stack

- **Language**: Rust 2024 Edition
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) - Fast, async web framework
- **Image Processing**: [image-rs](https://github.com/image-rs/image) - Pure Rust image processing library
- **WebP Encoding**: [webp](https://crates.io/crates/webp) crate for WebP format support
- **Async Runtime**: [Tokio](https://tokio.rs) for asynchronous operations
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) for structured logging
- **Containerization**: Docker with multi-stage builds for optimized images

## Quick Start with Docker

The easiest way to run the Image Transformer API is using the pre-built Docker image published to GitHub Container Registry.

### Using Docker Run

```bash
docker run -p 3000:3000 ghcr.io/loupeznik/image-transformer-api:master
```

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
services:
  image-transformer-api:
    image: ghcr.io/loupeznik/image-transformer-api:master
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
```

Then run:

```bash
docker-compose up -d
```

## API Endpoints

### Health Check

Check if the service is running:

```http
GET /healthz
```

**Response**: `200 OK` with body `"OK"`

### Image Transformation

Transform and resize images:

```http
POST /transform
Content-Type: multipart/form-data
```

**Form Parameters**:
- `image` (required): The image file (PNG, JPEG, or WebP)
- `size` (optional): Target dimensions in format `WIDTHxHEIGHT` (e.g., `800x600`)
- `quality` (optional): WebP quality for lossy compression (0.0-100.0, default: 100.0)

**Response**: WebP image with `Content-Type: image/webp`

## API Usage Examples

### Basic Image Conversion

Convert an image to WebP format:

```bash
curl -X POST http://localhost:3000/transform \
  -F "image=@input.jpg" \
  -o output.webp
```

### Resize and Convert

Resize an image to 800x600 pixels and convert to WebP:

```bash
curl -X POST http://localhost:3000/transform \
  -F "image=@input.png" \
  -F "size=800x600" \
  -o resized_output.webp
```

### Resize with Quality Control

Resize and set WebP quality to 85% for smaller file size:

```bash
curl -X POST http://localhost:3000/transform \
  -F "image=@input.jpg" \
  -F "size=1200x800" \
  -F "quality=85" \
  -o compressed_output.webp
```

### Using with wget

```bash
wget --post-file=input.jpg \
  --header="Content-Type: multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW" \
  http://localhost:3000/transform \
  -O output.webp
```

## Local Development Environment

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **System Dependencies**: 
  - On Ubuntu/Debian: `apt-get install clang libwebp-dev pkg-config`
  - On macOS: `brew install webp`
  - On Windows: Install Visual Studio Build Tools

### Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Loupeznik/image-transformer-api.git
   cd image-transformer-api
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Run the development server**:
   ```bash
   cargo run
   ```

   The API will be available at `http://localhost:3000`

4. **Run in development mode with auto-reload**:
   ```bash
   cargo install cargo-watch
   cargo watch -x run
   ```

### Development Commands

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Test**: `cargo test`
- **Check**: `cargo check`
- **Format**: `cargo fmt`
- **Lint**: `cargo clippy`

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)
- `PORT`: Override the default port (3000)

Example:
```bash
RUST_LOG=debug cargo run
```

## Building Docker Image Locally

If you want to build the Docker image yourself:

```bash
# Build the image
docker build -t image-transformer-api .

# Run the locally built image
docker run -p 3000:3000 image-transformer-api
```

## Performance Considerations

- The API uses Tokio for async processing to handle multiple requests concurrently
- Large images are processed in blocking threads to avoid blocking the async runtime
- Memory usage scales with image size; consider container memory limits for production
- WebP encoding provides excellent compression ratios while maintaining quality

## Error Handling

The API returns appropriate HTTP status codes:

- `200 OK`: Successful transformation
- `400 Bad Request`: Invalid input (missing image, invalid size format, invalid quality)
- `500 Internal Server Error`: Processing errors

Error responses include descriptive messages in the response body.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Support

For questions and support, please open an issue on the GitHub repository.