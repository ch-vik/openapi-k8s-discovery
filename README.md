# OpenAPI K8s Operator

A production-ready Kubernetes operator written in Rust that automatically discovers services with OpenAPI documentation and provides a centralized Scalar UI interface for browsing all discovered APIs.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-1.32+-blue.svg)](https://kubernetes.io/)

## Features

- **Automatic Discovery**: Watches for Kubernetes services with API documentation annotations
- **Centralized UI**: Provides a single Scalar UI interface for all discovered APIs with dropdown selector
- **Health Monitoring**: Continuously monitors API availability and updates status
- **Production Ready**: Built with proper error handling, reconciliation, and RBAC
- **Standard Annotations**: Uses standard Kubernetes annotation patterns
- **Modern Rust**: Built with Rust 2024 edition and latest stable dependencies
- **Workspace Architecture**: Organized as a Cargo workspace with shared components
- **Scalar UI**: Beautiful, modern API documentation interface with first-class OpenAPI support

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   API Services  │    │  Rust Operator   │    │  Scalar UI      │
│                 │    │                  │    │                 │
│ - Service A     │───▶│ - Watches        │───▶│ - Centralized   │
│   (annotated)   │    │   Services       │    │   Interface     │
│ - Service B     │    │ - Updates        │    │ - Multi-API     │
│   (annotated)   │    │   ConfigMap      │    │   Support       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites

- Kubernetes cluster (1.28+)
- Helm 3.x
- kubectl configured to access your cluster

### Installation

#### Option 1: Helm Repository (Recommended)

```bash
# Add the Helm repository
helm repo add openapi-k8s-discovery https://ch-vik.github.io/openapi-k8s-discovery/
helm repo update

# Install the operator with OpenAPI server
helm install openapi-operator openapi-k8s-discovery/openapi-k8s-discovery \
  --set openapiServer.enabled=true
```

#### Option 2: Direct Helm Chart

```bash
# Clone the repository
git clone https://github.com/ch-vik/openapi-k8s-discovery.git
cd openapi-k8s-discovery

# Install with Helm
helm install openapi-operator ./helm/openapi-k8s-discovery \
  --set openapiServer.enabled=true
```

### Access the UI

```bash
# Port forward to access the Scalar UI
kubectl port-forward service/openapi-server 3000:80

# Open in browser
open http://localhost:3000
```

## Usage

### Annotate Your Services

Add the following annotations to your services that expose OpenAPI documentation:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-api-service
  annotations:
    api-doc.io/enabled: "true"
    api-doc.io/name: "My API"
    api-doc.io/description: "A sample API service"
    api-doc.io/path: "/swagger/openapi.yml"
spec:
  ports:
    - port: 80
      targetPort: 80
  selector:
    app: my-api
```

### Configuration Options

#### Basic Configuration

```bash
# Install with custom namespace
helm install openapi-operator openapi-k8s-discovery/openapi-k8s-discovery \
  --set namespace.create=true \
  --set namespace.name=openapi-system \
  --set openapiServer.enabled=true
```

#### Advanced Configuration

```bash
# Cluster-wide monitoring with custom settings
helm install openapi-operator openapi-k8s-discovery/openapi-k8s-discovery \
  --set operator.config.watchNamespaces=all \
  --set operator.serviceMonitor.enabled=true \
  --set openapiServer.enabled=true \
  --set openapiServer.ingress.enabled=true \
  --set openapiServer.ingress.hosts[0].host=api-docs.example.com
```

#### Environment Variables

| Variable              | Default               | Description                                                                 |
| --------------------- | --------------------- | --------------------------------------------------------------------------- |
| `WATCH_NAMESPACES`    | `""`                  | Namespaces to watch (`""` = current, `"all"` = all, `"ns1,ns2"` = specific) |
| `DISCOVERY_NAMESPACE` | `"default"`           | Namespace where ConfigMap will be created                                   |
| `DISCOVERY_CONFIGMAP` | `"openapi-discovery"` | Name of the discovery ConfigMap                                             |
| `RUST_LOG`            | `"info"`              | Logging level                                                               |

### Annotations Reference

| Annotation               | Required | Default                  | Description                                           |
| ------------------------ | -------- | ------------------------ | ----------------------------------------------------- |
| `api-doc.io/enabled`     | Yes      | -                        | Set to `"true"` to enable API documentation discovery |
| `api-doc.io/name`        | No       | `"{service-name} API"`   | Display name for the API in the UI                    |
| `api-doc.io/description` | No       | -                        | Description of the API                                |
| `api-doc.io/path`        | No       | `"/swagger/openapi.yml"` | Path to the OpenAPI specification                     |

## Examples

### Basic Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: user-service
  annotations:
    api-doc.io/enabled: "true"
spec:
  ports:
    - port: 80
      targetPort: 80
  selector:
    app: user-service
```

### Advanced Service Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: payment-service
  annotations:
    api-doc.io/enabled: "true"
    api-doc.io/name: "Payment API"
    api-doc.io/description: "Handles payment processing and billing"
    api-doc.io/path: "/api/v1/openapi.json"
spec:
  ports:
    - port: 80
      targetPort: 80
  selector:
    app: payment-service
```

### Test Services

Deploy example services for testing:

```bash
# Deploy test services
kubectl apply -f https://raw.githubusercontent.com/ch-vik/openapi-k8s-discovery/master/examples/test-api.yaml
kubectl apply -f https://raw.githubusercontent.com/ch-vik/openapi-k8s-discovery/master/examples/simple-service.yaml
```

## Workspace Structure

This project is organized as a Cargo workspace with three master components:

```
openapi-k8s-discovery/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── openapi-common/           # Shared library
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── openapi-k8s-discovery/     # Kubernetes operator
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── master.rs
│   │       └── error.rs
│   └── openapi-doc-server/       # Scalar UI server
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── src/master.rs
├── examples/                     # Example service definitions
├── helm/                        # Helm chart for deployment
└── README.md                    # This documentation
```

## Development

### Prerequisites

- Rust 1.85+
- Docker
- kubectl
- Helm 3.x

### Building from Source

```bash
# Clone the repository
git clone https://github.com/ch-vik/openapi-k8s-discovery.git
cd openapi-k8s-discovery

# Build all crates
cargo build --workspace

# Build with release optimizations
cargo build --release --workspace

# Run tests
cargo test --workspace
```

### Building Docker Images

```bash
# Build operator image
docker build -f crates/openapi-k8s-discovery/Dockerfile -t ghcr.io/ch-vik/openapi-k8s-discovery:latest .

# Build server image
docker build -f crates/openapi-doc-server/Dockerfile -t ghcr.io/ch-vik/openapi-doc-server:latest .
```

### Running Locally

```bash
# Set up kubectl context
kubectl config use-context your-cluster

# Run the operator
cargo run -p openapi-k8s-discovery

# Run the server (for testing)
cargo run -p openapi-doc-server
```

## Troubleshooting

### Check Operator Status

```bash
# Check if the operator is running
kubectl get pods -l app.kubernetes.io/name=openapi-k8s-discovery

# Check operator logs
kubectl logs -l app.kubernetes.io/name=openapi-k8s-discovery
```

### Check Discovery ConfigMap

```bash
# View the discovery configuration
kubectl get configmap openapi-discovery -o yaml

# Check the discovery JSON
kubectl get configmap openapi-discovery -o jsonpath='{.data.discovery\.json}' | jq .
```

### Check Scalar UI Server

```bash
# Check server status
kubectl get pods -l app.kubernetes.io/component=openapi-server

# Check server logs
kubectl logs -l app.kubernetes.io/component=openapi-server

# Port forward for local testing
kubectl port-forward service/openapi-server 3000:80
```

### Common Issues

1. **ConfigMap not found**: The operator initializes the ConfigMap at startup. If it's missing, check operator logs for initialization errors.

2. **Services not discovered**: Ensure services have the `api-doc.io/enabled: "true"` annotation and are in watched namespaces.

3. **OpenAPI specs not loading**: Check that the service is accessible and the path annotation is correct.

4. **Port forwarding issues**: Ensure the service is running and the port mapping is correct.

## Security Considerations

- The operator uses minimal RBAC permissions
- Only watches services and manages the discovery ConfigMap
- No access to sensitive resources or data
- All API calls are made within the cluster network
- Scalar UI server fetches specs internally to avoid CORS issues

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 🐛 [Issue Tracker](https://github.com/ch-vik/openapi-k8s-discovery/issues)
- 📧 [Email Support](mailto:kevin.ceresa@swisstilab.ch)

