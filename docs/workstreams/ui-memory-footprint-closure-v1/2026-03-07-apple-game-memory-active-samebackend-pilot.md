# 2026-03-07 Apple Game Memory active same-backend pilot

## Goal

Test whether launch-mode `Game Memory` can become the next closure path for the active hello-world
plateau on the same backend.

This pilot intentionally starts with the smallest high-value pair:

- same-backend control: `wgpu_hello_world_control` with continuous redraw
- Fret active case: `hello_world_compare_demo` in `rerender-only` full mode

The question is whether `Game Memory` launch capture now exposes app-owned rows for:

- `metal-current-allocated-size`
- `metal-resource-allocations`
- `metal-io-surface-access`

If it does, we can scale the same path to `paint-model` / `layout-model`. If it does not, there is no
value in brute-forcing a larger active matrix yet.

## Setup

### Control

- Binary: `target/release/wgpu_hello_world_control`
- Env:
  - `FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_WIDTH=500`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_WINDOW_HEIGHT=500`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW=1`
  - `FRET_WGPU_HELLO_WORLD_CONTROL_EXIT_AFTER_SECS=7.25`

### Fret

- Binary: `target/release/hello_world_compare_demo`
- Env:
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH=500`
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT=500`
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=rerender-only`
  - `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS=7.25`

### Trace path

- Helper: `tools/capture_binary_xctrace.py`
- Template: `Game Memory`
- Record mode: `launch`
- Time limit: `8s`
- Finalization timeout: `180s`
- Summarizer: `tools/summarize_hello_world_compare_xctrace.py`
- Preset: `game-memory-attach`
- Export timeout: `60s` per schema

## Artifacts

Base dir:

- `target/diag/apple-game-memory-active-samebackend-20260307-r1/`

Control:

- `target/diag/apple-game-memory-active-samebackend-20260307-r1/control-continuous/summary.json`
- `target/diag/apple-game-memory-active-samebackend-20260307-r1/control-continuous/analysis/summary.json`

Fret:

- `target/diag/apple-game-memory-active-samebackend-20260307-r1/fret-rerender-only-full/summary.json`
- `target/diag/apple-game-memory-active-samebackend-20260307-r1/fret-rerender-only-full/analysis/summary.json`

## Trace health

Both launch captures produced full `.trace` bundles (`trace_complete_guess=true`), but `xctrace`
returned code `2` on this machine.

Interpretation:

- bundle completeness is good enough to continue analysis,
- but launch-mode `Game Memory` still looks operationally fragile.

## Findings

### 1. Launch-mode `Game Memory` still does not expose a useful active app-owned ledger here

Control continuous redraw:

- `metal-current-allocated-size`: `row_count = 0` (`fallback-direct-store-empty`)
- `metal-resource-allocations`: `row_count = 0` (`fallback-direct-store-empty`)
- `metal-io-surface-access`: `row_count = 33353`, but `pid_filter_match_count = 0`

Fret `rerender-only` full:

- `metal-current-allocated-size`: `row_count = 0` (`fallback-direct-store-empty`)
- `metal-resource-allocations`: `row_count = 0` (`fallback-direct-store-empty`)
- `metal-io-surface-access`: `row_count = 6597`, but `pid_filter_match_count = 0`
- `virtual-memory`: only `58` rows / `950272` bytes cumulative event size

So the strong active rows still do **not** yield a direct app-owned `Game Memory` explanation for the
large graphics bucket.

### 2. This is worse than the earlier startup-inclusive attach path for the app-visible metrics we care about

Earlier startup-inclusive attach captures were at least able to surface app-owned values such as:

- app-owned `metal-current-allocated-size`
- app-owned `metal-resource-allocations`
- app-owned `metal-io-surface-access` rows and drawable counts

This launch-mode active pilot does **not** recover those same app-visible rows reliably. So it is not
yet an upgrade over the earlier attach-based same-backend evidence.

### 3. Even the app-aware `metal-application-*` stores are header-only on this launch path

A follow-up probe on the Fret `rerender-only` launch trace checked:

- `metal-application-command-buffer-submissions`
- `metal-application-intervals`
- `metal-application-encoders-list`

All three timed out through `xctrace export`, and the direct-store fallback only recovered metadata:

- `target-pid = SINGLE`
- `row_count = 0`
- `fallback-direct-store-metadata`

So the problem is not merely that we picked the wrong first Game Memory schema. Even the most obvious
app-aware Metal application tables are not yet yielding active rows on this launch-mode path.

### 4. The pilot blocks a larger active `Game Memory` matrix for now

Because the most important app-owned schemas are either empty or not pid-attributed on this path,
running the full active matrix now would mostly spend time producing more non-closing bundles.

## Working interpretation

At the moment, launch-mode `Game Memory` is still not a stable attribution path for active same-
backend closure on this machine.

The likely failure modes are now narrower:

1. `Game Memory` launch mode is producing bundles that are complete enough structurally, but the
   app-owned stores we need are empty or unusable in the resulting export path.
2. The current direct-store fallback is still insufficient for some Game Memory launch schemas.
3. The right next Apple-side path may require either:
   - a stronger parser/join layer for Game Memory launch stores, or
   - a different instrument/template combination such as `Metal Resource Events`.

## Recommendation

Do **not** expand this into `paint-model` / `layout-model` yet.

Instead, the next useful step is one of:

1. Investigate why launch-mode `Game Memory` loses the app-owned rows that the earlier attach path
   could see.
2. Extend direct-store parsing/join logic around the Game Memory launch stores before rerunning a
   larger matrix.
3. Try `Metal Resource Events` (or another Apple template) on the same active pair and compare whether
   it preserves app-owned resource rows better than launch-mode `Game Memory`.
