#!/usr/bin/env bats
# Copyright (c) 2025 Erick Bourgeois, firestoned
# SPDX-License-Identifier: Apache-2.0
#
# Unit tests for .github/scripts/calm-args.sh.
# Install bats-core (brew install bats-core) and run:
#   bats .github/scripts/calm-args.bats

setup() {
  SCRIPT="${BATS_TEST_DIRNAME}/calm-args.sh"
  [ -x "$SCRIPT" ] || chmod +x "$SCRIPT"
  # Clear every env var the script reads so prior tests can't leak in.
  unset CMD ARCH PATTERN OUTPUT TEMPLATE TEMPLATE_DIR BUNDLE URL_MAP \
        SCHEMA_DIR HUB_URL CLEAR_OUT SCAFFOLD STRICT FORMAT VERBOSE EXTRA
}

# ── Command validation ───────────────────────────────────────────────────────

@test "fails when CMD is unset" {
  run env -u CMD "$SCRIPT"
  [ "$status" -ne 0 ]
  [[ "$output" == *"CMD is required"* ]]
}

@test "fails with exit 2 on unknown command" {
  run env CMD=deploy "$SCRIPT"
  [ "$status" -eq 2 ]
  [[ "$output" == *"Unsupported command"* ]]
}

@test "accepts each supported command with no extra args" {
  for cmd in validate generate template docify; do
    run env CMD="$cmd" "$SCRIPT"
    [ "$status" -eq 0 ] || {
      echo "cmd=$cmd status=$status output=$output" >&2
      return 1
    }
    # No args emitted when nothing else is supplied.
    [ -z "$output" ]
  done
}

# ── Flag mapping ─────────────────────────────────────────────────────────────

@test "architecture maps to -a" {
  run env CMD=template ARCH=arch.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-a" ]
  [ "${lines[1]}" = "arch.json" ]
}

@test "pattern maps to -p" {
  run env CMD=generate PATTERN=pattern.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-p" ]
  [ "${lines[1]}" = "pattern.json" ]
}

@test "output maps to -o" {
  run env CMD=generate OUTPUT=out.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-o" ]
  [ "${lines[1]}" = "out.json" ]
}

@test "template single file maps to -t" {
  run env CMD=template TEMPLATE=mermaid.hbs "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-t" ]
  [ "${lines[1]}" = "mermaid.hbs" ]
}

@test "template dir maps to -d" {
  run env CMD=template TEMPLATE_DIR=templates/mermaid "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-d" ]
  [ "${lines[1]}" = "templates/mermaid" ]
}

@test "bundle maps to -b" {
  run env CMD=template BUNDLE=bundles/one-pager "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-b" ]
  [ "${lines[1]}" = "bundles/one-pager" ]
}

@test "url-map maps to -u" {
  run env CMD=template URL_MAP=urlmap.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-u" ]
  [ "${lines[1]}" = "urlmap.json" ]
}

@test "schema-directory maps to -s" {
  run env CMD=validate SCHEMA_DIR=./calm/release "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-s" ]
  [ "${lines[1]}" = "./calm/release" ]
}

@test "calm-hub-url maps to -c" {
  run env CMD=validate HUB_URL=https://hub.example.com "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-c" ]
  [ "${lines[1]}" = "https://hub.example.com" ]
}

# ── Boolean scoping ──────────────────────────────────────────────────────────

@test "clear-output maps to --clear-output-directory when true" {
  run env CMD=template CLEAR_OUT=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "--clear-output-directory" ]
}

@test "clear-output is omitted when false" {
  run env CMD=template CLEAR_OUT=false "$SCRIPT"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "scaffold is emitted only for docify" {
  run env CMD=docify SCAFFOLD=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"--scaffold"* ]]

  run env CMD=template SCAFFOLD=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" != *"--scaffold"* ]]
}

@test "strict is emitted only for validate" {
  run env CMD=validate STRICT=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"--strict"* ]]

  run env CMD=generate STRICT=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" != *"--strict"* ]]
}

@test "format is emitted only for validate" {
  run env CMD=validate FORMAT=junit "$SCRIPT"
  [ "$status" -eq 0 ]
  # Find "-f" then "junit" as consecutive lines.
  found=0
  for i in "${!lines[@]}"; do
    if [ "${lines[$i]}" = "-f" ] && [ "${lines[$((i+1))]}" = "junit" ]; then
      found=1; break
    fi
  done
  [ "$found" -eq 1 ]

  run env CMD=template FORMAT=junit "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" != *"-f"* ]]
}

@test "verbose maps to -v" {
  run env CMD=validate VERBOSE=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"-v"* ]]
}

@test "verbose default (unset) emits nothing" {
  run env CMD=validate "$SCRIPT"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

# ── Extra args passthrough ───────────────────────────────────────────────────

@test "extra args are word-split and appended" {
  run env CMD=template EXTRA="--foo bar --baz" "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "--foo" ]
  [ "${lines[1]}" = "bar" ]
  [ "${lines[2]}" = "--baz" ]
}

# ── End-to-end compositions ──────────────────────────────────────────────────

@test "validate with architecture, strict, format, and verbose" {
  run env CMD=validate ARCH=arch.json STRICT=true FORMAT=pretty VERBOSE=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"-a"* ]]
  [[ "$output" == *"arch.json"* ]]
  [[ "$output" == *"--strict"* ]]
  [[ "$output" == *"-f"* ]]
  [[ "$output" == *"pretty"* ]]
  [[ "$output" == *"-v"* ]]
}

@test "generate with pattern and output" {
  run env CMD=generate PATTERN=p.json OUTPUT=a.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"-p"* ]]
  [[ "$output" == *"p.json"* ]]
  [[ "$output" == *"-o"* ]]
  [[ "$output" == *"a.json"* ]]
}

@test "template with architecture, template-dir, output, clear" {
  run env CMD=template ARCH=a.json TEMPLATE_DIR=tpl OUTPUT=out CLEAR_OUT=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"-a"* ]]
  [[ "$output" == *"-d"* ]]
  [[ "$output" == *"tpl"* ]]
  [[ "$output" == *"-o"* ]]
  [[ "$output" == *"out"* ]]
  [[ "$output" == *"--clear-output-directory"* ]]
}

@test "docify scaffold mode" {
  run env CMD=docify ARCH=a.json OUTPUT=site SCAFFOLD=true "$SCRIPT"
  [ "$status" -eq 0 ]
  [[ "$output" == *"-a"* ]]
  [[ "$output" == *"a.json"* ]]
  [[ "$output" == *"-o"* ]]
  [[ "$output" == *"site"* ]]
  [[ "$output" == *"--scaffold"* ]]
}

@test "values with spaces survive word-preservation for named flags" {
  # ARCH may include spaces; script must pass it through as a single arg.
  run env CMD=template ARCH="my arch.json" "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "-a" ]
  [ "${lines[1]}" = "my arch.json" ]
}

@test "output line count matches exactly for a minimal invocation" {
  run env CMD=validate ARCH=a.json "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "${#lines[@]}" -eq 2 ]
}
