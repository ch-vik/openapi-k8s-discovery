# Local Testing Environment

This directory contains a Docker Compose setup for testing the OpenAPI documentation server locally.

## What's Included

- **Mock API Services**: Three simple nginx services that serve OpenAPI specs:
  - `user-service` (port 8081)
  - `product-service` (port 8082)
  - `order-service` (port 8083)

- **Discovery Configuration**: A `discovery.json` file that lists all mock APIs

- **Documentation Server**: The OpenAPI doc server with both Scalar and Redoc frontends enabled

## Prerequisites

- Docker and Docker Compose installed
- The project built (or use Docker to build it)

## Quick Start

1. **Start all services:**
   ```bash
   cd test/local
   docker-compose up -d
   ```

2. **Access the documentation:**
   - Scalar UI (default): http://localhost:8080/
   - Scalar UI (direct): http://localhost:8080/scalar
   - Redoc UI: http://localhost:8080/redoc
   - Health check: http://localhost:8080/health

3. **View logs:**
   ```bash
   docker-compose logs -f openapi-doc-server
   ```

4. **Stop services:**
   ```bash
   docker-compose down
   ```

## Configuration

You can customize the server by modifying environment variables in `docker-compose.yml`. All configuration is done through environment variables - no config files needed!

### Environment Variables Reference

#### Frontend Selection

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLED_FRONTENDS` | `scalar` | Comma-separated list of frontends to enable. Options: `scalar`, `redoc`, or `scalar,redoc` |
| `DEFAULT_FRONTEND` | First enabled frontend | Which frontend to show at `/`. Must be one of the enabled frontends (e.g., `scalar` or `redoc`) |

**Examples:**
```bash
# Enable only Scalar
ENABLED_FRONTENDS=scalar
DEFAULT_FRONTEND=scalar

# Enable both, default to Redoc
ENABLED_FRONTENDS=scalar,redoc
DEFAULT_FRONTEND=redoc
```

#### Scalar Frontend Options

| Variable | Default | Description |
|----------|---------|-------------|
| `SCALAR_THEME` | `purple` | Theme name. Options: `default`, `alternate`, `moon`, `purple`, `solarized`, `bluePlanet`, `saturn`, `kepler`, `mars`, `deepSpace`, `laserwave`, `none` |
| `SCALAR_LAYOUT` | `modern` | Layout style. Options: `modern` or `classic` |
| `SCALAR_DARK_MODE` | `false` | Enable dark mode. Set to `true` or `false` |
| `SCALAR_SHOW_SIDEBAR` | `true` | Show the sidebar navigation. Set to `true` or `false` |
| `SCALAR_EXPAND_ALL_RESPONSES` | `true` | Expand all response sections by default. Set to `true` or `false` |
| `SCALAR_EXPAND_ALL_MODEL_SECTIONS` | `false` | Expand all model sections by default. Set to `true` or `false` |
| `SCALAR_HIDE_DOWNLOAD_BUTTON` | `false` | Hide the download button. Set to `true` or `false` |

**Example Scalar Configuration:**
```bash
SCALAR_THEME=bluePlanet
SCALAR_LAYOUT=classic
SCALAR_DARK_MODE=true
SCALAR_SHOW_SIDEBAR=true
```

#### Redoc Frontend Options

| Variable | Default | Description |
|----------|---------|-------------|
| `REDOC_EXPAND_RESPONSES` | `200,201,400,401,403,404` | Comma-separated list of HTTP response codes to expand by default (e.g., `200,201,400,500`) |
| `REDOC_REQUIRED_PROPS_FIRST` | `true` | Show required properties first in schema definitions. Set to `true` or `false` |
| `REDOC_SHOW_API_SELECTOR` | `true` | Show the API selector dropdown when multiple APIs are available. Set to `true` or `false` |

**Example Redoc Configuration:**
```bash
REDOC_EXPAND_RESPONSES=200,201,400,500
REDOC_REQUIRED_PROPS_FIRST=true
REDOC_SHOW_API_SELECTOR=true
```

#### Path Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CACHE_DIR` | `/tmp/openapi-cache` | Directory where API specs are cached as JSON files |
| `DISCOVERY_PATH` | `/etc/config/discovery.json` | Path to the discovery.json file containing API metadata |

**Note**: These paths are relative to the container's filesystem. In Docker Compose, mount volumes accordingly.

#### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level. Options: `trace`, `debug`, `info`, `warn`, `error` |

**Example:**
```bash
RUST_LOG=debug  # For detailed debugging information
```

## Features

### Multi-Frontend Support
- **Scalar**: Modern, feature-rich API documentation interface with themes, layouts, and extensive customization
- **Redoc**: Clean, simple API documentation with customizable display options
- **Flexible Routing**: Access frontends via `/scalar`, `/redoc`, or `/` (default frontend)
- **Feature Flags**: Compile only the frontends you need using Cargo features to reduce binary size

### File-Based Caching
- API specs are cached to disk for persistence across restarts
- Each API is stored as:
  - `{api-name}.json` - The OpenAPI specification
  - `{api-name}.meta.json` - API metadata (name, description, availability, etc.)
- Cache survives container restarts (when using Docker volumes)

### Configuration-Based
- All settings via environment variables (no config files to manage)
- Frontend-specific options are clearly namespaced (`SCALAR_*`, `REDOC_*`)
- Sensible defaults for all options

## Testing Different Configurations

### Test with only Scalar:
Edit `docker-compose.yml` and set:
```yaml
environment:
  - ENABLED_FRONTENDS=scalar
  - DEFAULT_FRONTEND=scalar
```

Then restart:
```bash
docker-compose up -d --build
```

### Test with only Redoc:
Edit `docker-compose.yml` and set:
```yaml
environment:
  - ENABLED_FRONTENDS=redoc
  - DEFAULT_FRONTEND=redoc
```

### Custom Theme Example:
```yaml
environment:
  - ENABLED_FRONTENDS=scalar,redoc
  - DEFAULT_FRONTEND=scalar
  - SCALAR_THEME=deepSpace
  - SCALAR_DARK_MODE=true
  - SCALAR_LAYOUT=classic
```

## File Structure

```
test/local/
├── README.md                  # This file
├── docker-compose.yml         # Docker Compose configuration
├── config/
│   └── discovery.json         # API discovery configuration
└── specs/
    ├── user-service.json      # User service OpenAPI spec
    ├── product-service.json   # Product service OpenAPI spec
    └── order-service.json     # Order service OpenAPI spec
```

## URL Configuration

The `discovery.json` file uses Docker service names for URLs (e.g., `http://user-service:8080/openapi.json`). These work within the Docker network.

If you want to test from outside Docker or run locally without Docker Compose, you can update the URLs in `discovery.json` to use `localhost`:

```json
{
  "apis": [
    {
      "url": "http://localhost:8081/openapi.json"
    },
    {
      "url": "http://localhost:8082/openapi.json"
    },
    {
      "url": "http://localhost:8083/openapi.json"
    }
  ]
}
```

**Note**: When running in Docker Compose, use service names (default). When testing locally without Docker, use `localhost` with the mapped ports.

## Troubleshooting

1. **Server not starting**: Check logs with `docker-compose logs openapi-doc-server`

2. **APIs not showing**: 
   - Verify discovery.json is correctly formatted
   - Check that mock API services are running: `docker-compose ps`
   - Verify API endpoints are accessible: `curl http://localhost:8081/openapi.json`

3. **Frontend not loading**: 
   - Check that the frontend is enabled in `ENABLED_FRONTENDS`
   - Verify the feature is compiled (check Dockerfile build args)

4. **Clear cache**: Remove the volume and restart:
   ```bash
   docker-compose down -v
   docker-compose up -d
   ```

## Building Locally

If you want to build the server locally instead of using Docker:

### Build Options

**With all frontends (default):**
```bash
cd ../..
cargo build --bin openapi-doc-server --features scalar,redoc --release
```

**With only Scalar:**
```bash
cargo build --bin openapi-doc-server --features scalar --release
```

**With only Redoc:**
```bash
cargo build --bin openapi-doc-server --no-default-features --features redoc --release
```

### Running Locally

1. **Start mock API services in Docker:**
   ```bash
   cd test/local
   docker-compose up -d user-service product-service order-service
   ```

2. **Set environment variables:**
   ```bash
   export CACHE_DIR=./test/local/cache
   export DISCOVERY_PATH=./test/local/config/discovery.json
   export ENABLED_FRONTENDS=scalar,redoc
   export DEFAULT_FRONTEND=scalar
   export SCALAR_THEME=purple
   export SCALAR_LAYOUT=modern
   export REDOC_EXPAND_RESPONSES=200,201,400,401,403,404
   export RUST_LOG=info
   ```

3. **Create cache directory:**
   ```bash
   mkdir -p ./test/local/cache
   ```

4. **Run the server:**
   ```bash
   cd ../..
   ./target/release/openapi-doc-server
   ```

### Access Points

When running locally, the server binds to `0.0.0.0:8080`, so you can access:
- Scalar UI: http://localhost:8080/
- Scalar UI (direct): http://localhost:8080/scalar
- Redoc UI: http://localhost:8080/redoc
- Health check: http://localhost:8080/health
- API specs: http://localhost:8080/api/{api-name} or http://localhost:8080/specs/{api-name}

## Architecture

The server works in the following way:

1. **Discovery**: Reads `discovery.json` from `DISCOVERY_PATH` (default: `/etc/config/discovery.json`)
2. **Fetching**: Periodically fetches OpenAPI specs from the URLs specified in discovery.json
3. **Caching**: Stores specs and metadata in `CACHE_DIR` as JSON files
4. **Serving**: 
   - Serves specs via `/api/{api-name}` and `/specs/{api-name}` endpoints
   - Renders frontend HTML at `/`, `/scalar`, and `/redoc` based on configuration
   - Frontends load specs via the `/specs/` endpoint
5. **Refresh**: Automatically refreshes the cache every 30 seconds

### Endpoints

- `GET /` - Default frontend (configured via `DEFAULT_FRONTEND`)
- `GET /scalar` - Scalar UI (if enabled)
- `GET /redoc` - Redoc UI (if enabled)
- `GET /api/{api_name}` - JSON endpoint for OpenAPI spec (backward compatible)
- `GET /specs/{api_name}` - JSON endpoint for OpenAPI spec (preferred)
- `GET /health` - Health check endpoint

