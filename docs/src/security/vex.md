<!--
Copyright (c) 2025 Erick Bourgeois, firestoned
SPDX-License-Identifier: Apache-2.0
-->

# VEX (Vulnerability Exploitability eXchange)

5-Spot publishes a signed
[OpenVEX](https://github.com/openvex/spec/blob/main/OPENVEX-SPEC.md)
document with every GitHub Release. The document records, per CVE, whether
the finding is `not_affected`, `affected`, `fixed`, or
`under_investigation` in **this specific release** of 5-Spot, plus an
OpenVEX-spec justification when we claim non-exploitability.

VEX closes the gap between "Grype flagged a CVE" and "is this CVE
actually reachable in 5-Spot?". The 5-Spot CI pipeline feeds the VEX
document into [Grype](https://github.com/anchore/grype) (`grype --vex
...`) to suppress pre-triaged findings before they reach GitHub Code
Scanning; downstream consumers can do the same with Grype or with
[Trivy](https://aquasecurity.github.io/trivy/latest/docs/supply-chain/vex/),
so the triage burden does not land on every downstream team
independently.

---

## What is published, and how

On every release of 5-Spot the CI pipeline performs the following steps:

1. **Validate** every file under `.vex/*.toml` (schema, enum values, CVE
   uniqueness) with `tools/validate-vex.sh`.
2. **Assemble** a single OpenVEX document (`vex.openvex.json`) with
   `tools/assemble_openvex.py`, stamped with a canonical
   `@id = https://github.com/<owner>/<repo>/releases/tag/<tag>/vex`.
3. **Cross-check** the output with `vexctl validate`.
4. **Cosign-attest** the document to *both* image digests (Chainguard
   and Distroless). The attestation lands in the Sigstore transparency
   log and is pushed to the registry alongside the image.
5. **GitHub attest** the document with `actions/attest-build-provenance`
   so `gh attestation verify` works for downstream pulls.
6. **Attach** `vex.openvex.json` and its `.bundle` to the GitHub
   Release as assets and register them in `checksums.sha256`.

No new GitHub secrets are required — all signing is keyless via the
GitHub OIDC token and Sigstore Fulcio.

---

## Consuming the VEX document

### With Grype

```sh
grype --vex vex.openvex.json \
    ghcr.io/<owner>/5-spot-chainguard@<digest>
```

Grype will suppress every CVE the document marks `not_affected` or
`fixed` for the scanned image, with the OpenVEX statement as the audit
record.

### With Trivy

```sh
trivy image \
    --vex file:vex.openvex.json \
    ghcr.io/<owner>/5-spot-chainguard@<digest>
```

### Verifying the Cosign attestation end-to-end

```sh
cosign verify-attestation \
    --type openvex \
    --certificate-identity-regexp '^https://github.com/<owner>/5-spot' \
    --certificate-oidc-issuer https://token.actions.githubusercontent.com \
    ghcr.io/<owner>/5-spot-chainguard@<digest>
```

### Verifying the GitHub attestation

```sh
gh attestation verify vex.openvex.json --repo <owner>/5-spot
```

Replace `<owner>` with the GitHub organization hosting your 5-Spot
release (for example, `finos`).

---

## How a 5-Spot maintainer adds a statement

When a new CVE surfaces on a release artifact, open a PR adding a
single file to [`.vex/`](https://github.com/finos/5-spot/tree/main/.vex):

```toml
# .vex/CVE-2025-12345.toml
cve = "CVE-2025-12345"
status = "not_affected"
justification = "vulnerable_code_not_in_execute_path"
impact_statement = "5-Spot does not parse untrusted XML; the affected libxml2 code path is never invoked."
products = [
    "pkg:oci/5-spot-chainguard",
    "pkg:oci/5-spot-distroless",
]
author = "maintainer@example"
timestamp = "2026-04-19T00:00:00Z"
```

The PR gate on `build.yaml` (`validate-vex` job) blocks malformed files
from merging. The same validator is re-run on release for
belt-and-suspenders.

### Required fields per status

| `status`              | Extra required field | Notes                                                              |
| --------------------- | -------------------- | ------------------------------------------------------------------ |
| `not_affected`        | `justification`      | OpenVEX enum. `impact_statement` is strongly recommended.          |
| `affected`            | `action_statement`   | What users should do until fixed (upgrade path, mitigation, etc.). |
| `under_investigation` | `action_statement`   | Same — give consumers something actionable.                        |
| `fixed`               | —                    | Just declares the CVE no longer applies to this release.           |

All four statuses additionally require `cve`, `products`, `author`, and
`timestamp` (RFC-3339 UTC).

---

## Why we did not auto-generate statements from scanner output

The VEX document is a trust claim, not a compliance artifact. If 5-Spot
automatically emitted `not_affected` for every Grype finding, the
document would be worthless the moment Grype missed a true positive.

The `.vex/` directory is therefore **hand-authored and PR-reviewed**.
Grype findings drive maintainers to write statements; the statements
themselves are deliberate human decisions. This keeps the audit trail
honest.

---

## References

- [OpenVEX specification](https://github.com/openvex/spec)
- [`vexctl`](https://github.com/openvex/vexctl)
- [Cosign OpenVEX attestation docs](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [Grype `--vex` flag](https://github.com/anchore/grype#supply-chain-security-with-vex)
- Repository convention: [`.vex/README.md`](https://github.com/finos/5-spot/blob/main/.vex/README.md)
