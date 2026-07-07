---
type: "Subagent Finding"
title: "Public example surface follow-up audit"
description: "Read-only explorer grouping for remaining public example raw-surface policy gaps."
timestamp: 2026-07-07T02:19:22Z
tags: ["fret", "examples", "surface-policy", "subagent", "audit"]
git_branch: "refactor/smoke-effects-surface-policy"
subagent_id: "019f3a51-a0a6-7510-bea7-baa90e9272ac"
---

# Finding

A read-only explorer reviewed the remaining `apps/fret-examples/src` raw/advanced public example
gaps and found that most remaining files are not default app-authoring surfaces. The important split
is:

- classify renderer, smoke, conformance, window interop, and IMUI/GenUI proof surfaces instead of
  teaching them as default examples;
- reserve real migration work for query/assets/selected plot or shadcn control demos where a public
  facade can replace the raw seams.

# Evidence

- `query_demo.rs`, `query_async_tokio_demo.rs`, and `async_playground_demo.rs` are default-facade
  candidates because their raw seams mostly come from lower-level text/helper generics.
- `assets_demo.rs` is a default-facade candidate, but its SVG GPU-ready path still depends on
  advanced services and likely needs an app-facing assets wrapper before migration.
- `date_picker_demo.rs`, `form_demo.rs`, `table_demo.rs`, and `sonner_demo.rs` are richer shadcn
  demos with manual runner/overlay/dispatch glue; migrate one file at a time or classify as
  advanced/manual until the facade is ready.
- `custom_effect_*`, `liquid_glass_demo.rs`, `postprocess_theme_demo.rs`, renderer/media/image
  demos, smoke/perf/conformance demos, window interop demos, and IMUI/GenUI proof surfaces should
  not be treated as first-contact authoring examples.

# Recommendation

Continue with small, homogeneous commits:

- renderer/media labs: `alpha_mode_demo.rs`, `drop_shadow_demo.rs`, `image_upload_demo.rs`;
- smoke/conformance harnesses: `cjk_conformance_demo.rs`, `emoji_conformance_demo.rs`,
  `ime_smoke_demo.rs`, `image_heavy_memory_demo.rs`, `text_heavy_memory_demo.rs`,
  `extras_marquee_perf_demo.rs`;
- true migration candidates after source-level reads: `assets_demo.rs`, `query_demo.rs`,
  `query_async_tokio_demo.rs`, `plot_image_demo.rs`, and `tags_demo.rs`.

# Disposition

The first follow-up slices have already classified custom-effect references, streaming import demos,
and the effects/first-frame smoke pair. Keep the remaining grouping as planning input, not as an
automatic allowlist; every future slice should still inspect source and add focused proof tests.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
