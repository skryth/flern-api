# Flern

Flern is the backend component of the Flutter Learn platform, available at [flern.website](https://flern.website/). This repository contains a Rust-based API server built with Axum.

## Overview

This backend provides RESTful API endpoints for the FLERN project, handling user authentication, data persistence, and content management. The application is built with Rust for performance and reliability, leveraging modern async runtime capabilities.

## Prerequisites

- Rust toolchain (2024 edition)
- PostgreSQL database
- Git

## Installation

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/skryth/flern-api.git
cd flern-api
```

## Configuration

Create a `config.toml` file in the project root with the following structure:

```toml
[host]
bindto = "127.0.0.1:5000"  # Use 0.0.0.0:5000 to bind to all interfaces

[app]
jwt = "your-secure-jwt-secret"
database_uri = "postgres://USERNAME:PASSWORD@HOST:PORT/DATABASE"
host_url = "http://your-domain.com/"
docs = true  # Set to false to disable API documentation endpoint
```

### Configuration Parameters

- `bindto`: Server bind address and port
- `jwt`: Secret key for JWT token signing (ensure this is cryptographically secure in production)
- `database_uri`: PostgreSQL connection string
- `host_url`: Base URL for serving uploaded content from the `uploads/` directory
- `docs`: Enable/disable Swagger documentation at `/api/v1/docs`

## Running the Application

### From Source

```bash
cargo run --release
```

### Using Pre-built Binary

```bash
cargo build --release
./target/release/flern
```

Ensure `config.toml` is in the same directory as the executable.

### Docker Deployment

The application is available as a container image on GitHub Container Registry:

1. Pull the image from GHCR
2. Create `config.toml` on the host system
3. Run the container with volume binding:
   ```bash
   docker run -v /path/to/config.toml:/app/config.toml flern
   ```

## Environment Variables

Flern reads `.env` files from the current working directory. Supported variables:

```bash
RUST_LOG=flern=trace  # Logging levels: trace, debug, info, warn, error
TEST_DATABASE_ADMIN_URL="postgres://postgres:PASSWORD@HOST/postgres"
```

Refer to the [tracing documentation](https://docs.rs/tracing) for advanced logging configuration.

## Testing

The test suite requires a PostgreSQL database with administrative privileges. Tests create temporary databases to ensure isolation.

Set the required environment variable:

```bash
export TEST_DATABASE_ADMIN_URL="postgres://postgres:PASSWORD@HOST/postgres"
```

Run tests with:

```bash
cargo test
```

> [!NOTE]
> The `TEST_DATABASE_ADMIN_URL` must connect as a superuser (typically `postgres`) to allow database creation during test execution.

## Technology Stack

- **Runtime**: Tokio async runtime
- **Web Framework**: Axum
- **Database**: PostgreSQL via SQLx
- **Authentication**: JWT with Argon2 password hashing
- **Documentation**: OpenAPI 3.0 via utoipa
- **Serialization**: Serde (JSON, TOML)

## Contributing

Contributions are welcome and encouraged! Feel free to submit pull requests, report issues, or suggest improvements. All contributions should follow standard Git workflow practices:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

This project is licensed under the MIT License. See [LICENSE.TXT](LICENSE.TXT) for details.

---

For questions or additional information, please refer to the project repository at [github.com/skryth/flern-api](https://github.com/skryth/flern-api).
