# Baseline Audit — 2026-04-09

Status: accepted baseline

Related:

- `docs/workstreams/iconify-presentation-defaults-report-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fretboard/src/icons/suggest.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

## Findings

### 1. The shipped helper already makes the right decision, but the rationale is ephemeral

The existing helper writes an explicit `presentation-defaults.json` and prints a short stdout
summary. That proves the advisory suggestion flow, but stdout is not a durable review artifact.

### 2. The config file cannot explain why it was suggested

`presentation-defaults.json` is intentionally small and generator-owned. It should not be inflated
with provenance facts or helper-specific explanations just to satisfy review ergonomics.

### 3. The missing artifact belongs next to the helper, not in import/generator contracts

The needed follow-on is a second explicit helper artifact derived from the same provenance input.
It should stay optional and live in `fretboard` so the stable generator contract remains unchanged.

### 4. Path conflicts need to fail before any write

Once the helper can write two output artifacts, it must also guard against self-overwrite between
provenance, config, and report paths.

## Baseline verdict

Treat this as a narrow follow-on for reviewability only:

- keep the existing suggestion output untouched,
- add one optional versioned report artifact,
- and prove that the extra artifact does not become implicit import behavior.
