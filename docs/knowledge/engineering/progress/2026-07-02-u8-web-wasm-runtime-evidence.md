---
type: Work Progress
title: U8 web wasm runtime evidence closeout
tags: fret,u8,wasm,diagnostics,text,renderer
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 web/wasm runtime evidence is now captured against the `code_editor_torture` UI Gallery page with
the `gallery-dev` feature enabled. The run proved that the web renderer can load the editor-grade
page, export a diagnostics bundle through devtools-ws, publish renderer text cache/glyph residency
resource snapshots, and pass the U8 web text budget gate.

# Changes

- `fret-ui-shadcn` wasm sidebar cookies now use `HtmlDocument` so `cookie()` and `set_cookie()` are available on wasm.
- `fret-render-wgpu` surface configuration no longer blocks the wasm main loop on WebGPU async error scopes.
- Renderer timing paths in text prepare/atlas flow use `fret_core::time::Instant` rather than `std::time::Instant`.
- The quad shadow WGSL path avoids a non-uniform early return before derivative calls.
- `ElementContext::timer_add_on_timer_for` rebuilds additive timer handler chains per frame instead of wrapping a previous-frame chain indefinitely.
- The web runner now treats renderer perf diagnostics as sufficient to publish `RendererTextPerfSnapshot`, so web bundles include `resource_caches.render_text`.

# Evidence

- Native U8 budget evidence:
  - `target/fret-diag-u8-text-budget-gate-native-r1/summary.json`
  - text-heavy: atlas live bytes `20971520` <= `50331648`; shape cache entries `1` <= `4096`; shape cache bytes `415952` <= `33554432`; atlas max pages `2` <= `2`.
  - code-editor: atlas live bytes `4194304` <= `16777216`; shape cache entries `635` <= `4096`; shape cache bytes `6491112` <= `16777216`; atlas max pages `2` <= `2`.
- Web/wasm runtime evidence:
  - bundle: `target/fret-diag-u8-web-export-code-editor-r3/1782959381479-bundle/bundle.json`
  - budget summary: `target/fret-diag-u8-web-budget-r3/summary.json`
  - selected page: `code_editor_torture`
  - `resource_caches` includes `render_text`, `render_text_fallback_policy`, and `render_text_font_trace`.
  - `render_text_shape_cache_entries=544`, `render_text_shape_cache_entry_limit=1024`, `render_text_shape_cache_bytes_estimate_total=3514264`.
  - `render_text_atlas_bytes_live_estimate_total=4194304`, `render_text_atlas_bytes_budget_estimate_total=37748736`.
  - mask/color/subpixel atlas `max_pages=1`; `renderer_text_atlas_evicted_pages=0`.
  - renderer text metrics present: `renderer_prepare_text_us`, atlas upload bytes, glyph instance upload bytes/count, text vertex upload bytes/count, and text op count.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-ui-gallery-web --target wasm32-unknown-unknown --features gallery-dev`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `cargo check -p fret-render-wgpu --lib`
- `cargo check -p fret-ui --lib`
- `cargo nextest run -p fret-ui additive_timer_handlers_rebuild_across_frames chained_timer_handlers_fall_through_until_one_handles chained_timer_handlers_short_circuit_after_first_handled --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu shaders_parse_as_wgsl path_shader_wgsl_validates_under_naga --no-fail-fast`
- `python3 tools/perf/diag_u8_text_budget_gate.py --skip-native --web-export-bundle target/fret-diag-u8-web-export-code-editor-r3/1782959381479-bundle/bundle.json --out-dir target/fret-diag-u8-web-budget-r3 --out-report target/fret-diag-u8-web-budget-r3/summary.json`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Commit this U8 web/wasm runtime evidence slice, then resume the remaining plan closeout audit from
the current worktree rather than rerunning the nav-search-based script path.
