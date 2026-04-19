# Reusable CALM workflow

[`.github/workflows/calm.yaml`](./calm.yaml) is a `workflow_call` wrapper
around the [FINOS CALM CLI][calm-cli] (`@finos/calm-cli`). It installs a
pinned CLI version, builds the argument list from workflow inputs, and
runs one of the CLI's four sub-commands against a CALM architecture or
pattern in this repo.

Typical uses from other workflows:

- **Validate** the checked-in architecture against its CALM meta-schema
  (or a pattern) on every PR.
- **Generate** an architecture stub from a pattern.
- **Template** a set of Handlebars templates (e.g. Mermaid diagrams) from
  the architecture and commit or publish the output.
- **Docify** a full documentation site from the architecture.

Argument-building lives in [`.github/scripts/calm-args.sh`](../scripts/calm-args.sh)
so it can be unit-tested with [bats][bats-core] — see the
[Testing](#testing) section below.

---

## Contents

- [Quick start](#quick-start)
- [Inputs](#inputs)
- [Outputs](#outputs)
- [Per-sub-command examples](#per-sub-command-examples)
  - [`validate`](#validate)
  - [`generate`](#generate)
  - [`template` (Mermaid)](#template-mermaid)
  - [`docify`](#docify)
- [Pinning the CLI version](#pinning-the-cli-version)
- [Permissions](#permissions)
- [Artifacts](#artifacts)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

---

## Quick start

Call the workflow from any other workflow in this repo:

```yaml
jobs:
  validate-calm:
    uses: ./.github/workflows/calm.yaml
    with:
      command: validate
      architecture: docs/architecture/calm/architecture.json
      strict: true
      format: pretty
```

---

## Inputs

All inputs are strings unless noted.

| Name | Required | Default | Applies to | Maps to | Description |
| --- | --- | --- | --- | --- | --- |
| `command` | ✅ | — | all | — | `validate` \| `generate` \| `template` \| `docify`. |
| `cli-version` | | `1.37.0` | all | `npm install -g @finos/calm-cli@<ver>` | Pinned CLI version. |
| `node-version` | | `20` | all | `actions/setup-node` | Node runtime. |
| `working-directory` | | `.` | all | shell `working-directory` | Dir the CLI is invoked from. |
| `architecture` | | `""` | validate, template, docify | `-a <path>` | Path to the CALM architecture JSON. |
| `pattern` | | `""` | generate, validate | `-p <file\|url>` | Path/URL of a CALM pattern. |
| `output` | | `""` | all | `-o <path>` | Output file (generate/validate) or directory (template/docify). |
| `template` | | `""` | template, docify | `-t <file>` | Single `.hbs`/`.md` template. |
| `template-dir` | | `""` | template, docify | `-d <dir>` | Directory of templates. |
| `bundle` | | `""` | template | `-b <dir>` | Template bundle directory. |
| `url-to-local-file-mapping` | | `""` | template, docify | `-u <file>` | URL → local path JSON map. |
| `schema-directory` | | `""` | generate, validate | `-s <dir>` | Local CALM meta-schema dir. |
| `calm-hub-url` | | `""` | generate, validate | `-c <url>` | CalmHub URL. |
| `clear-output-directory` (bool) | | `false` | template, docify | `--clear-output-directory` | Wipe output dir first. |
| `scaffold` (bool) | | `false` | docify | `--scaffold` | Scaffold-only mode. |
| `strict` (bool) | | `false` | validate | `--strict` | Fail on warnings too. |
| `format` | | `json` | validate | `-f <fmt>` | `json` \| `junit` \| `pretty`. |
| `verbose` (bool) | | `false` | all | `-v` | Verbose CLI logging. |
| `extra-args` | | `""` | all | appended raw | Extra arguments appended to the CLI call (word-split). |
| `upload-artifact` (bool) | | `false` | all | — | Upload `output` as a workflow artifact. |
| `artifact-name` | | `calm-output` | all | — | Artifact name. |
| `artifact-retention-days` (number) | | `30` | all | — | Artifact retention. |

Boolean-scoped flags (`scaffold`, `strict`, `format`) are only emitted
for the sub-commands that accept them — passing them to the wrong
command is a no-op rather than an error.

## Outputs

| Name | Description |
| --- | --- |
| `output-path` | Resolved `output` path the CLI wrote to. Empty if `output` was not supplied. |

---

## Per-sub-command examples

### `validate`

```yaml
jobs:
  calm-validate:
    uses: ./.github/workflows/calm.yaml
    with:
      command: validate
      architecture: docs/architecture/calm/architecture.json
      format: pretty   # human-readable
      strict: true     # warnings fail the job
      verbose: true
```

Validating against a specific pattern (instead of only the meta-schema):

```yaml
    with:
      command: validate
      architecture: docs/architecture/calm/architecture.json
      pattern: docs/architecture/calm/pattern.json
      format: junit
      output: validate-report.xml
      upload-artifact: true
      artifact-name: calm-validate-report
```

### `generate`

Create an architecture stub from a pattern file:

```yaml
jobs:
  calm-generate:
    uses: ./.github/workflows/calm.yaml
    with:
      command: generate
      pattern: docs/architecture/calm/pattern.json
      output: docs/architecture/calm/architecture.json
      verbose: true
```

### `template` (Mermaid)

Render Mermaid diagrams from the architecture using a Handlebars
template directory. Output lands somewhere MkDocs includes
(`docs/src/architecture/diagrams/` in this repo):

```yaml
jobs:
  calm-mermaid:
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

A minimal template (`nodes.md.hbs`) that emits a Mermaid graph:

````handlebars
```mermaid
flowchart LR
{{#each nodes}}
  {{this.unique-id}}["{{this.name}}"]
{{/each}}
{{#each relationships}}
  {{#if this.relationship-type.connects}}
  {{this.relationship-type.connects.source.node}} -->|{{this.protocol}}| {{this.relationship-type.connects.destination.node}}
  {{/if}}
{{/each}}
```
````

You can also use a full [template bundle][calm-template-bundle] via
`bundle:` instead of `template-dir:`.

### `docify`

Generate a full docs site from the architecture:

```yaml
jobs:
  calm-docify:
    uses: ./.github/workflows/calm.yaml
    with:
      command: docify
      architecture: docs/architecture/calm/architecture.json
      output: docs/src/architecture/site
      clear-output-directory: true
```

Scaffold-only mode (stage 1 of the two-stage docify workflow):

```yaml
    with:
      command: docify
      architecture: docs/architecture/calm/architecture.json
      output: docs/src/architecture/site
      scaffold: true
```

---

## Pinning the CLI version

The default `cli-version` is pinned (`1.37.0`) so downstream workflows
don't silently break when a new CLI ships. Override per-call:

```yaml
    with:
      command: validate
      cli-version: "1.38.0"
      architecture: docs/architecture/calm/architecture.json
```

Check the latest published version:

```bash
npm view @finos/calm-cli version
```

## Permissions

The reusable workflow only declares `contents: read`. If your caller
needs to, e.g., open a PR with the rendered Mermaid output, set the
extra permissions in the **caller** workflow, not here.

## Artifacts

Setting `upload-artifact: true` with `output: <path>` uploads the output
path (file or directory) via `actions/upload-artifact@v4`. The artifact
fails the job if the output path is missing (`if-no-files-found: error`)
so template/docify errors don't silently produce empty artifacts.

---

## Testing

Three layers, wired up in
[`.github/workflows/calm-test.yaml`](./calm-test.yaml):

1. **Unit — [bats][bats-core] against `calm-args.sh`**
   [`.github/scripts/calm-args.bats`](../scripts/calm-args.bats) covers:
   - `CMD` required / unknown (exit code 2).
   - Every flag maps to the right CLI option (`-a`, `-p`, `-o`, `-t`,
     `-d`, `-b`, `-u`, `-s`, `-c`).
   - Scoped booleans (`--clear-output-directory`, `--scaffold`,
     `--strict`) are only emitted for the right sub-commands.
   - `-f <format>` only emitted for `validate`.
   - Verbose flag mapping.
   - `EXTRA` word-splits into trailing args.
   - Values with spaces are preserved as single args.
   - Minimal invocations produce the exact expected line count.
2. **Static — [shellcheck][shellcheck]** on `calm-args.sh`.
3. **Integration — reusable workflow end-to-end**
   - `integration-validate` calls the reusable workflow with
     `command: validate` against the checked-in architecture.
   - `integration-template` writes a tiny Handlebars template to
     `/tmp`, invokes `calm template`, and asserts output files exist.
   - `negative-unknown-command` asserts the script exits with code 2.
   - `negative-missing-cmd` asserts the script fails when `CMD` is
     unset.

Run the unit suite locally:

```bash
brew install bats-core          # or: apt-get install bats
bats .github/scripts/calm-args.bats
```

Run shellcheck:

```bash
brew install shellcheck
shellcheck .github/scripts/calm-args.sh
```

Add new flags to both the workflow **and** a bats case before merging.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `::error:: Unsupported command 'xxx'` | Typo in `command` input. | Must be one of `validate`, `generate`, `template`, `docify`. |
| `calm: not found` | Node cache didn't restore or npm install failed. | Re-run; if it persists, bump `node-version` or check npm registry availability. |
| Validation exits `0` with warnings | Warnings are informational by default. | Set `strict: true` to fail on warnings. |
| Empty artifact uploaded | Output path was empty at upload time. | The workflow uses `if-no-files-found: error`; check the previous step's logs. |
| Flag silently ignored | Scoped booleans (`scaffold`, `strict`, `format`) only apply to their owning sub-command. | Use them with the correct `command`. |
| Template bundle can't resolve URLs | Remote URLs referenced in templates are unreachable. | Supply `url-to-local-file-mapping` with local paths. |

[calm-cli]: https://github.com/finos/architecture-as-code/tree/main/cli
[calm-template-bundle]: https://github.com/finos/architecture-as-code/tree/main/cli#creating-a-template-bundle
[bats-core]: https://github.com/bats-core/bats-core
[shellcheck]: https://www.shellcheck.net/
