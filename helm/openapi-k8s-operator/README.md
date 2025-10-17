# OpenAPI K8s Operator Helm Chart

This Helm chart deploys the OpenAPI K8s Operator, a Kubernetes operator that aggregates OpenAPI documentation from multiple services and presents them in a centralized Swagger UI.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- Prometheus (optional, for metrics)

## Installation

### Basic Installation

```bash
# Add the chart repository (if using a repository)
helm repo add openapi-k8s-operator https://your-repo.com/charts
helm repo update

# Install the operator in a specific namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator -n openapi-system --create-namespace

# Or install in the default namespace
helm install openapi-k8s-operator ./helm/openapi-k8s-operator
```

### Installation with Custom Values

```bash
# Create a custom values file
cat > custom-values.yaml << EOF
operator:
  config:
    watchNamespaces: "all"
    discoveryNamespace: "default"
    discoveryConfigMap: "openapi-discovery"
  deployment:
    useStatefulSet: true
    replicaCount: 1
  extraEnv:
    - name: LOG_LEVEL
      value: "info"
    - name: METRICS_PORT
      value: "8080"
  serviceAccount:
    create: true
  rbac:
    create: true
    clusterWide: true
  networkPolicy:
    enabled: true
    allowClusterWide: true

openapiServer:
  enabled: true
  image:
    repository: ghcr.io/ch-vik/openapi-k8s-discovery-server
    tag: "0.1.0"
    pullPolicy: IfNotPresent
  service:
    type: ClusterIP
    port: 80
    targetPort: 8080
  ingress:
    enabled: true
    className: "nginx"
    hosts:
      - host: openapi.example.com
        paths:
          - path: /
            pathType: Prefix
  resources:
    limits:
      cpu: 200m
      memory: 256Mi
    requests:
      cpu: 50m
      memory: 64Mi
  extraEnv:
    - name: LOG_LEVEL
      value: "debug"

namespace:
  create: true
  name: "openapi-system"

global:
  imageRegistry: ""
  imagePullSecrets: []
EOF

# Install with custom values
helm install openapi-k8s-operator ./helm/openapi-k8s-operator -f custom-values.yaml
```

## Configuration

### Operator Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `operator.config.watchNamespaces` | Namespaces to watch (empty = current, "all" = all namespaces, comma-separated list) | `""` |
| `operator.config.discoveryNamespace` | Namespace where discovery ConfigMap will be created (defaults to release namespace) | `""` (uses release namespace) |
| `operator.config.discoveryConfigMap` | Name of the discovery ConfigMap | `"openapi-discovery"` |
| `operator.extraEnv` | Additional environment variables for customization | `[]` |
| `operator.deployment.useStatefulSet` | Use StatefulSet instead of Deployment | `true` |
| `operator.deployment.replicaCount` | Number of replicas (should be 1 for operator) | `1` |
| `operator.resources.limits.cpu` | CPU limit | `500m` |
| `operator.resources.limits.memory` | Memory limit | `512Mi` |
| `operator.resources.requests.cpu` | CPU request | `100m` |
| `operator.resources.requests.memory` | Memory request | `128Mi` |
| `operator.serviceAccount.create` | Create service account | `true` |
| `operator.serviceAccount.name` | Service account name | `""` (auto-generated) |
| `operator.rbac.create` | Create RBAC resources | `true` |
| `operator.rbac.clusterWide` | Use cluster-wide RBAC | `false` |
| `operator.networkPolicy.enabled` | Enable network policy | `true` |
| `operator.networkPolicy.allowClusterWide` | Allow cluster-wide communication | `false` |

### OpenAPI Server Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `openapiServer.enabled` | Enable OpenAPI server deployment | `true` |
| `openapiServer.image.repository` | OpenAPI server image repository | `ghcr.io/ch-vik/openapi-k8s-discovery-server` |
| `openapiServer.image.tag` | OpenAPI server image tag | `0.1.0` |
| `openapiServer.image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `openapiServer.deployment.replicaCount` | Number of replicas | `1` |
| `openapiServer.service.type` | Service type | `ClusterIP` |
| `openapiServer.service.port` | Service port | `80` |
| `openapiServer.service.targetPort` | Service target port | `8080` |
| `openapiServer.ingress.enabled` | Enable ingress | `false` |
| `openapiServer.ingress.className` | Ingress class name | `""` |
| `openapiServer.ingress.hosts` | Ingress hosts configuration | `[{host: "openapi.example.com", paths: [{path: "/", pathType: "Prefix"}]}]` |
| `openapiServer.resources.limits.cpu` | CPU limit | `200m` |
| `openapiServer.resources.limits.memory` | Memory limit | `256Mi` |
| `openapiServer.resources.requests.cpu` | CPU request | `50m` |
| `openapiServer.resources.requests.memory` | Memory request | `64Mi` |
| `openapiServer.extraEnv` | Additional environment variables for OpenAPI server | `[]` |

### Global Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `global.imageRegistry` | Global image registry | `""` |
| `global.imagePullSecrets` | Global image pull secrets | `[]` |

### Namespace Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `namespace.create` | Create a dedicated namespace for the operator | `false` |
| `namespace.name` | Namespace name (only used when create: true) | `"openapi-system"` |

### Common Labels and Annotations

| Parameter | Description | Default |
|-----------|-------------|---------|
| `commonLabels` | Common labels for all resources | `{}` |
| `commonAnnotations` | Common annotations for all resources | `{}` |

### Namespace Behavior

The chart follows standard Helm namespace behavior:

- **Deployment Namespace**: All resources are deployed to the namespace specified during `helm install` (via `-n` or `--namespace` flag)
- **Discovery ConfigMap**: Created in the release namespace by default (can be overridden with `operator.config.discoveryNamespace`)
- **Optional Namespace Creation**: The `namespace.create` setting creates a separate namespace resource (useful for dedicated operator namespaces)

### RBAC Configuration

The chart automatically determines if cluster-wide RBAC is needed based on the `WATCH_NAMESPACES` setting:

- If `WATCH_NAMESPACES` is empty or a single namespace: Uses Role and RoleBinding
- If `WATCH_NAMESPACES` is "all" or multiple namespaces: Uses ClusterRole and ClusterRoleBinding

### Network Policy

The chart includes NetworkPolicies that provide secure communication:

**Operator NetworkPolicy:**
- Allows ingress on port 8080 (metrics) from the same namespace
- If watching all namespaces, allows ingress from any namespace
- Allows egress to Kubernetes API server (ports 443, 6443)
- Allows egress to services in the same namespace (ports 80, 443, 8080)
- If watching all namespaces, allows egress to services across all namespaces
- Allows DNS resolution (UDP/TCP port 53)

**OpenAPI Server NetworkPolicy (when enabled):**
- Allows ingress on port 8080 from any namespace (cluster-wide access)
- Allows egress to Kubernetes API server and all services for fetching OpenAPI specs
- Allows communication with services on common ports (80, 443, 8080, 3000, 8000, 9000, 10000)

### OpenAPI Server Configuration

The OpenAPI server automatically receives the same configuration as the operator:

- **DISCOVERY_NAMESPACE**: Namespace where the discovery ConfigMap is located
- **DISCOVERY_CONFIGMAP**: Name of the discovery ConfigMap to read from
- **ConfigMap Mount**: The discovery ConfigMap is mounted at `/etc/config`
- **Custom Environment Variables**: Use `openapiServer.extraEnv` for additional configuration

## Usage

### Annotating Services

To make a service discoverable by the operator, add these annotations:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-api-service
  annotations:
    api-doc.io/enabled: "true"
    api-doc.io/path: "/swagger/openapi.yml"
    api-doc.io/name: "My API"
    api-doc.io/description: "My API Documentation"
spec:
  # ... service spec
```

### Accessing the OpenAPI Server

If the OpenAPI server is enabled, you can access it via:

- **Service**: `http://openapi-server.default.svc.cluster.local`
- **Ingress**: `https://openapi.example.com` (if ingress is enabled)

### Monitoring

The operator exposes Prometheus metrics on port 8080 at `/metrics`. You can scrape these metrics using your existing Prometheus configuration or ServiceMonitor resources.

## Scaling Considerations

**Important**: This operator is designed to run as a single instance. Running multiple replicas can cause issues with ConfigMap updates and service discovery. The chart defaults to `replicaCount: 1` and uses a StatefulSet to ensure only one instance runs.

If you need high availability, consider:
1. Running the operator in a single namespace with multiple services
2. Using a different approach for multi-cluster scenarios
3. Implementing leader election (not currently supported)

## Troubleshooting

### Check Operator Logs

```bash
kubectl logs -l app.kubernetes.io/name=openapi-k8s-operator
```

### Check Discovery ConfigMap

```bash
kubectl get configmap openapi-discovery -o yaml
```

### Verify Service Annotations

```bash
kubectl get services -o custom-columns=NAME:.metadata.name,ENABLED:.metadata.annotations.api-doc\.io/enabled,PATH:.metadata.annotations.api-doc\.io/path
```

### Check NetworkPolicy Issues

If the operator or OpenAPI server cannot communicate:

```bash
# Check NetworkPolicy status
kubectl get networkpolicy

# Check if pods can reach Kubernetes API server
kubectl exec -it <operator-pod> -- curl -k https://kubernetes.default.svc.cluster.local/api/v1/namespaces

# Check if OpenAPI server can reach services
kubectl exec -it <openapi-server-pod> -- curl http://<service-name>.<namespace>.svc.cluster.local
```

### Disable NetworkPolicy (if needed)

```bash
helm upgrade openapi-k8s-operator ./helm/openapi-k8s-operator \
  --set operator.networkPolicy.enabled=false
```

## Uninstallation

```bash
helm uninstall openapi-k8s-operator
```

**Note**: This will not remove the discovery ConfigMap or any services. You may want to clean up manually if needed.
