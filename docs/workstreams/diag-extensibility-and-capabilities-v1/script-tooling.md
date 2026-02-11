---
title: Diagnostics Extensibility + Capabilities v1 - Script Tooling
status: draft
date: 2026-02-10
scope: diagnostics, automation, scripts, tooling
---

# Diagnostics Extensibility + Capabilities v1 - Script Tooling

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`.

Goal: keep JSON scripts as the portable artifact, while providing tooling that makes scripts maintainable at scale
(reviewable diffs, CI-friendly checks, and shrink/minimize workflows).

## Commands (implemented)

### 1) `diag script normalize`

Parse a script JSON (v1/v2) and re-emit pretty JSON with stable formatting.

Command:

- `fretboard diag script normalize <script.json> [--write|--check]`

Behavior:

- Canonicalizes JSON key order recursively and emits `serde_json::to_string_pretty` output.
- Normalized output always ends with `\n` (and treats `\r\n` and `\n` as equivalent on input).

Outputs:

- normalized JSON on stdout (optional),
- or write-in-place with `--write`.
- `--check` exits with code 1 when the file is not normalized.

Why:

- stable diffs reduce review noise,
- scripts become safe to auto-generate.

### 2) `diag script validate`

Validate:

- JSON schema version is known,
- step variants and fields parse,
- unknown fields follow the protocol crate’s `serde` rules (strictness modes are TBD).

Command:

- `fretboard diag script validate <script.json>... [--check-out <path>] [--json]`

Outputs:

- `check.script_schema.json` evidence file for CI (default: under `--dir`, typically `target/fret-diag/`),
- clear human error messages (path + reason).

Exit code:

- exits with code 1 if any script fails to parse/validate.

### 3) `diag script lint`

Script-only lint (no app required):

- can infer required capabilities (e.g. screenshot steps),
- can flag discouraged patterns (raw coordinate injection when semantics selector exists),
- can flag missing metadata (optional, based on repo policy).

Command:

- `fretboard diag script lint <script.json>... [--check-out <path>] [--json]`

Outputs:

- `check.script_lint.json`.

Exit code:

- exits with code 1 if any script fails to parse/lint (tooling errors).

### 4) `diag script shrink` (delta debugging)

Given:

- a failing script,
- a deterministic runner invocation (`diag run --launch ...` or devtools-ws transport),

Perform a bounded search to produce a smaller script that still reproduces the failure.

Outputs:

- `repro.min.json`,
- a summary of removals (step indices),
- last failing bundle + trace, if available.

Constraints:

- must be opt-in (can be expensive),
- must respect `required_capabilities` and keep metadata intact.

## CI “generate + check” workflow

If typed templates are used (scriptgen):

- `scriptgen write <name> --out <tmp>` then `diag script normalize --check` against the committed script.

The goal is to treat scripts like compiled assets: authored or generated, but always reviewable and reproducible.

## Bundle lint (post-run sanity checks)

Bundle lint is intentionally separate from script-only lint: it validates the *captured evidence*.

Command:

- `fretboard diag lint <bundle_dir|bundle.json>`

Behavior:

- emits `check.lint.json` (default) next to `bundle.json`,
- exits with code 1 when error-level findings exist,
- supports `--warmup-frames <n>` to ignore early transient frames.

Initial checks (expand over time, but keep codes stable):

- `semantics.duplicate_test_id`,
- `semantics.active_descendant_missing`,
- focused/active out-of-window geometry sanity,
- empty/zero-size bounds on focused or `test_id` nodes.
