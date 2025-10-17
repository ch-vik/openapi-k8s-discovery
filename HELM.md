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

### 1. Enhanced OpenAPI Server Integration

- **Custom Discovery Server**: Built-in Rust/Axum server for OpenAPI documentation aggregation
- **ConfigMap Integration**: Automatic mounting of discovery ConfigMap
- **Flexible Networking**: Configurable service ports and ingress settings
- **Resource Optimization**: Tunable CPU and memory limits for different environments
- **Security Hardening**: Non-root execution with proper security contexts

### 2. Smart RBAC Configuration

- **Single Namespace**: Uses `Role` and `RoleBinding`
- **Multiple/All Namespaces**: Automatically uses `ClusterRole` and `ClusterRoleBinding`
- **Automatic Detection**: Based on `WATCH_NAMESPACES` environment variable

### 3. Flexible Deployment Options

- **StatefulSet** (default): Better for single-instance operators
- **Deployment**: Alternative option for different use cases
- **Configurable Replicas**: Defaults to 1 (recommended for operators)

### 4. Security Features

- **NetworkPolicy**: Comprehensive network policies for operator and OpenAPI server
  - Operator: Secure communication with Kubernetes API server and services
  - OpenAPI Server: Cluster-wide access for fetching OpenAPI specifications
- **Pod Security Context**: Non-root user, read-only filesystem
- **Resource Limits**: CPU and memory constraints
- **Security Context**: Drop all capabilities, no privilege escalation

### 5. Optional OpenAPI Server

- **Custom Server**: Deployable OpenAPI discovery server (Rust/Axum)
- **ConfigMap Mount**: Automatically mounts discovery ConfigMap
- **Service & Ingress**: Full networking configuration with customizable ports
- **Resource Management**: Configurable CPU and memory limits/requests
- **Security Context**: Non-root execution with proper security settings
- **Image Configuration**: Customizable repository, tag, and pull policy

### 6. Configuration Management

- **Structured Configuration**: Uses `config` section instead of direct environment variables
- **Extra Environment Variables**: `extraEnv` for additional customization
- **Global Settings**: Image registry and pull secrets configuration
- **Namespace Management**: Optional namespace creation and configuration
- **Common Labels/Annotations**: Consistent labeling across all resources
- **Best Practices**: Follows Helm best practices for configuration

## Latest Configuration Options

### Global Settings
- `global.imageRegistry`: Override image registry for all images
- `global.imagePullSecrets`: Global image pull secrets
- `commonLabels`: Common labels applied to all resources
- `commonAnnotations`: Common annotations applied to all resources

### Namespace Management
- `namespace.create`: Create a dedicated namespace resource (separate from deployment namespace)
- `namespace.name`: Namespace name for the created namespace resource
- **Deployment Namespace**: All resources deploy to the namespace specified during `helm install`
- **Discovery ConfigMap**: Created in release namespace by default (configurable via `operator.config.discoveryNamespace`)

### Enhanced OpenAPI Server
- `openapiServer.enabled`: Enable the OpenAPI discovery server (default: true)
- `openapiServer.image.repository`: Custom image repository
- `openapiServer.image.tag`: Image tag version
- `openapiServer.image.pullPolicy`: Image pull policy
- `openapiServer.service.targetPort`: Custom target port for the service
- `openapiServer.resources`: CPU and memory limits/requests
- `openapiServer.extraEnv`: Additional environment variables for customization
- **Automatic Configuration**: Receives same discovery settings as operator (DISCOVERY_NAMESPACE, DISCOVERY_CONFIGMAP)

### Monitoring Integration
- The operator exposes Prometheus metrics on port 8080 at `/metrics`
- You can scrape these metrics using your existing Prometheus configuration
- ServiceMonitor resources can be created separately if needed

## Configuration Examples

### Basic Installation

```bash
# Install in a specific namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator -n openapi-system --create-namespace

# Or install in default namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator
```

### With OpenAPI Server

```bash
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set openapiServer.enabled=true \
  --set openapiServer.image.repository=ghcr.io/ch-vik/openapi-k8s-discovery-server \
  --set openapiServer.image.tag=0.1.0 \
  --set openapiServer.ingress.enabled=true \
  --set openapiServer.ingress.className=nginx \
  --set openapiServer.ingress.hosts[0].host=openapi.example.com
```

### Custom Namespace

```bash
# Create a dedicated namespace and install in it
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  -n openapi-system --create-namespace \
  --set namespace.create=true \
  --set namespace.name=openapi-system

# Or install in existing namespace with custom discovery namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  -n my-namespace \
  --set operator.config.discoveryNamespace=my-namespace
```

### Production Deployment with Monitoring

```bash
helm install openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set operator.config.watchNamespaces=all \
  --set operator.rbac.clusterWide=true \
  --set operator.networkPolicy.allowClusterWide=true \
  --set openapiServer.enabled=true \
  --set openapiServer.ingress.enabled=true \
  --set openapiServer.ingress.className=nginx \
  --set openapiServer.ingress.hosts[0].host=api-docs.company.com \
  --set openapiServer.resources.limits.cpu=500m \
  --set openapiServer.resources.limits.memory=512Mi \
  --set global.imageRegistry=your-registry.com \
  --set namespace.create=true \
  --set namespace.name=openapi-system
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
