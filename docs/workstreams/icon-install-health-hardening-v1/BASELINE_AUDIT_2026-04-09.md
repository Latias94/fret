# Baseline Audit — 2026-04-09

Status: accepted baseline

Related:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`
- `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`

## Purpose

Freeze the assumptions-first baseline for the narrow install-health follow-on before claiming a new
contract split.

This note records what the codebase already settles:

- the broad icon contract lane is closed,
- explicit install seams already exist,
- helper fallback already exists,
- and the remaining question is the failure semantics of those already-shipped surfaces.

## Baseline findings

### 1) The surrounding install chain is not fallible today

The app/bootstrap registration path still flows through explicit closures that mutate `App`
directly rather than returning a `Result`.

Evidence:

- `ecosystem/fret-bootstrap/src/lib.rs`
- `crates/fret-launch/src/runner/desktop/runner/run.rs`

Consequence:

- this lane should not pretend that a broad fallible bootstrap redesign is a “small fix”.

### 2) Explicit install seams already define a real contract surface

The codebase now exposes explicit install seams for first-party and generated packs:

- `crate::app::install(...)` for pack crates,
- `BootstrapBuilder::register_icon_pack_contract(...)` for bootstrap,
- and the generated `src/app.rs` template for imported packs.

Evidence:

- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`

Consequence:

- failure semantics at these seams are part of the author-facing contract rather than an internal
  implementation detail.

### 3) Helper fallback currently owns a different problem

`freeze_or_default_with_context(...)` exists for helper-owned, non-fallible convenience paths such
as runtime preload/lazy freezing.

Evidence:

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`

Consequence:

- it should not be used as the justification for weakening explicit install semantics.

### 4) A broken entry can currently poison unrelated valid icons

If any icon resolution fails, the old helper fallback posture would replace the whole frozen
registry with an empty default registry.

Evidence:

- `ecosystem/fret-icons/src/lib.rs`

Consequence:

- helper fallback is currently too destructive for its intended “best-effort” role.

### 5) Pack metadata conflicts are too important for debug-only enforcement

Now that pack metadata is explicit and may be consumed by tooling, the same `pack_id` cannot map
to different metadata without becoming a real contract error.

Evidence:

- `ecosystem/fret-icons/src/lib.rs`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Consequence:

- release behavior must stop silently accepting conflicting metadata.

## Baseline decision

Treat the remaining work as a narrow failure-semantics split:

1. explicit install surfaces should be strict;
2. non-fallible helper surfaces should be best-effort and partial;
3. metadata conflict must be explicit;
4. and this lane should not widen into a general `Result` conversion of the setup/bootstrap stack.
