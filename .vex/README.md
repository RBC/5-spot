<!--
Copyright (c) 2025 Erick Bourgeois, firestoned
SPDX-License-Identifier: Apache-2.0
-->

# `.vex/` — Per-CVE Triage Source of Truth

This directory is the **human-authored source of truth** for 5-Spot's
[VEX (Vulnerability Exploitability eXchange)](https://github.com/openvex/spec)
statements. Each `.vex/<id>.json` is a single-statement
[OpenVEX](https://github.com/openvex/spec/blob/main/OPENVEX-SPEC.md)
document in the native format. CI merges every file at release time via
[`vexctl`](https://github.com/openvex/vexctl) into a single signed
document that is:

- attached to the GitHub Release as an asset,
- recorded in `checksums.sha256`,
- Cosign-attested against every published image digest (Chainguard +
  Distroless), and
- GitHub-attested via `actions/attest-build-provenance`.

Downstream scanners (Grype, Trivy, Harbor) consume the OpenVEX document
and suppress findings we have already triaged as not applicable.

## Automated statements (roadmap Phase 2)

On every push + release, CI also produces a `vex-auto-presence`
workflow artifact: a separate OpenVEX document containing
`not_affected + component_not_present` statements for every Grype
finding whose affected package URL is not present in any image SBOM
(and is not already triaged in this directory). This is produced by
the `auto-vex-presence` bin under `src/bin/auto_vex_presence.rs` and
is merged **unconditionally** into the signed release VEX alongside
the hand-authored statements in this directory.

Verification of the merged document — including the auto-generated
statements — is performed downstream by the Security team, which
re-evaluates the VEX against the attached evidence (SBOMs, Cosign
attestations, SLSA provenance) and counter-signs the document if they
agree. There is no parallel-run gate on our side: our job is to emit
as aggressively as the evidence supports; their job is to verify.

## When to add a statement

When a scanner (Grype in CI, or a downstream consumer) flags a CVE on a
5-Spot release artifact, open a PR adding **one file per CVE** in this
directory. Merging is gated by:

1. `make vex-validate` — parses every `.vex/*.json` via `vexctl merge`;
   any malformed file fails the merge and blocks the PR.
2. Human review of the impact statement.

No automated "everything is `not_affected`" statements are written. Every
statement is explicitly authored and reviewed.

## File format

One JSON file per advisory, named `<identifier>.json`. Accepted
identifier shapes for the `vulnerability.name` field:

- `CVE-YYYY-NNNN+` — MITRE CVE (the common case).
- `GHSA-xxxx-xxxx-xxxx` — GitHub Security Advisory (used when the
  advisory has no assigned CVE yet, e.g. `GHSA-cq8v-f236-94qc`).
- `RUSTSEC-YYYY-NNNN` — RustSec advisory DB.

Each file is a single-statement OpenVEX v0.2.0 document:

```json
{
  "@context": "https://openvex.dev/ns/v0.2.0",
  "@id": "https://github.com/finos/5-spot/.vex/CVE-2025-12345",
  "author": "erick.bourgeois@gmail.com",
  "timestamp": "2026-04-19T00:00:00Z",
  "version": 1,
  "statements": [
    {
      "vulnerability": {"name": "CVE-2025-12345"},
      "products": [{"@id": "pkg:oci/5-spot"}],
      "status": "not_affected",
      "justification": "vulnerable_code_not_in_execute_path",
      "impact_statement": "5-Spot does not parse untrusted XML; the affected libxml2 code path is never invoked.",
      "timestamp": "2026-04-19T00:00:00Z"
    }
  ]
}
```

The document-level `@id`, `author`, and `timestamp` are replaced by
CI at release time; the statement-level fields (inside `statements[]`)
are what ship in the merged release document.

### Field reference (statement level)

| Field              | Required                                                        | Notes                                                                                                        |
| ------------------ | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `vulnerability.name` | yes                                                           | Canonical identifier: `CVE-YYYY-NNNN+`, `GHSA-xxxx-xxxx-xxxx`, or `RUSTSEC-YYYY-NNNN`.                       |
| `status`           | yes                                                             | One of: `not_affected`, `affected`, `fixed`, `under_investigation`.                                          |
| `justification`    | required iff `status = "not_affected"`                          | OpenVEX enum, see below.                                                                                     |
| `impact_statement` | recommended for `not_affected`                                  | Free-form explanation of why the CVE is non-exploitable in 5-Spot.                                           |
| `action_statement` | required iff `status = "affected"` or `"under_investigation"`   | What a consumer should do until a fix is available (e.g. upgrade path, mitigation).                          |
| `products`         | yes, non-empty                                                  | List of product identifiers. Use package URLs (`pkg:oci/...`) or image references.                           |
| `timestamp`        | yes                                                             | RFC-3339 UTC timestamp.                                                                                      |

### Allowed `justification` values

Per the OpenVEX spec:

- `component_not_present`
- `vulnerable_code_not_present`
- `vulnerable_code_not_in_execute_path`
- `vulnerable_code_cannot_be_controlled_by_adversary`
- `inline_mitigations_already_exist`

## Local validation

```bash
make vex-validate
```

This installs `vexctl` if missing and runs `vexctl merge` over every
`.vex/*.json`. Any malformed file fails the merge — successful
parse = valid structure, valid enum values, and no duplicate
statements for the same (vulnerability, product) pair.

## Local assembly

```bash
make vex-assemble
```

Prints the merged OpenVEX document to stdout. Useful for previewing
the release-time artifact locally.

## References

- [OpenVEX specification](https://github.com/openvex/spec)
- [vexctl](https://github.com/openvex/vexctl)
- [Grype `--vex` flag](https://github.com/anchore/grype#supply-chain-security-with-vex)
