#!/usr/bin/env bash
# Copyright (c) 2025 Erick Bourgeois, firestoned
# SPDX-License-Identifier: Apache-2.0
#
# validate-vex.sh — schema, enum, and uniqueness gate for `.vex/*.toml`.
#
# Exits 0 if every VEX statement file in the target directory is well-formed
# and the set of CVE IDs is unique across files. Exits 1 on any violation and
# prints a line per failure to stderr.
#
# Usage:
#   tools/validate-vex.sh              # validate the repo's .vex/ dir
#   tools/validate-vex.sh <dir>        # validate an alternate directory (tests)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="${1:-$REPO_ROOT/.vex}"

if ! command -v python3 >/dev/null 2>&1; then
    echo "validate-vex: python3 is required (needs tomllib, Python >= 3.11)" >&2
    exit 2
fi

exec python3 "$SCRIPT_DIR/validate_vex.py" "$TARGET_DIR"
