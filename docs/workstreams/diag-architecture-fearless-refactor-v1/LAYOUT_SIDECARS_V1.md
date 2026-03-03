# Layout sidecars v1 (Taffy-focused, semantics-first)

Status: Draft (design note)

This document proposes a **bundle-scoped layout sidecar** format that makes layout regressions
explainable without requiring ad-hoc UI debug panels in demos.

The goal is to support a workflow:

1. run a scripted repro (bundle),
2. assert geometry using semantics selectors (`test_id`) (gate),
3. on failure (or on request), attach a layout sidecar that explains *why* geometry changed.

This is explicitly **tooling-oriented** and should remain **best-effort**: layout sidecars must not
turn successful runs into failures just because a dump is missing.

## Why sidecars (instead of widening the main bundle schema)

- Layout dumps are high-volume, fast-evolving, and platform-dependent.
- The primary bundle artifact should remain reasonably bounded and stable.
- Sidecars allow incremental adoption, opt-in capture, and separate clipping limits.

Long-term, runtime extensibility should prefer `debug.extensions` keys for *bounded* diagnostic
payloads. Layout dumps are typically too large for inline snapshot payloads, so a sidecar file is a
better fit.

## Scope

In scope for v1:

- Native-only, best-effort capture path (desktop first).
- Taffy subtree dump sufficient to answer “what constraints and computed sizes led to this bound?”.
- A stable file naming scheme and minimal metadata.

Out of scope for v1:

- Full “layout timeline” or correlation with profiling traces.
- Cross-engine support (non-Taffy) beyond a minimal `engine` discriminator field.
- A polished GUI viewer (raw JSON view is acceptable initially).

## Where the data comes from (today)

There is already a “deep debug escape hatch” in `fret-ui`:

- `crates/fret-ui/src/tree/layout/taffy_debug.rs`
- env wiring: `crates/fret-ui/src/runtime_config.rs`

v1 should reuse this path where possible (or extend it minimally), and surface it to `diag` as a
sidecar file tied to a script step or a post-run dump request.

## Contract: file naming and placement

Sidecars live next to the bundle artifact in the bundle directory.

Recommended naming (v1):

- `layout.taffy.v1.json`

Optional future variants (not required for v1):

- `layout.taffy.v1.<step_id>.json` (per-step dumps)
- `layout.taffy.v1.<selector_hash>.json` (focused subtree dumps)

## Contract: JSON shape (v1)

This is a minimal shape that keeps room for growth without breaking parsers.

Top-level:

- `schema_version`: `"v1"`
- `engine`: `"taffy"`
- `captured_at_unix_ms`: number
- `bundle_run_id`: string (optional, if known at capture time)
- `window`: object (optional; window id/bounds when captured)
- `clip`: object (required; indicates whether the dump was clipped)
- `roots`: array of layout root nodes (required; may be empty)

Clipping metadata:

- `clip.max_nodes`: number
- `clip.max_bytes`: number
- `clip.clipped_nodes`: number
- `clip.clipped_bytes`: number

Node shape (illustrative, not exhaustive):

- `node_id`: number (engine-local id)
- `parent_id`: number or null
- `debug_label`: string (optional; best-effort)
- `test_id`: string (optional; semantics selector correlation)
- `constraints`: object (min/max sizes, margins, etc.)
- `style`: object (engine-relevant style inputs; bounded)
- `layout`: object (computed size/position)
- `children`: array of `node_id` (optional; may omit if represented by `parent_id`)

Guidance:

- Prefer `test_id` linkage when available. When not available, keep linkage best-effort.
- Keep values numeric and explicit; avoid lossy formatted strings.
- Always write `clip` even when not clipped (`clipped_* = 0`).

## How `diag` should request/collect the sidecar

Two compatible request modes:

1. **Script meta capability** (preferred)
   - A script declares it can emit `layout_sidecar.taffy.v1` (capability).
   - Runtime emits the sidecar when the capability is present.

2. **Tooling flag** (escape hatch)
   - `fretboard diag run --dump-layout-sidecar` (name TBD).
   - Tooling forwards the request via env/runtime config to enable the dump.

In both cases:

- missing sidecar is a warning, not a failure,
- the script result should record whether a sidecar was requested and whether it was produced.

## Minimal viewer requirements

v1 viewer can be extremely small:

- render raw JSON (with search),
- allow filtering by `test_id`,
- show clipping metadata prominently.

DevTools GUI integration is a later milestone; CLI should still provide `diag query` affordances to
locate and open the sidecar path.

## Next steps

1. Decide the exact request surface (capability vs flag) and name it.
2. Implement a tiny capture path for `layout.taffy.v1.json` (native only) and ensure it is bounded.
3. Add one deterministic layout gate script that:
   - asserts semantics bounds,
   - on failure, points to the sidecar file as additional evidence.

