# Setter Idempotency Contract (Perf Safety) — v1

This note documents a lightweight contract for "handle-style" setters that may be invoked from
declarative render code every frame.

## Problem

In a declarative UI, element code often re-applies state to handles during render (e.g.
`handle.set_language(...)`, `handle.set_soft_wrap_cols(...)`).

If a setter performs work even when the value is unchanged (cache resets, epoch bumps, allocations),
the cost becomes **per-frame**, which can turn into a "mystery hitch" that looks like a paint/layout
hotspot rather than a logic bug.

Concrete example (evidence in the perf log):

- `CodeEditorHandle::set_language(...)` was called during render even when the language did not
  change, which reset syntax/rich caches every frame and forced re-highlighting work during live
  resize.
  - Fix: make the setter idempotent.
  - Result: `ui-code-editor-resize-probes` worst frame `~42ms → ~16ms` (commit `1778ba563`).

## Contract

For any `handle.set_*` method that can be called from render:

1. **Idempotent for the same value**
   - If the new value is equivalent to the current value, return early.
2. **No cache resets / epoch bumps on no-op**
   - "Reset" counters should reflect real changes, not repeated re-application.
3. **If you need an imperative side-effect, provide an explicit method**
   - Example patterns:
     - `handle.reset_*_cache()`
     - `handle.invalidate_*()`
   - Avoid "hidden" side effects in `set_*` when the value is unchanged.
4. **Add a regression test for each high-risk setter**
   - Tests should assert that calling the setter twice with the same value does not bump epochs and
     does not increment reset counters.

## Implementation patterns (Rust)

### Option / scalar values

- Normalize first (`filter(|v| *v > 0)` etc), then compare, then apply.

### Vec/Span payloads

If the handle stores an `Arc<[T]>`:

- If the new `Vec<T>` is empty:
  - No-op if the key does not exist.
  - Otherwise remove it.
- If non-empty:
  - No-op if `existing.as_ref() == spans.as_slice()`.
  - Otherwise store `Arc::from(spans)`.

This keeps epoch/cache invalidation tied to actual changes, even if render calls the setter every
frame.

## Diagnostics & evidence

Preferred evidence loop (commit-addressable):

1. Run a probe/gate (`tools/perf/*`) and capture the worst bundle(s).
2. Use `fretboard diag stats <bundle.json> --sort time --top N` to attribute the worst frame.
3. If the hotspot is inside a Canvas/paint path, add phase attribution or cache/reset counters to
   `app_snapshot` to confirm the underlying cause.

## Audit ledger (v1)

| Surface | Setter(s) | Status | Evidence |
|---|---|---:|---|
| `CodeEditorHandle` | `set_language` | Done | `perf(fret-code-editor): make set_language idempotent` (commit `1778ba563`) + perf log entry `2026-02-09 12:34:16` |
| `CodeEditorHandle` | `set_line_folds`, `set_line_inlays` | Done | `perf(fret-code-editor): make fold/inlay setters idempotent` (commit `007006b28`) + perf log entry `2026-02-09 13:46:46` |
| `TextArea` | `set_text` | Done | `perf(fret-ui): make TextArea::set_text idempotent` (commit TBD) |

## Next

- Extend this audit to other handle-style surfaces that are configured from render in the UI gallery
  (docking, viewport tooling, etc.).
- Consider adding a short "render-time setter safety" guideline in the main perf workstream doc once
  we have 2–3 surfaces audited.
