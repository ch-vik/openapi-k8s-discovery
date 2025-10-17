# Helm Chart for OpenAPI K8s Operator

This document describes the comprehensive Helm chart created for the OpenAPI K8s Operator, following the latest Helm standards and best practices.

## Chart Structure

```
helm/openapi-k8s-operator/
├── Chart.yaml                    # Chart metadata
├── values.yaml                   # Default configuration values
├── README.md                     # Chart documentation
├── .helmignore                   # Files to ignore during packaging
├── templates/
│   ├── _helpers.tpl             # Template helpers
│   ├── namespace.yaml           # Namespace creation (optional)
│   ├── serviceaccount.yaml      # ServiceAccount for operator
│   ├── rbac.yaml               # RBAC (Role/ClusterRole based on config)
│   ├── statefulset.yaml        # StatefulSet for operator (default)
│   ├── deployment.yaml         # Deployment for operator (alternative)
│   ├── networkpolicy.yaml     # NetworkPolicy for security
│   ├── openapi-server-deployment.yaml  # OpenAPI server deployment
│   ├── openapi-server-service.yaml     # OpenAPI server service
│   ├── openapi-server-ingress.yaml     # OpenAPI server ingress
│   ├── NOTES.txt               # Post-installation notes
│   └── tests/
│       └── test-connection.yaml # Helm test for OpenAPI server
```

## Key Features

### 1. Smart RBAC Configuration

- **Single Namespace**: Uses `Role` and `RoleBinding`
- **Multiple/All Namespaces**: Automatically uses `ClusterRole` and `ClusterRoleBinding`
- **Automatic Detection**: Based on `WATCH_NAMESPACES` environment variable

### 2. Flexible Deployment Options

- **StatefulSet** (default): Better for single-instance operators
- **Deployment**: Alternative option for different use cases
- **Configurable Replicas**: Defaults to 1 (recommended for operators)

### 3. Security Features

- **NetworkPolicy**: Restricts network access based on namespace configuration
- **Pod Security Context**: Non-root user, read-only filesystem
- **Resource Limits**: CPU and memory constraints
- **Security Context**: Drop all capabilities, no privilege escalation

### 4. Optional OpenAPI Server

- **Scalar UI**: Deployable Scalar UI server (Axum)
- **ConfigMap Mount**: Automatically mounts discovery ConfigMap
- **Service & Ingress**: Full networking configuration
- **Customizable**: Image, resources, and deployment options

### 5. Configuration Management

- **Structured Configuration**: Uses `config` section instead of direct environment variables
- **Extra Environment Variables**: `extraEnv` for additional customization
- **Best Practices**: Follows Helm best practices for configuration

## Configuration Examples

### Basic Installation

```bash
helm install openapi-k8s-operator ./helm/openapi-k8s-operator
```

### With OpenAPI Server

```bash
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set openapiServer.enabled=true \
  --set openapiServer.ingress.enabled=true \
  --set openapiServer.ingress.hosts[0].host=openapi.example.com
```

### Custom Namespace

```bash
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set namespace.create=true \
  --set namespace.name=openapi-system \
  --set operator.config.discoveryNamespace=openapi-system
```

## Scaling Considerations

**Important**: This operator is designed to run as a single instance. The chart enforces this with:

- `replicaCount: 1` by default
- StatefulSet for better single-instance management
- Clear documentation about scaling limitations

## Security Best Practices

1. **Network Isolation**: NetworkPolicy restricts communication
2. **Non-Root Execution**: All containers run as non-root user
3. **Read-Only Filesystem**: Operator container uses read-only root filesystem
4. **Resource Limits**: Prevents resource exhaustion
5. **RBAC**: Minimal required permissions

## Validation

The chart has been validated with:

- `helm lint` - No errors or warnings
- `helm template` - All templates render correctly
- Template testing with different configurations
- RBAC logic validation for different namespace scenarios
