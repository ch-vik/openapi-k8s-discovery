# OpenAPI K8s Operator Workspace

This project uses a Cargo workspace structure to organize the codebase into separate, reusable components.

## Workspace Structure

```
openapi-k8s-operator/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── openapi-common/           # Shared types and utilities
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       └── lib.rs
│   ├── openapi-k8s-operator/     # Kubernetes operator
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       └── error.rs
│   └── openapi-doc-server/       # Scalar UI server
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── src/
│           └── main.rs
├── examples/                     # Example service definitions
├── helm/                        # Helm chart
└── README.md                    # Main project documentation
```

## Crates

### `openapi-common`
Shared library containing:
- Common data structures (`ApiDocEntry`, `DiscoveryConfig`)
- Utility functions for OpenAPI spec parsing
- Namespace handling utilities
- Constants and configuration

### `openapi-k8s-operator`
Kubernetes operator that:
- Watches for services with API documentation annotations
- Fetches OpenAPI specs from discovered services
- Updates a ConfigMap with aggregated API documentation

### `openapi-doc-server`
Web server that:
- Serves a Scalar UI for API documentation
- Reads API specs from the ConfigMap created by the operator
- Provides a centralized interface for viewing multiple APIs

## Building

### Build all crates
```bash
cargo build
```

### Build specific crate
```bash
cargo build -p openapi-k8s-operator
cargo build -p openapi-doc-server
```

### Build with release optimizations
```bash
cargo build --release
```

## Docker Images

Each crate has its own Dockerfile for building production images:

### Operator
```bash
docker build -f crates/openapi-k8s-operator/Dockerfile -t openapi-k8s-operator:latest .
```

### Server
```bash
docker build -f crates/openapi-doc-server/Dockerfile -t openapi-doc-server:latest .
```

### Building Both Images
```bash
# Build operator image
docker build -f crates/openapi-k8s-operator/Dockerfile -t openapi-k8s-operator:latest .

# Build server image  
docker build -f crates/openapi-doc-server/Dockerfile -t openapi-doc-server:latest .
```

## Development

### Adding new shared functionality
1. Add new types or functions to `crates/openapi-common/src/lib.rs`
2. Export them in the `lib.rs` file
3. Import and use them in the other crates

### Adding dependencies
- Add workspace dependencies to the root `Cargo.toml` under `[workspace.dependencies]`
- Reference them in individual crate `Cargo.toml` files using `{ workspace = true }`

## Benefits of Workspace Structure

1. **Code Reuse**: Shared types and utilities prevent duplication
2. **Separation of Concerns**: Each component has a clear responsibility
3. **Independent Development**: Each crate can be developed and tested independently
4. **Easier Maintenance**: Changes to shared code are automatically available to all crates
5. **Better Organization**: Clear structure makes the codebase easier to navigate
