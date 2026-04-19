# Prerequisites

Before installing 5-Spot, ensure your environment meets the following requirements.

## Kubernetes Cluster

- **Version**: Kubernetes 1.27 or later
- **Access**: `kubectl` configured with cluster-admin privileges
- **RBAC**: Role-Based Access Control enabled

## Cluster API (CAPI)

5-Spot integrates with Cluster API for machine management. It is
provider-agnostic: ScheduledMachine's `bootstrapSpec` and
`infrastructureSpec` embed whatever bootstrap / infrastructure CRDs
your CAPI installation provides.

### Required Components

- CAPI Core Provider (v1.5+)
- A **Bootstrap Provider** — e.g. `K0sWorkerConfig` (k0smotron),
  `KubeadmConfig` (kubeadm bootstrap)
- An **Infrastructure Provider** — e.g. `RemoteMachine` (k0smotron),
  `Metal3Machine`, `PacketMachine`, `VSphereMachine`

The quickstart uses k0smotron (`K0sWorkerConfig` + `RemoteMachine`)
because it needs no cloud credentials; substitute your own provider
CRDs as appropriate.

### Verify CAPI Installation

```bash
# Check CAPI core
kubectl get pods -n capi-system

# Check the bootstrap + infrastructure providers you installed
# (namespace varies: capi-kubeadm-bootstrap-system, k0smotron, etc.)
kubectl get pods -A | grep -E 'capi|k0smotron'

# Verify CRDs
kubectl get crds | grep cluster.x-k8s.io
```

## Network Requirements

| Source | Destination | Port | Protocol | Purpose |
|--------|-------------|------|----------|---------|
| Operator Pod | Kubernetes API | 443/6443 | HTTPS | Reconciles CAPI CRs |
| Prometheus | Operator Pod | 8080 | HTTP | `/metrics` scrape |
| Kubelet | Operator Pod | 8081 | HTTP | liveness/readiness probes |

5-Spot itself only talks to the Kubernetes API — it does not SSH to
machines. Any SSH egress to target hardware is performed by the
infrastructure provider (e.g. k0smotron's RemoteMachine controller),
not by 5-Spot.

## Resource Requirements

### Operator Pod

| Resource | Request | Limit |
|----------|---------|-------|
| CPU | 100m | 500m |
| Memory | 128Mi | 512Mi |

These values match the shipped `deploy/deployment/deployment.yaml`;
override via your kustomize/Helm overlay if your workload needs more.

### Storage

- No persistent storage required
- ConfigMaps for configuration

## Optional Components

### Prometheus (Recommended)

For metrics collection and monitoring:

```bash
kubectl get pods -n monitoring | grep prometheus
```

### Cert-Manager (For Webhooks)

If using admission webhooks:

```bash
kubectl get pods -n cert-manager
```

## Next Steps

- [Quick Start](./quickstart.md) - Get started quickly
- [Installing CRDs](./crds.md) - Install Custom Resource Definitions
- [Deploying Operator](./controller.md) - Deploy the 5-Spot controller
