#!/usr/bin/env bash
# Copyright (c) 2025 Erick Bourgeois, firestoned
# SPDX-License-Identifier: Apache-2.0
#
# Builds the argument list for `calm <CMD>` from environment variables.
# Prints one argument per line on stdout so callers can read into an array:
#
#   mapfile -t args < <(CMD=validate ARCH=x.json calm-args.sh)
#   calm "$CMD" "${args[@]}"
#
# Exit codes:
#   0  success
#   2  CMD unset or not one of: validate | generate | template | docify
#
# Inputs (environment):
#   CMD            required — validate | generate | template | docify
#   ARCH           -a <path>          (template, docify, validate)
#   PATTERN        -p <file|url>      (generate, validate)
#   OUTPUT         -o <path>          (all)
#   TEMPLATE       -t <file>          (template, docify)
#   TEMPLATE_DIR   -d <dir>           (template, docify)
#   BUNDLE         -b <dir>           (template)
#   URL_MAP        -u <file>          (template, docify)
#   SCHEMA_DIR     -s <dir>           (generate, validate)
#   HUB_URL        -c <url>           (generate, validate)
#   CLEAR_OUT      "true" -> --clear-output-directory  (template, docify)
#   SCAFFOLD       "true" -> --scaffold                (docify only)
#   STRICT         "true" -> --strict                  (validate only)
#   FORMAT         -f <json|junit|pretty>              (validate only)
#   VERBOSE        "true" -> -v
#   EXTRA          raw additional args, word-split
set -euo pipefail

: "${CMD:?CMD is required (validate | generate | template | docify)}"

case "$CMD" in
  validate|generate|template|docify) ;;
  *)
    echo "Unsupported command: $CMD (expected: validate, generate, template, docify)" >&2
    exit 2
    ;;
esac

declare -a args=()

[[ -n "${ARCH:-}"         ]] && args+=(-a "$ARCH")
[[ -n "${PATTERN:-}"      ]] && args+=(-p "$PATTERN")
[[ -n "${OUTPUT:-}"       ]] && args+=(-o "$OUTPUT")
[[ -n "${TEMPLATE:-}"     ]] && args+=(-t "$TEMPLATE")
[[ -n "${TEMPLATE_DIR:-}" ]] && args+=(-d "$TEMPLATE_DIR")
[[ -n "${BUNDLE:-}"       ]] && args+=(-b "$BUNDLE")
[[ -n "${URL_MAP:-}"      ]] && args+=(-u "$URL_MAP")
[[ -n "${SCHEMA_DIR:-}"   ]] && args+=(-s "$SCHEMA_DIR")
[[ -n "${HUB_URL:-}"      ]] && args+=(-c "$HUB_URL")

[[ "${CLEAR_OUT:-false}" == "true" ]] && args+=(--clear-output-directory)
[[ "${SCAFFOLD:-false}"  == "true" && "$CMD" == "docify"   ]] && args+=(--scaffold)
[[ "${STRICT:-false}"    == "true" && "$CMD" == "validate" ]] && args+=(--strict)

if [[ "$CMD" == "validate" && -n "${FORMAT:-}" ]]; then
  args+=(-f "$FORMAT")
fi

[[ "${VERBOSE:-false}" == "true" ]] && args+=(-v)

if [[ -n "${EXTRA:-}" ]]; then
  # shellcheck disable=SC2206
  extra_arr=($EXTRA)
  args+=("${extra_arr[@]}")
fi

if [[ ${#args[@]} -gt 0 ]]; then
  printf '%s\n' "${args[@]}"
fi
