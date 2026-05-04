# Servyx Kubernetes Collector

Read-only agent that collects Kubernetes cluster state (namespaces, nodes, pods, deployments, statefulsets, daemonsets) and sends it to [Servyx](https://servyx.ai) for infrastructure intelligence and optimization recommendations.

## What it collects

| Resource | API Group | Permissions |
|----------|-----------|-------------|
| Namespaces | core/v1 | get, list |
| Nodes | core/v1 | get, list |
| Pods | core/v1 | get, list |
| Services | core/v1 | get, list |
| Deployments | apps/v1 | get, list |
| StatefulSets | apps/v1 | get, list |
| DaemonSets | apps/v1 | get, list |
| CronJobs | batch/v1 | get, list |
| Ingresses | networking.k8s.io/v1 | get, list |
| Node/Pod Metrics | metrics.k8s.io | get, list |

**This collector is strictly read-only. It cannot create, modify, or delete any resources.**

## Quick Start

### 1. Get your collector token

Register your cluster on the Servyx dashboard and generate a collector token.

### 2. Install with Helm

```bash
helm install servyx-collector \
  oci://ghcr.io/servyx-ai/servyx-k8s-collector \
  --set collector.clusterId=<YOUR_CLUSTER_ENDPOINT> \
  --set collector.servyxUrl=https://app.servyx.ai \
  --set collector.token=<YOUR_COLLECTOR_TOKEN> \
  --namespace servyx \
  --create-namespace
```

### 3. Done

The collector runs as a CronJob every 6 hours by default. Your cluster data will appear on your Servyx dashboard.

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `collector.clusterId` | Cluster identifier (EKS endpoint or name) | `""` (required) |
| `collector.servyxUrl` | Servyx platform URL | `""` (required) |
| `collector.token` | Collector token from Servyx | `""` (required) |
| `collector.schedule` | Cron schedule for collection | `"0 */6 * * *"` |
| `image.repository` | Container image | `ghcr.io/servyx-ai/servyx-k8s-collector` |
| `image.tag` | Image tag | `0.1.0` |
| `resources.requests.cpu` | CPU request | `50m` |
| `resources.requests.memory` | Memory request | `64Mi` |
| `resources.limits.cpu` | CPU limit | `200m` |
| `resources.limits.memory` | Memory limit | `256Mi` |

### Custom schedule

```bash
# Every hour
helm install servyx-collector ... --set collector.schedule="0 * * * *"

# Every 15 minutes
helm install servyx-collector ... --set collector.schedule="*/15 * * * *"

# Daily at 6 AM
helm install servyx-collector ... --set collector.schedule="0 6 * * *"
```

## Uninstall

```bash
helm uninstall servyx-collector -n servyx
kubectl delete namespace servyx
```

## Security

- **Read-only**: The collector only has `get` and `list` permissions. It cannot modify any resources.
- **Namespaced**: Runs in its own namespace with a dedicated ServiceAccount.
- **Token in Secret**: The collector token is stored as a Kubernetes Secret, not passed as a plain argument.
- **No persistent storage**: The collector runs, sends data, and exits. Nothing is stored locally.

## Development

```bash
# Build locally
cargo build --release

# Run against current kubectl context
./target/release/servyx-k8s-collector collect \
  --cluster-id my-cluster \
  --servyx-url https://app.servyx.ai \
  --token svx_live_...
```

## License

MIT
