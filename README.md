# OpenAPI K8s Operator

A production-ready Kubernetes operator written in Rust that automatically discovers services with OpenAPI documentation and provides a centralized Swagger UI interface.

## Features

- **Automatic Discovery**: Watches for Kubernetes services with API documentation annotations
- **Centralized UI**: Provides a single Swagger UI interface for all discovered APIs
- **Health Monitoring**: Continuously monitors API availability and updates status
- **Production Ready**: Built with proper error handling, reconciliation, and RBAC
- **Standard Annotations**: Uses standard Kubernetes annotation patterns
- **Modern Rust**: Built with Rust 2024 edition and latest stable dependencies
- **Distroless Security**: Uses distroless base image for minimal attack surface

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   API Services  │    │  Rust Operator   │    │  Swagger UI     │
│                 │    │                 │    │                 │
│ - Service A     │───▶│ - Watches        │───▶│ - Centralized   │
│   (annotated)   │    │   Services       │    │   Interface     │
│ - Service B     │    │ - Updates        │    │ - Multi-API     │
│   (annotated)   │    │   ConfigMap      │    │   Support       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Quick Start

### 1. Deploy the Operator

```bash
# Apply RBAC permissions
kubectl apply -f manifests/rbac.yaml

# Deploy the operator
kubectl apply -f manifests/deployment.yaml

# Deploy the centralized Swagger UI
kubectl apply -f manifests/swagger-ui.yaml
```

### 2. Annotate Your Services

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
    - port: 8080
  selector:
    app: my-api
```

### 3. Access the Centralized UI

Once deployed, access the centralized Swagger UI at:
- `http://localhost:8080/swagger-ui/` (via port-forward)
- Or configure an Ingress for external access

## Helm Chart Deployment (Recommended)

For production deployments, use the included Helm chart which provides comprehensive configuration options:

### Basic Installation

```bash
# Install with default settings
helm install openapi-k8s-operator ./helm/openapi-k8s-operator
```

### Advanced Configuration

```bash
# With OpenAPI server and ingress
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set openapiServer.enabled=true \
  --set openapiServer.ingress.enabled=true \
  --set openapiServer.ingress.hosts[0].host=openapi.example.com

# Cluster-wide monitoring
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set operator.config.watchNamespaces=all \
  --set operator.serviceMonitor.enabled=true

# Custom namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set namespace.create=true \
  --set namespace.name=openapi-system
```

**Key Features:**
- Smart RBAC (Role vs ClusterRole based on namespace configuration)
- StatefulSet support for single-instance operators
- NetworkPolicy for security
- Optional OpenAPI server deployment
- Prometheus monitoring integration
- Comprehensive customization options

See [HELM.md](HELM.md) for detailed chart documentation.

## Annotations

The operator recognizes the following standard Kubernetes annotations:

| Annotation | Required | Default | Description |
|------------|----------|---------|-------------|
| `api-doc.io/enabled` | Yes | - | Set to `"true"` to enable API documentation discovery |
| `api-doc.io/name` | No | `"{service-name} API"` | Display name for the API in the UI |
| `api-doc.io/description` | No | - | Description of the API |
| `api-doc.io/path` | No | `"/swagger/openapi.yml"` | Path to the OpenAPI specification |

## Configuration

### Environment Variables

The operator supports the following environment variables:

- `RUST_LOG`: Logging level (default: `info`)
- `RUST_BACKTRACE`: Enable backtraces for debugging
- `WATCH_NAMESPACES`: Namespace configuration (`"all"` = all namespaces, empty = current namespace, `"ns1,ns2"` = specific namespaces)
- `DISCOVERY_NAMESPACE`: Namespace where the discovery ConfigMap will be created (default: `default`)
- `DISCOVERY_CONFIGMAP`: Name of the discovery ConfigMap (default: `openapi-discovery`)

### Namespace Configuration

The operator can be configured to watch namespaces in three ways:

**Watch Current Namespace Only (Default):**
```yaml
env:
  - name: WATCH_NAMESPACES
    value: ""  # Empty = current namespace only
```

**Watch All Namespaces (Requires Cluster RBAC):**
```yaml
env:
  - name: WATCH_NAMESPACES
    value: "all"  # "all" = all namespaces
```

**Watch Specific Namespaces:**
```yaml
env:
  - name: WATCH_NAMESPACES
    value: "default,production,staging"
```

**Custom Discovery Namespace:**
```yaml
env:
  - name: DISCOVERY_NAMESPACE
    value: "api-docs"  # ConfigMap will be created in api-docs namespace
```

**Custom ConfigMap Name:**
```yaml
env:
  - name: DISCOVERY_CONFIGMAP
    value: "my-api-discovery"  # Custom ConfigMap name
```

### Resource Requirements

The operator is designed to be lightweight:

- **Memory**: 64Mi request, 128Mi limit
- **CPU**: 50m request, 100m limit

## Development

### Building the Operator

```bash
# Build the Rust binary
cargo build --release

# Build Docker image
docker build -t openapi-k8s-operator:latest .
```

### Running Locally

```bash
# Set up kubectl context
kubectl config use-context your-cluster

# Run the operator
cargo run
```

### Testing

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Examples

### Basic Service Annotation

```yaml
apiVersion: v1
kind: Service
metadata:
  name: user-service
  annotations:
    api-doc.io/enabled: "true"
spec:
  ports:
    - port: 8080
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
    - port: 8080
  selector:
    app: payment-service
```

## Troubleshooting

### Check Operator Status

```bash
# Check if the operator is running
kubectl get pods -l app.kubernetes.io/name=openapi-k8s-operator

# Check operator logs
kubectl logs -l app.kubernetes.io/name=openapi-k8s-operator
```

### Check Discovery ConfigMap

```bash
# View the discovery configuration
kubectl get configmap openapi-discovery -o yaml

# Check the discovery JSON
kubectl get configmap openapi-discovery -o jsonpath='{.data.discovery\.json}' | jq .
```

### Check Swagger UI

```bash
# Check Swagger UI status
kubectl get pods -l app.kubernetes.io/name=swagger-ui

# Port forward for local testing
kubectl port-forward svc/swagger-ui 8080:80
```

## Security Considerations

- The operator uses minimal RBAC permissions
- Only watches services and manages the discovery ConfigMap
- No access to sensitive resources or data
- All API calls are made within the cluster network

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
