---
title: Diagnostics Fearless Refactor v1 (Agent Loop)
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, agent
---

# Agent-friendly diagnostics loop (recommended)

This document captures a **repeatable**, **agent-friendly** workflow for triaging UI diagnostics bundles.
The goal is to avoid “open a huge `bundle.json`” during first-pass triage by preferring small sidecars.

## Inputs

- A diagnostics bundle directory (or a `bundle.json` / `bundle.schema2.json` path).
- A `warmup_frames` value (defaults are fine; for scripted runs use the same value as the run/suite/perf command).

## Step 0: Preflight (self-heal missing artifacts)

Run doctor first. It is safe to run repeatedly.

- (Optional) Generate a plan file:
  - `fretboard diag agent <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames <n>`

- `fretboard diag doctor --check <bundle_dir> --warmup-frames <n>`
- If missing/invalid artifacts are reported:
  - `fretboard diag doctor --fix <bundle_dir> --warmup-frames <n>`
  - If `bundle.json` is large and you want a compact view for tooling/agents:
    - `fretboard diag doctor --fix-schema2 <bundle_dir> --warmup-frames <n>`

Note:

- When using `diag run` / `diag suite` / `diag perf`, `--bundle-doctor fix` will also attempt the schema2 repair (when `bundle.json` exists).

## Optional: Pack a small shareable zip (schema2-only)

If you want to share a repro bundle, prefer schema2-only zips (avoid shipping a huge `bundle.json`):

- Ensure schema2 exists (or let `--bundle-doctor fix` handle it when possible):
  - `fretboard diag doctor --fix-schema2 <bundle_dir> --warmup-frames <n>`
- Pack:
  - `fretboard diag pack <bundle_dir> --include-all --pack-schema2-only`

## Step 1: First-pass perf triage (no bundle.json materialization)

Use `triage --lite` (frames-index based) to identify the worst frames quickly:

- `fretboard diag triage --lite <bundle_dir> --warmup-frames <n> --metric total`
- Optional:
  - `--metric layout`
  - `--metric paint`

## Step 2: Perf “hotspots” fallback (slow frames report)

When the bundle artifact is too large to run JSON-size hotspots, use:

- `fretboard diag hotspots --lite <bundle_dir> --warmup-frames <n> --metric total`

Notes:

- `hotspots --lite` reports **slow frames** (perf hotspots), not JSON subtree byte hotspots.
- If you specifically need “JSON size hotspots”, use `hotspots` (non-lite) with `--force` on manageable bundle sizes.

## Step 3: Slice targeted evidence (small, shareable)

Once you have a candidate frame/window, slice only the relevant snapshot(s):

- `fretboard diag slice <bundle_dir> --test-id <test_id> --window <id> --frame-id <fid> --warmup-frames <n>`
- Or, if you have snapshot sequence instead:
  - `fretboard diag slice <bundle_dir> --test-id <test_id> --window <id> --snapshot-seq <seq> --warmup-frames <n>`

## Step 4: Generate an AI packet

To hand off to an AI agent, generate a compact packet directory:

- `fretboard diag ai-packet <bundle_dir> --warmup-frames <n>`
- Optional (if you already know a test_id):
  - `fretboard diag ai-packet <bundle_dir> --test-id <test_id> --warmup-frames <n>`

Expected contents include:

- `doctor.json`
- optional `bundle.schema2.json` (when present and within packet budget)
- `frames.index.json`
- `triage.lite.json` (frames-index derived)
- `hotspots.lite.json` (frames-index derived)
- `bundle.index.json` / `bundle.meta.json` / `test_ids.index.json`
- optional `slice.*.json` and `script.result.json` when present

## Step 5: Escalate only when needed

If the lite loop points to a specific failure, escalate to heavier artifacts only as needed:

- full `triage.json` (stats-heavy; may require materializing more of `bundle.json`)
- full `hotspots` (JSON subtree size hotspots)
- screenshot diffs / renderdoc / tracy traces
