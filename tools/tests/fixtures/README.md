<!--
Copyright (c) 2025 Erick Bourgeois, firestoned
SPDX-License-Identifier: Apache-2.0
-->

# VEX validator test fixtures

Each directory here is an independent input corpus for
`tools/tests/validate-vex-tests.sh`. The test driver runs
`tools/validate-vex.sh <this-dir>` and asserts the expected exit code.

> **Do not "fix" fixtures that look malformed.** Several directories hold
> files that are wrong on purpose — they exist to exercise the validator's
> negative and exception paths.

## Positive-path fixtures (exit 0)

| Directory         | Purpose                                                            |
| ----------------- | ------------------------------------------------------------------ |
| `empty-dir/`      | No TOML files; validator must treat an empty corpus as valid.      |
| `valid-single/`   | One fully-populated `not_affected` statement.                      |
| `valid-multiple/` | Two valid files with different statuses (`not_affected`, `fixed`). |
| `valid-affected/` | Valid `status = "affected"` with the required `action_statement`.  |

## Negative-path fixtures (exit 1)

Each of these exercises exactly one validation rule, so failures point at a
specific defect if one is introduced.

| Directory                   | Rule under test                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `malformed-toml/`            | `tomllib.TOMLDecodeError` branch (unterminated string, missing `]`, unquoted enum).      |
| `missing-cve/`               | Required field `cve` absent.                                                             |
| `missing-status/`            | Required field `status` absent.                                                          |
| `missing-products/`          | Required field `products` absent.                                                        |
| `missing-author/`            | Required field `author` absent.                                                          |
| `missing-timestamp/`         | Required field `timestamp` absent.                                                       |
| `invalid-cve-format/`        | `cve` does not match `CVE-YYYY-NNNN+`.                                                   |
| `invalid-status/`            | `status` is not one of the four OpenVEX enum values.                                     |
| `empty-products/`            | `products = []` — must be a non-empty list.                                              |
| `bad-timestamp/`             | `timestamp` is not an RFC-3339 UTC string (`yesterday`).                                 |
| `missing-justification/`     | `status = "not_affected"` without a `justification`.                                     |
| `invalid-justification/`     | `justification` value is outside the OpenVEX enum (`nah_its_fine`).                      |
| `missing-action-statement/`  | `status = "under_investigation"` without an `action_statement`.                          |
| `duplicate-cve/`             | Two files declaring the same CVE — cross-file uniqueness check.                          |

The test driver also runs a `missing-dir` case against a temp path that does
not exist (no fixture needed for that one — it is constructed at runtime).

## Adding a new case

1. Create `tools/tests/fixtures/<case-name>/` with the TOML file(s) that
   reproduce the condition.
2. Add one `run_case` line to `tools/tests/validate-vex-tests.sh` with the
   expected exit code.
3. Run `./tools/tests/validate-vex-tests.sh` and confirm it passes.
4. If the new rule also affects the assembler, add a matching case to
   `tools/tests/assemble-openvex-tests.sh`.
