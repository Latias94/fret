# Fret Launch + App Surface (Fearless Refactor v1) 鈥?`WinitRunnerConfig` Inventory

This note classifies the current `WinitRunnerConfig` surface into reviewable subdomains.

Goal:

- make the config shape easier to reason about,
- separate app-facing defaults from backend-heavy tuning,
- create a low-risk documentation-first step before any API reshaping.

## Why this note exists

`WinitRunnerConfig` is currently a valid and useful public type, but it carries several different kinds of concerns at once:

- main-window defaults,
- new-window defaults,
- input/event-loop tuning,
- file/dialog safety budgets,
- renderer/cache tuning,
- text/bootstrap concerns,
- web-specific setup,
- streaming/media pipeline tuning,
- GPU initialization strategy.

That breadth is one of the reasons the current launch surface feels more runner-centric than app-centric.

This document does **not** propose a breaking redesign yet. It only records the current field groups and the likely public-story direction.

## Recommended field groups

### Group A 鈥?App/window defaults

These are the fields a general app author is most likely to care about directly.

- `main_window_title`
- `main_window_size`
- `main_window_position`
- `main_window_style`
- `default_window_title`
- `default_window_size`
- `default_window_position`
- `new_window_anchor_offset`
- `exit_on_main_window_close`

Recommended public-story status:

- **Primary app-facing knobs**

These belong in beginner-facing docs and builder examples.

### Group B 鈥?Input + frame cadence defaults

- `wheel_line_delta_px`
- `wheel_pixel_delta_scale`
- `frame_interval`

Recommended public-story status:

- **App-facing, but advanced-default-adjacent**

These are still reasonable public knobs, but most app authors should be fine with defaults.

### Group C 鈥?Drag/drop + file-dialog safety budgets

- `external_drop_max_total_bytes`
- `external_drop_max_file_bytes`
- `external_drop_max_files`
- `file_dialog_max_total_bytes`
- `file_dialog_max_file_bytes`
- `file_dialog_max_files`

Recommended public-story status:

- **Advanced app-facing safety knobs**

These are not renderer internals, but they are operational/safety tuning rather than first-hour authoring concerns.

### Group D 鈥?Renderer/cache tuning

- `clear_color`
- `svg_raster_budget_bytes`
- `renderer_intermediate_budget_bytes`
- `path_msaa_samples`

Recommended public-story status:

- **Advanced backend/perf tuning**

These should remain available, but they should not dominate the high-level launch story.

### Group E 鈥?Accessibility + text bootstrap

- `accessibility_enabled`
- `text_font_families`

Recommended public-story status:

- **Stable app-facing configuration**

These are meaningful to real apps and should stay visible, but they can usually be documented as optional overrides rather than mandatory setup.

### Group F 鈥?GPU initialization strategy

- `wgpu_init`

Related type:

- `WgpuInit::{CreateDefault, Provided, Factory}`

Recommended public-story status:

- **Stable specialized contract**

This is one of Fret's important advanced host-integration seams. It is not a beginner feature, but it is absolutely a real framework capability and should remain first-class.

### Group G 鈥?Web-specific launch setup

- `web_canvas_id`

Recommended public-story status:

- **Platform-specific specialized config**

This should remain public, but it should not shape the mental model for desktop callers.

### Group H 鈥?Streaming/media pipeline tuning

- `streaming_upload_budget_bytes_per_frame`
- `streaming_staging_budget_bytes`
- `streaming_perf_snapshot_enabled`
- `streaming_update_ack_enabled`
- `streaming_nv12_gpu_convert_enabled`

Recommended public-story status:

- **Specialized backend/media tuning**

These are legitimate features, but they are clearly not the same tier of concern as window title/size or close behavior.

## Recommended interpretation

### What should stay easy to discover

If a user is building a normal desktop app, the easiest-to-discover knobs should be roughly:

- window title/size/position/style,
- close behavior,
- accessibility,
- font family overrides,
- perhaps frame cadence when animation-heavy behavior is relevant.

### What should remain public but de-emphasized

The following should stay available without being front-and-center in beginner-facing docs:

- file/drop safety budgets,
- renderer/cache budgets,
- web canvas ID,
- streaming/media tuning,
- GPU init strategy.

### Why `WgpuInit` is different from other advanced fields

Most advanced config fields are tuning knobs.

`WgpuInit` is different: it is part of the framework's **integration contract**. It enables:

- host-owned GPU setup,
- embedding and interop,
- custom initialization flows that general UI toolkits often struggle to support cleanly.

That makes it a good candidate to stay first-class even if the rest of the config surface gets more strongly grouped later.

## Low-risk next moves

### Option 1 鈥?Documentation-only grouping (lowest risk)

- Keep `WinitRunnerConfig` exactly as-is.
- Document the groups above in launch docs and builder docs.
- Teach only Group A / E by default.

### Option 2 鈥?Helper-layer grouping (still low risk)

- Keep the struct shape as-is.
- Add higher-level builder helpers in `fret` / `fret-bootstrap` that expose common app-facing settings.
- Leave the full struct for advanced callers.

### Option 3 鈥?Future nested config split (higher risk)

- Introduce grouped public config sections later, only after we know the stable domain boundaries.

This should be a later step, not the first response to the current curation debt.

## Recommended next decision

Short term:

- treat `WinitRunnerConfig` as **stable but over-broad**,
- document the field groups,
- do not rush into a breaking shape change.

Medium term:

- use `fret` and `fret-bootstrap` to present a narrower app-author story on top of the full config object.

## Evidence anchors

- Config definition: `crates/fret-launch/src/runner/common/config.rs`
- Builder exposure in `fret`: `ecosystem/fret/src/lib.rs`
- Builder exposure in bootstrap: `ecosystem/fret-bootstrap/src/lib.rs`
- Related workstream: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`
