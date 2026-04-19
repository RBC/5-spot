# Security

5-Spot is designed for deployment in regulated environments (banking, financial services, healthcare) where auditability, least-privilege access, and defence-in-depth are non-negotiable requirements.

This section documents the security controls built into 5-Spot and the guidance for operating it securely.

---

## Documents in This Section

### [Admission Validation](admission-validation.md)

How 5-Spot uses a Kubernetes `ValidatingAdmissionPolicy` to reject invalid `ScheduledMachine` specs at API-server admission time — before they reach the reconciler or are persisted to etcd.

Covers: CEL validation rules, deployment instructions, rollout strategy, and testing.

### [Threat Model](threat-model.md)

A STRIDE-based analysis of the threats facing 5-Spot and the mitigations in place. Includes trust boundaries, threat actor profiles, residual risks, and a compliance control mapping against NIST 800-53, SOX, and Basel III.

### [VEX (Vulnerability Exploitability eXchange)](vex.md)

How 5-Spot publishes a signed OpenVEX document with every release so downstream scanners can suppress CVEs we have already triaged as non-exploitable. Covers the `.vex/` authoring workflow, the Cosign attestation chain, and `grype --vex` / `trivy --vex` consumer usage.

---

## Security Posture at a Glance

| Control | Status | Reference |
|---|---|---|
| Non-root container, read-only rootfs, all caps dropped | ✅ | `deploy/deployment/deployment.yaml` |
| Least-privilege RBAC — explicit resources, no wildcards | ✅ | `deploy/deployment/rbac/clusterrole.yaml` |
| NetworkPolicy — egress to API server only | ✅ | `deploy/deployment/networkpolicy.yaml` |
| Admission validation — 13 CEL rules, `failurePolicy: Fail` | ✅ | `deploy/admission/` |
| Finalizer cleanup timeout (10 min) — prevents deletion hangs | ✅ | `src/constants.rs` |
| Cross-namespace resource creation prevented | ✅ | `namespace` field removed from `EmbeddedResource` |
| Label / annotation injection prevented | ✅ | `validate_labels()` in reconciler |
| API group allowlist for bootstrap / infrastructure providers | ✅ | `validate_api_group()` in reconciler |
| Kubernetes Event audit trail on all phase transitions | ✅ | `update_phase()` in reconciler |
| Integer overflow protection in duration parser | ✅ | `parse_duration()` — `checked_mul` + 24 h cap |
| Structured JSON logging (SIEM-ready) | ✅ | `--log-format json` (default) |
| PodDisruptionBudget — minimum 1 replica | ✅ | `deploy/deployment/pdb.yaml` |

---

## Compliance Mapping Summary

| Framework | Status | Outstanding |
|---|---|---|
| **SOX §404** | ✅ Audit trail via Kubernetes Events | Correlation IDs (future) |
| **Basel III** | ⚠️ Partial | HA leader election, persistent log sink |
| **NIST 800-53 SC-7** | ✅ NetworkPolicy deployed | — |
| **NIST 800-53 CM-5** | ✅ ValidatingAdmissionPolicy deployed | — |
| **NIST 800-53 AU-2/AU-3** | ✅ Phase transition events | Actor tracking |
| **NIST 800-53 SC-28** | ❌ Inline bootstrap credentials | Secret resolution (P3-1) |
