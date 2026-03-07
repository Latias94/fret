# 2026-03-07 Apple Metal Resource Events active same-backend paired capture

## Goal

Test whether launch-mode `Metal Resource Events` can become the first Apple-side attribution path that
stably exposes **app-owned active Metal rows** for the same-backend hello-world plateau.

This investigation now covers two layers of the question:

1. same-backend control versus Fret full-scene active modes
2. a focused `present-only` full/empty split to separate framework floor from hello-world content at
   the same app-visible Metal layer

The main questions are:

1. Does this path expose app-visible live Metal allocation rows?
2. If yes, how much of the active plateau does that ledger actually explain?
3. Is the large active plateau already present in a pure present loop, or does it require real per-frame
   rerender/layout work?
4. How much of the remaining app-visible Metal plateau is content versus framework floor?

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
- Shared env:
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH=500`
  - `FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT=500`
  - `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS=7.25`
- Full-scene active modes:
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=present-only`
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=rerender-only`
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=paint-model`
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=layout-model`
- Empty present-only follow-up:
  - `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=present-only`
  - `FRET_HELLO_WORLD_COMPARE_NO_TEXT=1`
  - `FRET_HELLO_WORLD_COMPARE_NO_SWATCHES=1`

### Trace path

- Helper: `tools/capture_binary_xctrace.py`
- Instrument: `metal-resource-events`
- Record mode: `launch`
- Time limit: `8s`
- Finalization timeout: `90s`
- Summarizer: `tools/summarize_hello_world_compare_xctrace.py`
- Requested schemas:
  - `metal-current-allocated-size`
  - `metal-resource-allocations`
  - `metal-kernel-resource-allocations`
  - `metal-residency-set-usage-event`
  - `metal-residency-set-resource-event`
  - `metal-residency-set-interval`
  - `metal-wired-sysmem-level-interval`

## Artifacts

Base dir:

- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/`

Control:

- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/control-continuous/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/control-continuous/analysis/summary.json`

Fret full scene:

- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-present-only-full/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-present-only-full/analysis/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-rerender-only-full/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-rerender-only-full/analysis/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-paint-model-full/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-paint-model-full/analysis/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-layout-model-full/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-layout-model-full/analysis/summary.json`

Fret empty follow-up:

- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-present-only-empty/summary.json`
- `target/diag/apple-metal-resource-events-active-samebackend-20260307-r1/fret-present-only-empty/analysis/summary.json`

## Trace health

All launch captures on this path are operationally healthy:

- `trace_returncode = 0`
- `status = recorded`
- `trace_complete_guess = true`

That already makes this capture path materially stronger than the earlier launch-mode `Game Memory`
pilot on this machine.

## Findings

### 1. `Metal Resource Events` is the first active same-backend Apple path here that exposes a usable app-owned ledger

Unlike the earlier `VM+Metal` and launch-mode `Game Memory` runs, this instrument exports stable,
process-attributed rows for:

- `metal-current-allocated-size`
- `metal-resource-allocations`
- `metal-residency-set-resource-event`
- `metal-residency-set-interval`

So this path finally gives a directly app-visible Metal ledger for the active same-backend pair.

### 2. The pure present loop already reaches the same full-scene live Metal plateau

| run | current allocated size (`last`) | current allocated size (`max`) | residency-set allocation sum |
| --- | ---: | ---: | ---: |
| control continuous | `9.00 MiB` | `9.62 MiB` | `11.72 MiB` |
| Fret `present-only` empty | `38.33 MiB` | `38.33 MiB` | `23.44 MiB` |
| Fret `present-only` full | `42.45 MiB` | `42.45 MiB` | `23.44 MiB` |
| Fret `rerender-only` full | `42.45 MiB` | `42.45 MiB` | `23.44 MiB` |
| Fret `paint-model` full | `42.45 MiB` | `42.45 MiB` | `23.44 MiB` |
| Fret `layout-model` full | `42.45 MiB` | `42.45 MiB` | `23.44 MiB` |

What this now says very directly:

- control sits near a `~9–10 MiB` live Metal level,
- `present-only full` already reaches the same `42.45 MiB` plateau as `rerender-only` / `paint-model` /
  `layout-model`,
- so the large live Metal floor does **not** require real per-frame rerender/layout to appear,
- the app-visible live Metal delta versus control is about `+33 MiB` on the full scene even in the pure
  present loop.

This is the clearest Apple-side confirmation so far that the active floor is established before real
per-frame declarative work is added.

### 3. Hello-world content only adds about `4.12 MiB` on top of that present-only floor

The new full/empty split shows:

- `present-only empty`: `38.33 MiB`
- `present-only full`: `42.45 MiB`
- content delta at this app-visible live Metal layer: about `+4.12 MiB`

So most of the live Metal plateau is framework/active-loop floor rather than visible hello-world
content.

This lines up closely with the existing internal Metal counters, which already suggested that the
content delta was single-digit MiB.

### 4. The full-scene active modes are effectively identical at the live Metal layer

`present-only full`, `rerender-only full`, `paint-model full`, and `layout-model full` differ slightly in
allocation-event counts, but not in the steady live allocation plateau.

That strongly supports the interpretation that the major active cost is a persistent active floor, not
an additional large residency increase caused by paint-vs-layout work itself.

### 5. Buffer churn remains large even in `present-only`, so it is not just a rerender/layout artifact

`metal-resource-allocations` shows:

- control continuous
  - `row_count = 28`
  - `resource_size_bytes_sum = 12.30 MiB`
  - `resource_types = { Buffer: 26, Texture: 2 }`
- Fret `present-only` empty
  - `row_count = 5133`
  - `resource_size_bytes_sum = 674.75 MiB`
  - `resource_types = { Buffer: 5127, Texture: 6 }`
- Fret `present-only` full
  - `row_count = 8492`
  - `resource_size_bytes_sum = 1.07 GiB`
  - `resource_types = { Buffer: 8485, Texture: 7 }`
- Fret `rerender-only` full
  - `row_count = 8372`
  - `resource_size_bytes_sum = 1.06 GiB`
  - `resource_types = { Buffer: 8365, Texture: 7 }`
- Fret `paint-model` full
  - `row_count = 8552`
  - `resource_size_bytes_sum = 1.08 GiB`
  - `resource_types = { Buffer: 8545, Texture: 7 }`
- Fret `layout-model` full
  - `row_count = 8542`
  - `resource_size_bytes_sum = 1.08 GiB`
  - `resource_types = { Buffer: 8535, Texture: 7 }`

This is valuable, but it must be interpreted carefully:

- these sums are **allocation-event payload totals**, not steady live residency,
- they do suggest substantial per-frame or per-present buffer churn,
- and the fact that `present-only` already shows large Buffer churn makes it a stronger optimization
  lead than before.

The first label-level clue is already surprisingly specific:

- control only shows a tiny number of staging labels,
- `present-only empty` is already dominated by `"(wgpu internal) Staging  ( 128.00 KiB ,  Shared )" = 5062` rows,
- `present-only full` raises that to `8413` rows,
- the other full active modes stay in the same band (`8300–8475` rows),
- and the event stream remains pinned to the main thread in these runs.

The new backtrace clustering makes that lead much sharper:

- the app/user-side boundary for the staging rows is overwhelmingly `wgpu_hal::metal::Device::create_buffer`,
- for Fret runs, the same staging label clusters primarily under `fret_render_wgpu::renderer::render_scene::execute`,
- the second-largest steady contributor is `fret_render_wgpu::renderer::uniform_resources::UniformResources::write_viewport_uniforms_into`,
- full-scene runs add only a small extra tail from `fret_render_wgpu::text::GlyphAtlas::flush_uploads`,
- and `present-only empty` still shows the same `render_scene_execute + viewport uniforms` pattern even without text/swatches.

So the first optimization suspect is no longer a vague “some buffers”: it is now much more likely the
`wgpu` staging/upload path driven by Fret's steady render/present path—especially scene execution and
viewport uniform writes—rather than the small number of visible `CAMetalLayer` drawable texture rows or
hello-world text content itself.

This should still be treated as a follow-up optimization lead rather than a closed steady-state
attribution, but it is now a much sharper lead than before.

A follow-up bounded runtime experiment on release head now sharpens this even further: `docs/workstreams/ui-memory-footprint-closure-v1/2026-03-07-present-uniform-dedupe-release.md` shows that deduping redundant viewport/clip/mask/render-space uploads can collapse the post-attach `Metal Resource Events` row stream to zero-row header-only stores on `present-only empty`, yet the steady memory floor barely moves. So the staging lead is real, but it explains **churn**, not the main steady residency bucket.

### 6. Residency-set rows do show more committed drawable-like texture volume in Fret, but still not enough to explain the full graphics gap

`metal-residency-set-resource-event` reports:

- control: `3` texture rows, `allocation_size_bytes_sum = 11.72 MiB`
- Fret active rows: `6` texture rows, `allocation_size_bytes_sum = 23.44 MiB`

`metal-residency-set-interval` also shows the committed set moving from roughly:

- control: up to `2 allocations / 7.84 MiB`
- Fret: up to `3 allocations / 11.77 MiB`

So there is a real app-visible residency increase in drawable-like texture commitment, but it is still
far smaller than the full active graphics gap seen in external macOS memory buckets.

### 7. A large residual still remains outside the app-visible current-allocation ledger

In the same-backend active baseline, the full-scene plateau against the control is still roughly
`~+104–109 MiB` in macOS-visible graphics, depending on the active mode.

This trace only exposes about:

- `+29.33 MiB` of live app-visible Metal delta for `present-only empty` versus control
- `+33.45 MiB` of live app-visible Metal delta for the full scene versus control

So even after the best Apple path so far, there is still roughly `~60–75 MiB` of active graphics cost
outside the app-visible `metal-current-allocated-size` ledger.

That remaining bucket still looks more like some combination of:

- driver/private residency,
- swapchain / surface bookkeeping not visible through this store,
- OS reservation / WindowServer-side accounting,
- or another Apple bucket we still are not decoding.

### 8. `metal-wired-sysmem-level-interval` is informative globally, but not app-attributable

This schema exported correctly, but it has no process column, so it remains a GPU-level signal rather
than an app-owned ledger. It can help as supporting context, but it does not close the residual.

## Working interpretation

This is now the **best Apple-side active same-backend attribution path** we have on this machine.

It closes several important parts of the story:

- Fret active mode really does maintain a larger app-visible live Metal plateau than the same-backend
  `wgpu` control,
- that plateau is already present in `present-only`,
- the full-scene plateau is about `42.45 MiB` versus `~9–10 MiB` for the control,
- the empty-scene plateau is still `38.33 MiB`,
- and the visible hello-world content only adds about `4.12 MiB` at this layer.

But it still does **not** close the entire graphics bucket:

- app-visible live Metal explains only part of the active plateau,
- the remaining residual is still too large to hand-wave away,
- and the earlier `VM+Metal` evidence already argues against a simple “Fret-only compositor surface
  explosion” explanation.

So the current best interpretation is:

1. Fret has a real active app-owned Metal floor of about `+29–33 MiB` versus the control on this
   scene.
2. That floor is already established in the pure present loop.
3. The hello-world content itself contributes only a small single-digit increment at this layer.
4. There is still another large active graphics bucket outside that app-visible Metal floor.
5. That remaining bucket is more likely driver/private/OS-side accounting than a purely app-visible
   live-resource ledger we simply forgot to print.

## Recommendation

Do **not** spend the next iteration re-proving the active mode split again.

The highest-value follow-ups now are:

1. Cluster `metal-resource-allocations` by label / backtrace so we can identify the dominant Buffer
   churn sources that already appear in `present-only`.
2. Keep a second Apple-side path focused on the residual outside `metal-current-allocated-size`
   (driver/private/WindowServer/OS reservation), because that bucket is still larger than the app-
   visible Metal delta we have now explained.
3. Only add more active matrix rows if they answer a new question that the current full/empty
   `present-only` split does not already answer.
