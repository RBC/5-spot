# 5-Spot CALM Architecture

This folder contains the [FINOS Common Architecture Language Model
(CALM)](https://calm.finos.org/) description of 5-Spot.

| File | Purpose |
| --- | --- |
| `architecture.json` | Single architecture document: nodes, relationships, flows, controls, metadata. Targets CALM schema **1.2**. |

## What it models

- **Nodes** — platform operator, management cluster, workload cluster, the
  5-Spot controller, the `ScheduledMachine` CR, the Kubernetes API server,
  CAPI core and bootstrap/infrastructure providers, the physical target node,
  Prometheus, and kubelet.
- **Relationships** — controller-to-API, CAPI reconciliation, SSH to the
  physical node, Prometheus scrape, kubelet probes, and containment in the
  management / workload clusters.
- **Flows** — schedule activation (enter window) and schedule deactivation
  (exit window or kill switch), each mapped to the relationships they traverse.
- **Controls** — least-privilege RBAC, container hardening, leader-election
  HA, NetworkPolicy boundary protection, graceful drain, and supply-chain
  scanning. Controls reference NIST SP 800-53 Rev. 5, SP 800-190, and
  SP 800-218 (SSDF) for traceability.

## Validating

Using the FINOS CALM CLI:

```bash
npm install -g @finos/calm-cli
calm validate -s https://calm.finos.org/release/1.2/meta/calm.json \
              -a docs/architecture/calm/architecture.json
```

Or validate as plain JSON Schema:

```bash
ajv validate \
  -s <(curl -s https://calm.finos.org/release/1.2/meta/calm.json) \
  -d docs/architecture/calm/architecture.json \
  --spec=draft2020
```

## CI: reusable CALM workflow

A reusable GitHub Actions workflow lives at
[`.github/workflows/calm.yaml`](../../../.github/workflows/calm.yaml) and wraps
the CALM CLI. Call it from any other workflow with `workflow_call`:

```yaml
jobs:
  validate:
    uses: ./.github/workflows/calm.yaml
    with:
      command: validate
      architecture: docs/architecture/calm/architecture.json
      strict: true

  mermaid:
    uses: ./.github/workflows/calm.yaml
    with:
      command: template
      architecture: docs/architecture/calm/architecture.json
      template-dir: docs/architecture/calm/templates/mermaid
      output: docs/src/architecture/diagrams
      clear-output-directory: true
      upload-artifact: true
      artifact-name: calm-mermaid
```

Pin a specific CLI version with `cli-version: "1.37.0"` (that is the default).

## Updating

When you add a new controller subsystem, provider, or external dependency:

1. Add a node with a stable `unique-id`.
2. Wire it into the appropriate `deployed-in` / `composed-of` relationship.
3. Add `connects` / `interacts` relationships for each protocol interaction.
4. If any flow traverses the new edge, add a transition.
5. Re-run validation.
