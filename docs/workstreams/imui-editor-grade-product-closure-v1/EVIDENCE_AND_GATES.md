# ImUi Editor-Grade Product Closure v1 - Evidence & Gates

Goal: keep the editor-grade maturity plan tied to real proof surfaces, not just strategy prose.

## Docking drop resolve target owner split - 2026-06-03

This maintenance slice keeps docking drop target resolution separate from drop intent, effect, and
diagnostics orchestration:

- `ecosystem/fret-docking/src/dock/drop_resolve/target.rs` owns dock drop target resolution,
  including tab-bar insert targets, inner/outer hint-pad targets, float/empty-space targets,
  previous-hover latching, inverted docking, and policy allow checks.
- `ecosystem/fret-docking/src/dock/drop_resolve/floating_hit.rs` remains the floating-window hit
  owner used by target resolution.
- `ecosystem/fret-docking/src/dock/drop_resolve.rs` keeps drop intent, apply, and diagnostics
  orchestration.
- Evidence anchor: Docking drop resolve target owner split - 2026-06-03.
- Evidence anchor: ecosystem/fret-docking/src/dock/drop_resolve/target.rs.
- Evidence anchor: dock drop target resolution.
- Evidence anchor: Drop resolve root keeps drop intent, apply, and diagnostics orchestration.
- Public docking APIs, floating/outside-window target classification, tab-bar insert resolution,
  hint picking, previous-hover latching, policy checks, drop intents, effect projection, and
  diagnostics payloads remain unchanged.
- `tools/gate_imui_workstream_source.py` now source-checks the split so target-resolution helpers
  cannot drift back into `drop_resolve.rs`.

Fresh gates:

- `cargo fmt -p fret-docking` - passed.
- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking floating_title_bar_drag center_drop tab_drop drop_hint --no-fail-fast` - passed.
- `cargo nextest run -p fret-docking drags_floating_title_bar --no-fail-fast` - passed.
- `cargo fmt -p fret-docking -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Docking drop resolve floating hit owner split - 2026-06-03

This maintenance slice keeps docking drop resolution moving toward editor-grade multi-window hand
feel by separating floating-window hit tests from drop target, intent, apply, and diagnostics
orchestration:

- `ecosystem/fret-docking/src/dock/drop_resolve/floating_hit.rs` owns the floating-window close,
  title-bar, and body hit classification used by drop target resolution.
- `ecosystem/fret-docking/src/dock/drop_resolve/floating_hit.rs` owns the layout-context projection
  that maps pointer position into floating-window inner content bounds.
- `ecosystem/fret-docking/src/dock/drop_resolve.rs` keeps target, intent, apply, and diagnostics
  orchestration.
- Evidence anchor: Docking drop resolve floating hit owner split - 2026-06-03.
- Evidence anchor: ecosystem/fret-docking/src/dock/drop_resolve/floating_hit.rs.
- Evidence anchor: floating-window hit tests used by drop target resolution.
- Evidence anchor: Drop resolve root keeps target, intent, apply, and diagnostics orchestration.
- Public docking APIs, floating title-bar center-drop projection, tab-bar insert resolution,
  policy allow checks, drop intents, effect projection, and diagnostics payloads remain unchanged.
- `tools/gate_imui_workstream_source.py` now source-checks the split so floating hit/context
  helpers cannot drift back into `drop_resolve.rs`.

Fresh gates:

- `cargo fmt -p fret-docking` - passed.
- `cargo check -p fret-docking` - passed.
- `cargo nextest run -p fret-docking floating_title_bar_drag center_drop --no-fail-fast` - passed.
- `cargo nextest run -p fret-docking drags_floating_title_bar --no-fail-fast` - passed.
- `cargo fmt -p fret-docking -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative area/shaded/stems props builder owner split - 2026-06-03

This maintenance slice turns the remaining props root into a pure builder-owner facade while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/area.rs` owns `AreaPlotPanelProps` construction plus
  output/state/style/axis-label/axis-scale/step-mode builder methods.
- `ecosystem/fret-plot/src/declarative/props/shaded.rs` owns `ShadedPlotPanelProps` construction
  plus output/state/style/axis-label/axis-scale/step-mode builder methods.
- `ecosystem/fret-plot/src/declarative/props/stems.rs` owns `StemsPlotPanelProps` construction
  plus output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props.rs` is now a pure builder-owner facade.
- Evidence anchor: Props root is now a pure builder-owner facade.
- Evidence anchor: Fret Plot declarative area props builder owner split - 2026-06-03.
- Evidence anchor: AreaPlotPanelProps builder owner.
- Evidence anchor: Fret Plot declarative shaded props builder owner split - 2026-06-03.
- Evidence anchor: ShadedPlotPanelProps builder owner.
- Evidence anchor: Fret Plot declarative stems props builder owner split - 2026-06-03.
- Evidence anchor: StemsPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- `tools/gate_imui_workstream_source.py` now source-checks the split so area/shaded/stems builder
  methods cannot drift back into `props.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot area_plot_panel shaded_plot_panel stems_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative histogram2d props builder owner split - 2026-06-03

This maintenance slice keeps histogram2d plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior and the histogram2d default colorbar:

- `ecosystem/fret-plot/src/declarative/props/histogram2d.rs` owns `Histogram2DPlotPanelProps`
  construction plus output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props/histogram2d.rs` preserves `style.heatmap_show_colorbar = true`.
- `props.rs` declares the histogram2d, heatmap, candlestick, bars, histogram, error-bars, and line
  builder owners, re-exports public prop records, and keeps remaining plot prop builders.
- Evidence anchor: builder methods for area remain in the props root.
- Evidence anchor: Props root declares histogram2d builder owner.
- Evidence anchor: Histogram2DPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The histogram2d props builder owner stays non-histogram2d-props-free, authoring-free,
  retained-free, paint-free, event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so histogram2d builder
  methods cannot drift back into `props.rs` and other plot prop builders cannot drift into
  `props/histogram2d.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot histogram2d_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative heatmap props builder owner split - 2026-06-03

This maintenance slice keeps heatmap plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior and the heatmap default colorbar:

- `ecosystem/fret-plot/src/declarative/props/heatmap.rs` owns `HeatmapPlotPanelProps`
  construction plus output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props/heatmap.rs` preserves `style.heatmap_show_colorbar = true`.
- `props.rs` declares the heatmap, candlestick, bars, histogram, error-bars, and line builder
  owners, re-exports public prop records, and keeps remaining plot prop builders plus the
  histogram2d colorbar default.
- Evidence anchor: builder methods for histogram2d remain in the props root.
- Evidence anchor: Props root declares heatmap builder owner.
- Evidence anchor: HeatmapPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The heatmap props builder owner stays non-heatmap-props-free, authoring-free, retained-free,
  paint-free, event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so heatmap builder methods
  cannot drift back into `props.rs` and other plot prop builders cannot drift into
  `props/heatmap.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot heatmap_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative candlestick props builder owner split - 2026-06-02

This maintenance slice keeps candlestick plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/candlestick.rs` owns
  `CandlestickPlotPanelProps` construction plus output/state/style/axis-label/axis-scale/step-mode
  builder methods.
- `props.rs` declares the candlestick, bars, histogram, error-bars, and line builder owners,
  re-exports public prop records, and keeps remaining plot prop builders plus heatmap colorbar
  defaults.
- Evidence anchor: builder methods for heatmap remain in the props root.
- Evidence anchor: Props root declares candlestick builder owner.
- Evidence anchor: CandlestickPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The candlestick props builder owner stays non-candlestick-props-free, authoring-free,
  retained-free, paint-free, event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so candlestick builder methods
  cannot drift back into `props.rs` and other plot prop builders cannot drift into
  `props/candlestick.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot candlestick_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative bars props builder owner split - 2026-06-02

This maintenance slice keeps bars plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/bars.rs` owns `BarsPlotPanelProps` construction plus
  output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props.rs` declares the bars, histogram, error-bars, and line builder owners, re-exports public
  prop records, and keeps remaining plot prop builders plus heatmap colorbar defaults.
- Evidence anchor: builder methods for candlestick remain in the props root.
- Evidence anchor: Props root declares bars builder owner.
- Evidence anchor: BarsPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The bars props builder owner stays non-bars-props-free, authoring-free, retained-free, paint-free,
  event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so bars builder methods cannot
  drift back into `props.rs` and other plot prop builders cannot drift into `props/bars.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot bars_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative histogram props builder owner split - 2026-06-02

This maintenance slice keeps histogram plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/histogram.rs` owns `HistogramPlotPanelProps`
  construction plus output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props.rs` declares the histogram, error-bars, and line builder owners, re-exports public prop
  records, and keeps remaining plot prop builders plus heatmap colorbar defaults.
- Evidence anchor: builder methods for bars remain in the props root.
- Evidence anchor: Props root declares histogram builder owner.
- Evidence anchor: HistogramPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The histogram props builder owner stays non-histogram-props-free, authoring-free, retained-free,
  paint-free, event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so histogram builder methods
  cannot drift back into `props.rs` and other plot prop builders cannot drift into
  `props/histogram.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot histogram_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative error-bars props builder owner split - 2026-06-02

This maintenance slice keeps error-bars plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/error_bars.rs` owns
  `ErrorBarsPlotPanelProps` construction plus output/state/style/axis-label/axis-scale/step-mode
  builder methods.
- `props.rs` declares the error-bars and line builder owners, re-exports public prop records, and
  keeps remaining plot prop builders plus heatmap colorbar defaults.
- Evidence anchor: builder methods for histogram remain in the props root.
- Evidence anchor: Props root declares error-bars builder owner.
- Evidence anchor: ErrorBarsPlotPanelProps builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The error-bars props builder owner stays non-error-bars-props-free, authoring-free,
  retained-free, paint-free, event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so error-bars builder methods
  cannot drift back into `props.rs` and other plot prop builders cannot drift into
  `props/error_bars.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot error_bars_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative line props builder owner split - 2026-06-02

This maintenance slice keeps line plot prop construction out of the shared props root while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/line.rs` owns `LinePlotPanelProps` construction plus
  output/state/style/axis-label/axis-scale/step-mode builder methods.
- `props.rs` declares the line builder owner, re-exports public prop records, and keeps remaining
  plot prop builders plus heatmap colorbar defaults.
- Evidence anchor: builder methods for error-bars remain in the props root.
- Evidence anchor: Props root declares line builder owner.
- Evidence anchor: LinePlotPanelProps builder owner.
- Evidence anchor: line plot props builder owner.
- Evidence anchor: Props root declares the private line builder owner.
- Public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners, output
  publication, and plot model projection remain unchanged.
- The line props builder owner stays non-line-props-free, authoring-free, retained-free, paint-free,
  event-free, and output-publication-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so line builder methods cannot
  drift back into `props.rs` and other plot prop builders cannot drift into `props/line.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot line_plot_panel_paints_seeded_line_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_uses_controlled_view_bounds_on_declarative_path --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative line/area/stems command owner split - 2026-06-02

This maintenance slice turns the shared command root into a thin projection hub while preserving the
opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/commands/line_area.rs` owns area fill path closure, stems
  baseline command projection, and step-mode expansion.
- `commands.rs` keeps shared line/area path keys and re-exports all private command owners.
- Evidence anchor: line, area-fill, stems, and step command projection owner.
- Evidence anchor: Commands root is now a command projection hub.
- `series_paint/line_area.rs` remains the paint owner for style/color, draw order, and painter
  dispatch.
- The line/area command owner stays bar/histogram-free, candlestick-free, error-bars-free,
  shaded-free, paint-free, event-free, output-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so line/area/stems/step
  command logic cannot drift back into `commands.rs` and other series command builders cannot drift
  into the line/area owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot line_plot_panel_paints_seeded_line_on_declarative_path area_plot_panel_paints_area_fill_and_stroke_on_declarative_path stems_plot_panel_paints_stems_from_baseline_on_declarative_path --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative shaded command owner split - 2026-06-02

This maintenance slice keeps shaded-band cursor and command construction out of the shared command
root while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/commands/shaded.rs` owns shaded lower path-key projection,
  sorted-series cursor interpolation, fallback aligned-series projection, upper/lower stroke
  commands, and fill band closure.
- `commands.rs` re-exports shaded command entrypoints and keeps non-shaded command builders plus
  shared path keys.
- Evidence anchor: shaded band sorted-cursor command projection owner.
- Evidence anchor: Commands root re-exports shaded command entrypoints.
- `series_paint/shaded.rs` remains the paint owner for style/color, draw order, and painter
  dispatch.
- The shaded command owner stays bar/histogram-free, candlestick-free, error-bars-free, paint-free,
  event-free, output-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so shaded command logic cannot
  drift back into `commands.rs` and other series command builders cannot drift into the shaded
  owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot shaded_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative error-bars command owner split - 2026-06-02

This maintenance slice keeps error-bars cap and marker command construction out of the shared
command root while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/commands/error_bars.rs` owns error-bar x/y cap command
  projection and marker shape command projection.
- `commands.rs` re-exports the error-bars command entrypoint and keeps non-error-bars command
  builders plus shared path keys.
- Evidence anchor: error-bars cap and marker command projection owner.
- Evidence anchor: Commands root re-exports error-bars command entrypoint.
- `series_paint/error_bars.rs` remains the paint owner for draw order, stroke style, color, and
  painter dispatch.
- The error-bars command owner stays bar/histogram-free, candlestick-free, shaded-free, paint-free,
  event-free, output-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so error-bars command logic
  cannot drift back into `commands.rs` and other series command builders cannot drift into the
  error-bars owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot error_bars_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative bar/histogram command owner split - 2026-06-02

This maintenance slice keeps bar and histogram closed-rectangle command construction out of the
shared command root while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/commands/bar_histogram.rs` owns histogram bin closed-rect
  command projection and grouped/stacked bar baseline closed-rect command projection.
- `commands.rs` re-exports bar and histogram command entrypoints and keeps non-bar/histogram
  command builders plus shared path keys.
- Evidence anchor: bar and histogram closed-rect command projection owner.
- Evidence anchor: Commands root re-exports bar and histogram command entrypoints.
- `series_paint/bar_histogram.rs` remains the paint owner for style/color, draw order, and painter
  dispatch.
- The bar/histogram command owner stays candlestick-free, error-bars-free, paint-free, event-free,
  output-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so bar/histogram command logic
  cannot drift back into `commands.rs` and other series command builders cannot drift into the
  bar/histogram owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo nextest run -p fret-plot histogram_plot_panel bars_plot_panel --no-fail-fast` - passed
  (3 passed, 86 skipped; `bars_plot_panel` also matched the existing `error_bars_plot_panel`
  substring test).
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative candlestick command owner split - 2026-06-02

This maintenance slice keeps candlestick path-command construction out of the shared command root
while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/commands/candlestick.rs` owns candlestick wick/body command
  projection, down-body path key projection, rectangle body closure, and device point budgeting.
- `commands.rs` re-exports candlestick command entrypoints and keeps non-candlestick command
  builders plus shared path keys.
- Evidence anchor: candlestick wick/body command projection owner.
- Evidence anchor: Commands root re-exports candlestick command entrypoints.
- `series_paint/candlestick.rs` remains the paint owner for colors, draw order, and painter
  dispatch.
- The candlestick command owner stays paint-free, event-free, output-free, authoring-free, and
  retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so candlestick command logic
  cannot drift back into `commands.rs` and other series command builders cannot drift into the
  candlestick owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib candlestick_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative prop records owner split - 2026-06-02

This maintenance slice keeps public plot panel prop records out of the builder-method owner while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/props/records.rs` owns the public `*PlotPanelProps` record
  definitions.
- `props.rs` re-exports prop records and keeps builder methods plus heatmap colorbar defaults.
- Evidence anchor: public `*PlotPanelProps` record definitions.
- Evidence anchor: Props root re-exports plot panel prop records.
- The records owner stays builder-free, panel-entrypoint-free, paint-free, event-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so public prop records cannot
  drift back into `props.rs` and builder/default policy cannot drift into `props/records.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative box zoom interaction owner split - 2026-06-02

This maintenance slice keeps box zoom event routing out of the shared interaction owner while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/interaction/box_zoom.rs` is the box zoom event routing owner.
- `interaction.rs` re-exports box zoom entrypoints and keeps legend routing plus event snapshots.
- Evidence anchor: box zoom event routing owner.
- Evidence anchor: Interaction root re-exports box zoom event routing.
- Evidence anchor: legend event routing and pointer event snapshots.
- The box zoom owner stays paint-free, output-publication-free, query/pan/wheel/draggable-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so box zoom session state,
  modifier expansion, active-selection updates, axis-lock filtering, and scaled view-bound writes
  cannot drift back into `interaction.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_box --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_box --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative query interaction owner split - 2026-06-02

This maintenance slice keeps query drag event routing out of the shared interaction owner while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/interaction/query.rs` is the query drag event routing owner.
- `interaction.rs` re-exports query entrypoints and keeps legend/box-zoom routing.
- Evidence anchor: query drag event routing owner.
- Evidence anchor: Interaction root re-exports query drag event routing.
- Evidence anchor: legend and box-zoom event routing.
- The query owner stays paint-free, output-publication-free, box/pan/wheel/draggable-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so query session state,
  query rectangle projection, active-selection updates, and query state writes cannot drift back
  into `interaction.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_query --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_query --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative pan interaction owner split - 2026-06-02

This maintenance slice keeps pan event routing out of the shared interaction owner while preserving
the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/interaction/pan.rs` is the pan event routing owner.
- `interaction.rs` re-exports pan entrypoints and keeps legend/query/box-zoom routing.
- Evidence anchor: pan event routing owner.
- Evidence anchor: Interaction root re-exports pan event routing.
- Evidence anchor: legend, query, and box-zoom event routing.
- The pan owner stays paint-free, output-publication-free, query/box/wheel/draggable-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so pan session state,
  pointer-drag routing, axis-lock filtering, and scaled pan projection cannot drift back into
  `interaction.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_pan --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative wheel zoom interaction owner split - 2026-06-02

This maintenance slice keeps wheel zoom event routing out of the shared interaction owner while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/interaction/wheel.rs` is the wheel zoom event routing owner.
- `interaction.rs` re-exports wheel zoom entrypoints and keeps legend/query/box-zoom/pan routing.
- Evidence anchor: wheel zoom event routing owner.
- Evidence anchor: Interaction root re-exports wheel zoom event routing.
- Evidence anchor: legend, query, box-zoom, and pan event routing.
- The wheel owner stays paint-free, output-publication-free, query/box/pan/draggable-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so wheel region detection,
  modifier-to-axis selection, axis-lock filtering, and zoom projection cannot drift back into
  `interaction.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_wheel --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative draggable interaction owner split - 2026-06-02

This maintenance slice keeps draggable overlay event routing out of the shared interaction owner
while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/interaction/draggable.rs` is the draggable overlay event routing owner.
- `interaction.rs` re-exports draggable interaction entrypoints and keeps legend/query/box-zoom/pan/wheel routing.
- Evidence anchor: draggable overlay event routing owner.
- Evidence anchor: Interaction root re-exports draggable overlay event routing.
- Evidence anchor: legend, query, box-zoom, pan, and wheel event routing.
- The draggable owner stays paint-free, output-publication-free, query/box/pan/wheel-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so draggable hit-test,
  session mutation, drag-output projection, and transform helpers cannot drift back into
  `interaction.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative annotation overlay helper owner split - 2026-06-02

This maintenance slice keeps annotation token resolution and marker/text-box helpers out of the
overlay root while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/annotation.rs` is the annotation token and marker paint helper owner.
- `overlays.rs` re-exports annotation helpers and overlay paint owner entrypoints.
- Evidence anchor: annotation token and marker paint helper owner.
- Evidence anchor: Overlays root re-exports annotation helpers.
- The annotation owner stays event-free, output-free, state-model-free, overlay-record-free,
  image-overlay-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so annotation helper bodies
  cannot drift back into `overlays.rs` and overlay projection concerns cannot drift into
  `overlays/annotation.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_tag_x_and_y_overlays --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_text_overlay --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_draggable_overlay_labels --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative draggable shape overlay paint owner split - 2026-06-02

This maintenance slice keeps draggable point and rectangle projection out of the shared overlay
owner while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/draggable_shapes.rs` is the draggable point and rectangle shape paint owner.
- `overlays.rs` re-exports draggable-shape overlay painting and keeps shared annotation helpers.
- Evidence anchor: draggable point and rectangle shape paint owner.
- Evidence anchor: Overlays root re-exports draggable-shape overlay painting.
- The draggable-shape owner stays event-free, output-free, state-model-free, image-overlay-free,
  annotation-helper-free, authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so draggable-shape projection
  cannot drift back into `overlays.rs` and non-shape overlay concerns cannot drift into
  `overlays/draggable_shapes.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_draggable_point_and_rect --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_draggable_shapes --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative reference-line overlay paint owner split - 2026-06-02

This maintenance slice keeps infinite-line and draggable-line rectangle projection out of the
shared overlay owner while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/reference_lines.rs` is the reference and draggable line rectangle paint owner.
- `overlays.rs` re-exports reference-line overlay painting and keeps draggable point/rect painting plus shared annotation helpers.
- Evidence anchor: reference and draggable line rectangle paint owner.
- Evidence anchor: Overlays root re-exports reference-line overlay painting.
- The reference-line owner stays event-free, output-free, state-model-free, image-overlay-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so reference-line projection
  cannot drift back into `overlays.rs` and non-reference overlay concerns cannot drift into
  `overlays/reference_lines.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_reference_lines --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_draggable_lines --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative plot text overlay paint owner split - 2026-06-02

This maintenance slice keeps `PlotText` overlay placement and text/background emission out of the
shared overlay owner while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/text.rs` is the `PlotText` overlay paint owner.
- `overlays.rs` re-exports plot text overlay painting and keeps shared annotation helpers.
- Evidence anchor: PlotText overlay paint owner.
- Evidence anchor: Overlays root re-exports plot text overlay painting.
- The text owner stays event-free, output-free, state-model-free, image-overlay-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so plot-text projection cannot
  drift back into `overlays.rs` and non-text overlay concerns cannot drift into `overlays/text.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_text_overlay --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_tag_x_and_y_overlays --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative tag overlay paint owner split - 2026-06-02

This maintenance slice keeps `TagX` and `TagY` overlay projection out of the shared overlay owner
while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/tags.rs` is the `TagX` and `TagY` overlay paint owner.
- `overlays.rs` re-exports tag overlay painting and keeps shared annotation helpers.
- Evidence anchor: TagX and TagY overlay paint owner.
- Evidence anchor: Overlays root re-exports tag overlay painting.
- The tag owner stays event-free, output-free, state-model-free, image-overlay-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so tag projection cannot drift
  back into `overlays.rs` and non-tag overlay concerns cannot drift into `overlays/tags.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_tag_x_and_y_overlays --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_draggable_overlay_labels --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative draggable overlay labels paint owner split - 2026-06-02

This maintenance slice keeps draggable line and point label projection out of the shared overlay
owner while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/draggable_labels.rs` is the draggable line and point label paint owner.
- `overlays.rs` re-exports draggable overlay label painting and keeps shared annotation helpers.
- Evidence anchor: draggable line and point label paint owner.
- Evidence anchor: Overlays root re-exports draggable overlay label painting.
- The draggable labels owner stays event-free, output-free, state-model-free, image-overlay-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so draggable label projection
  cannot drift back into `overlays.rs` and non-label overlay concerns cannot drift into
  `overlays/draggable_labels.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_draggable_overlay_labels --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_draggable_overlay_labels --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_tag_x_and_y_overlays --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative image overlay paint owner split - 2026-06-02

This maintenance slice keeps caller-owned `PlotImage` layer filtering, multi-axis projection,
clipping, opacity filtering, and `ImageRegion` scene emission out of the shared overlay owner while
preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/overlays/images.rs` is the caller-owned `PlotImage`
  `ImageRegion` paint owner.
- `overlays.rs` re-exports image overlay painting and keeps non-image overlay paint concerns.
- Evidence anchor: caller-owned PlotImage ImageRegion paint owner.
- Evidence anchor: Overlays root re-exports image overlay painting.
- The image owner stays event-free, output-free, state-model-free, authoring-free, and
  retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so image scene-emission
  details cannot drift back into `overlays.rs` and non-image overlay concerns cannot drift into
  `overlays/images.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_image_overlay --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_plot_image_overlays --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_tag_x_and_y_overlays --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative line/area/stems series paint owner split - 2026-06-02

This maintenance slice keeps line, area-fill, and stems stroke path drawing out of the shared
series router while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/series_paint/line_area.rs` is the line, area-fill, and stems stroke path owner.
- Series paint router delegates line/area/stems drawing and keeps axis transform selection plus
  concrete series routing.
- The line/area/stems owner stays event-free, output-free, overlay-free, axis-routing-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so line/area/stems
  command/path logic cannot drift back into `series_paint.rs` and non-line/area/stems concerns
  cannot drift into `line_area.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_seeded_line --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib area_plot_panel_paints_area_fill_and_stroke_on_declarative_path --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib stems_plot_panel_paints_stems_from_baseline_on_declarative_path --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative shaded series paint owner split - 2026-06-02

This maintenance slice keeps shaded band fill and upper/lower stroke path drawing out of the shared
series router while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/series_paint/shaded.rs` is the shaded band fill and upper/lower stroke path owner.
- Series paint router delegates shaded drawing and keeps non-shaded series routing.
- The shaded owner stays event-free, output-free, overlay-free, axis-routing-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so shaded command/path logic
  cannot drift back into `series_paint.rs` and non-shaded concerns cannot drift into `shaded.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib shaded_plot_panel_paints_band_fill_and_two_strokes_on_declarative_path --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative error-bars series paint owner split - 2026-06-02

This maintenance slice keeps error-bars caps/markers stroke path drawing out of the shared series
router while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/series_paint/error_bars.rs` is the error-bars caps and markers stroke path owner.
- Series paint router delegates error-bars drawing and keeps non-error-bars series routing.
- The error-bars owner stays event-free, output-free, overlay-free, axis-routing-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so error-bars command logic
  cannot drift back into `series_paint.rs` and non-error-bars concerns cannot drift into
  `error_bars.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib error_bars_plot_panel_paints_x_y_errors_caps_and_markers --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative bar and histogram series paint owner split - 2026-06-02

This maintenance slice keeps bar/histogram closed fill path drawing out of the shared series router
while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/series_paint/bar_histogram.rs` is the bar and histogram closed fill path owner.
- Series paint router delegates bar and histogram drawing and keeps non-bar/histogram series
  routing.
- The bar/histogram owner stays event-free, output-free, overlay-free, axis-routing-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so bar/histogram command logic
  cannot drift back into `series_paint.rs` and non-bar/histogram concerns cannot drift into
  `bar_histogram.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib bars_plot_panel_paints_grouped_and_stacked_closed_fill_paths --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib histogram_plot_panel_paints_closed_bin_fill_paths --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative candlestick series paint owner split - 2026-06-02

This maintenance slice keeps candlestick-specific wick/body paint logic out of the shared series
router while preserving the opt-in IMUI plot adapter behavior:

- `ecosystem/fret-plot/src/declarative/series_paint/candlestick.rs` is the candlestick wick/body command paint owner.
- Series paint router delegates candlestick drawing and keeps non-candlestick line, area, shaded,
  stems, histogram, bars, and error-bar routing.
- The candlestick owner stays event-free, output-free, overlay-free, axis-routing-free,
  authoring-free, and retained-free.
- `tools/gate_imui_workstream_source.py` now source-checks the split so candlestick command/path
  logic cannot drift back into `series_paint.rs` and non-candlestick series concerns cannot drift
  into `candlestick.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib candlestick_plot_panel_paints_wicks_and_up_down_bodies --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_seeded_line --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative series paint owner split - 2026-06-02

This maintenance slice keeps concrete plot series drawing out of the panel-level orchestration
owner while preserving the opt-in IMUI plot adapter behavior:

- At that slice, `ecosystem/fret-plot/src/declarative/series_paint.rs` was the line, area, shaded, stems, histogram, bars, candlestick, and error-bar series paint owner; the later candlestick owner split above narrows current series router responsibility.
- Panel paint owner keeps background, grid, overlays, legend, selection, and readout orchestration.
- The series owner stays event-free, output-free, overlay-free, authoring-free, and retained-free;
  it imports existing command builders, multi-axis geometry, and style helper owners explicitly.
- `tools/gate_imui_workstream_source.py` now source-checks the split so series path/color command
  logic cannot drift back into `panel_paint.rs` and panel/readout/overlay concerns cannot drift
  into `series_paint.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_seeded_line --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axes_and_grid --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_image_overlay --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative panel paint owner split - 2026-06-02

This maintenance slice keeps the opt-in plot adapter's declarative panel implementation reviewable
while preserving the editor-grade IMUI layering goal:

- At that slice, `ecosystem/fret-plot/src/declarative/panel_paint.rs` was the panel background, grid, series, overlay, legend, and readout paint orchestration owner; the later series owner split above narrows current panel paint responsibility.
- Root keeps panel element and event wiring in `ecosystem/fret-plot/src/declarative.rs`, plus the
  public `panels.rs` and `props.rs` re-export hub.
- The paint owner stays event-free, state-model-free, authoring-free, and retained-free; it imports
  existing grid/axes, heatmap, right-axis labels, overlays, selection, readout, command, geometry,
  and style helper owners explicitly.
- `tools/gate_imui_workstream_source.py` now source-checks the split so paint orchestration cannot
  drift back into the root event/element module and event/state concerns cannot drift into
  `panel_paint.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_seeded_line --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axes_and_grid --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_image_overlay --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `cargo fmt -p fret-plot -- --check` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## IMUI drag preview cross-window owner split - 2026-06-02

This maintenance slice keeps the drag-preview recipe aligned with the Dear ImGui-style editor goal
without moving preview policy into the thin `fret-imui` facade or the lower `fret-ui-kit::imui`
mechanism layer:

- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs` now stays as the recipe facade for
  options, same-window tooltip overlay presentation, anchor policy, and public re-exports.
- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview/cross_window.rs` now owns the cross-window
  descriptor store, publish helpers, current-window render loop, and stale-session pruning.
- The public API remains under `fret_ui_kit::recipes::imui_drag_preview::{...}` through re-exports,
  preserving existing proof surfaces and app call sites.
- `tools/gate_imui_workstream_source.py` now source-checks the split so the facade cannot silently
  absorb the cross-window store again and the cross-window owner cannot absorb same-window overlay
  policy.

Fresh gates:

- `cargo fmt -p fret-ui-kit` - passed.
- `cargo check -p fret-ui-kit --features imui` - passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_preview_smoke --no-fail-fast` - passed.
- `cargo nextest run -p fret-ui-kit --features imui ghost_anchor_ --no-fail-fast` - passed, 2 tests.
- `cargo nextest run -p fret-imui drag_preview_ghost_follows_pointer_and_clears_on_release cross_window_drag_preview_ghost_transfers_between_windows --no-fail-fast` - passed, 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.

## DevTools native regression test owner split - 2026-06-02

This maintenance slice keeps the DevTools GUI source file reviewable while preserving the existing
first-open, recent-evidence, workflow-run, and Demo/Metrics/Debug proof pressure:

- `apps/fret-devtools/src/native.rs` now keeps only the bin-root GUI shell and an explicit
  `#[path = "native/tests.rs"]` test module hook.
- `apps/fret-devtools/src/native/tests.rs` owns the former inline regression tests for first-open
  evidence, recent evidence, workflow handoff, file URL projection, regression summary drilldown,
  and Demo/Metrics/Debug route projection.
- The IMUI source gate now treats `native/tests.rs` as the additional DevTools native test owner
  instead of forcing all test markers to stay in the main GUI source file.
- The DevTools first-open and product-chain gates now include the split test source in their
  combined source validation, so route/runtime markers and regression-test markers still drift
  together.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools --no-fail-fast` - passed, 92 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## DevTools Demo/Metrics/Debug workflow artifact load handoff - 2026-06-02

This refresh keeps perf workflow artifact handoff close to the Demo/Metrics/Debug workflow controls:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now reuses the existing
  `Load workflow regression summary` and `Load workflow regression index` commands from the
  always-visible Demo/Metrics/Debug panel.
- The guide projection now emits `workflow artifact action` rows that show whether the selected
  workflow result has regression summary/index artifacts ready to load.
- The implementation still relies on the existing selected workflow result helpers and Regression
  Workspace loaders in `apps/fret-devtools/src/native.rs`; no artifact parser or runner fork was
  added to the Demo/Metrics/Debug owner.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_workflow_lines_surface_runtime_readiness_and_status demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.

## DevTools Demo/Metrics/Debug workflow result handoff - 2026-06-02

This refresh keeps workflow execution and result handoff in the same always-visible
Demo/Metrics/Debug route:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now reuses the existing Workflow Runs commands for
  `Copy workflow result` and `Open workflow JSON` directly from the Demo/Metrics/Debug panel.
- The result buttons are disabled until a workflow result artifact exists, and the guide projection
  records `workflow result action` rows with the copied/opened command ids and availability reason.
- No parallel runner or command parser was added; result handoff still flows through the existing
  workflow result selection and platform URL handlers in `apps/fret-devtools/src/native.rs`.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_workflow_lines_surface_runtime_readiness_and_status demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.

## DevTools Demo/Metrics/Debug workflow status loop - 2026-06-02

This refresh makes the Demo/Metrics/Debug route a clearer workflow handoff loop instead of a
button-only launch surface:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now projects workflow readiness for
  `campaign-validate-imui-p3-multiwindow` and `perf-docking-suite-ws` into the same always-visible
  Demo/Metrics/Debug panel.
- The readiness rows report in-flight blocking, missing selected-session state, and selected-session
  availability from the same `devtools_workflow_commands_from_state(...)` source used by the run
  buttons.
- The panel now also projects `workflow status` with in-flight state, last workflow result path, and
  last workflow error so users can see the immediate result handoff without leaving the route.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_workflow_lines_surface_runtime_readiness_and_status demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.

## DevTools Demo/Metrics/Debug perf workflow run entry - 2026-06-02

This refresh extends the Demo/Metrics/Debug route from docking workflow execution to the existing
perf-docking workflow surface:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now surfaces a `Run perf workflow` button and a
  `workflow handoff` line for `perf-docking-suite-ws`.
- The button is disabled when a workflow is already in flight or when no selected session makes the
  perf workflow runnable.
- `apps/fret-devtools/src/native.rs` routes
  `fret.devtools.demo_metrics_debug.run_perf_workflow` through the same
  `workflow_run::start_workflow_run(...)` path as the Workflow Runs panel.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## DevTools Demo/Metrics/Debug docking workflow run entry - 2026-06-02

This refresh moves the Demo/Metrics/Debug route from copy-only diagnostics toward an executable
DevTools workflow entry:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now surfaces a `Run docking workflow` button and a
  `workflow handoff` line for the existing `campaign-validate-imui-p3-multiwindow` workflow.
- `apps/fret-devtools/src/native.rs` routes
  `fret.devtools.demo_metrics_debug.run_docking_workflow` through the existing
  `workflow_run::start_workflow_run(...)` path instead of adding a parallel process runner.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the stateful action-row signature and
  the docking workflow button.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## DevTools Demo/Metrics/Debug per-action copy commands - 2026-06-02

This refresh keeps the Demo/Metrics/Debug route always available in the DevTools GUI while making
each canonical action individually copyable:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now owns stable
  `fret.devtools.demo_metrics_debug.copy_action.*` command ids, per-action copy command projection,
  and GUI buttons for each action in addition to the full action-bundle copy button.
- `apps/fret-devtools/src/native.rs` routes those dynamic action-copy command ids to clipboard
  writes without claiming a shared DevTools command palette contract.
- The existing first-open/product-chain discovery gates keep their original route-surface marker
  and now also cover the per-action copy command GUI source.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## IMUI facade trait roster owner split - 2026-06-02

This refresh continues the `fret-ui-kit::imui` file split by moving the public writer extension
trait roster out of the facade writer hub without changing the public import path:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/trait_ext.rs` now owns the
  `UiWriterImUiFacadeExt` macro roster and blanket `UiWriter` implementation.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps module declarations, and
  `facade_writer.rs` remains the public re-export hub for `ImUiFacade` and
  `UiWriterImUiFacadeExt`.
- `tools/gate_imui_workstream_source.py` now checks `trait_ext.rs` as the roster owner and rejects
  the extension trait returning to `facade_writer.rs`.

Fresh gates:

- `cargo fmt -p fret-ui-kit` - passed.
- `cargo check -p fret-ui-kit --features imui` - passed.
- `cargo nextest run -p fret-ui-kit --features imui imui_text --no-fail-fast` - passed.

## Fret Plot declarative panel entrypoint owner split - 2026-06-02

This refresh follows the plot props split by moving the public declarative panel adapters out of
the root implementation file without changing the retained-free plot rendering path:

- `ecosystem/fret-plot/src/declarative/panels.rs` now owns the public `*_plot_panel` entrypoints
  and `*_plot_panel_in` wrappers for line, error-bars, histogram, bars, candlestick, heatmap,
  histogram2d, area, shaded, and stems plot panels.
- The retained-free paint/event core stays in `declarative.rs`, including grid/axes, overlays,
  readout, interaction output, and tests.
- `ecosystem/fret-plot/src/declarative/props.rs` remains the public props/builder owner, so the
  optional `fret-plot/imui` adapter continues to delegate through declarative panel entrypoints.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot` - passed with existing dead-code warnings in `plot/view.rs`.
- `cargo check -p fret-plot --features imui` - passed with the same existing warnings.
- `cargo nextest run -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative props owner split - 2026-06-02

This refresh keeps the retained-canvas-free optional IMUI plot adapter declarative-only while
reducing the remaining `fret-plot` declarative implementation file:

- `ecosystem/fret-plot/src/declarative/props.rs` now owns the public `*PlotPanelProps` types and
  builder methods for line, error-bars, histogram, bars, candlestick, heatmap, histogram2d, area,
  shaded, and stems plot panels.
- `ecosystem/fret-plot/src/declarative.rs` keeps retained-free canvas painting, event handling,
  readout, and test coverage while the public panel entrypoints and props live in child owners.
- `ecosystem/fret-plot/src/imui.rs` remains a thin optional `UiWriter` adapter over the
  declarative panel entrypoints, so plot ergonomics do not move into `fret-imui` or
  `fret-ui-kit::imui`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot` - passed with existing dead-code warnings in `plot/view.rs`.
- `cargo check -p fret-plot --features imui` - passed with the same existing warnings.
- `cargo nextest run -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative model projection owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving normalized plot
model projection out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/model.rs` now owns the projection path where all concrete plot models project into private `PlotPanelModel` records
  for line, area, shaded, stems, error-bars, histogram, bars, candlestick, heatmap, and histogram2d
  panels.
- The projection owner also owns histogram bin projection, including conversion from histogram
  values to sorted render series and bin-width metadata.
- `ecosystem/fret-plot/src/declarative.rs` imports the private projection records and keeps
  paint/event/layout orchestration only.
- `ecosystem/fret-plot/src/declarative/panels.rs` and
  `ecosystem/fret-plot/src/declarative/props.rs` remain the public entrypoint and props owners,
  which keeps paint/event/panel entrypoints out of the projection owner.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative legend owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving legend paint and
hit testing out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/legend.rs` now owns the legend paint and hit-test owner,
  including row metrics, text/swatch painting, hover/pin highlight and swatch/label hit testing.
- `ecosystem/fret-plot/src/declarative.rs` imports the legend paint/hit-test entrypoints while
  event state mutation stays in `declarative.rs`, preserving existing hidden-series and pinned-series
  behavior.
- The legend owner does not depend on `fret-imui`, `fret-authoring`, retained plot bridges, or plot
  panel props; it stays a private child of the declarative plot surface.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative path-command owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving series-to-path
projection out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/commands.rs` now owns the path-command projection owner,
  including line, area, shaded, stems, histogram, bars, candlestick, and error-bar command builders.
- The command owner also owns path-key helpers, step-mode expansion, marker command generation, and
  candlestick device-point budgeting.
- `ecosystem/fret-plot/src/declarative.rs` imports the command projection entrypoints while
  paint/event orchestration stays in `declarative.rs`.
- The command owner does not depend on `CanvasPainter`, `UiHost`, `ElementContext`, `PlotState`,
  `fret-authoring`, retained plot bridges, or optional IMUI adapter policy.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative selection overlay owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving selection overlay
paint and tooltip geometry out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/selection.rs` now owns the selection overlay paint and tooltip owner,
  including query/box-zoom selection rectangles and tooltip placement.
- `ecosystem/fret-plot/src/declarative.rs` imports the selection paint/tooltip entrypoints while
  drag/session mutation stays in `declarative.rs`.
- The selection owner reads the existing overlay records and formatting helpers, but it does not
  own `PlotState`, pointer event handling, drag sessions, `UiHost`, `fret-authoring`, retained plot
  bridges, or optional IMUI adapter policy.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative style helpers owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving shared formatting
and color fallback out of the implementation root:

- `ecosystem/fret-plot/src/declarative/style_helpers.rs` now owns the axis label formatting and series color fallback owner,
  including log-scale auto labels and palette fallback selection.
- Axis labels, readout, selection, overlays, legend, and panel paint import style helpers explicitly.
- `ecosystem/fret-plot/src/declarative.rs` keeps panel assembly, paint orchestration, and plot state
  model wiring.
  Panel paint orchestration stays in `declarative.rs`.
- No public props, optional IMUI adapter policy, retained plot bridge, event routing, output
  publication, geometry, or crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axis_tick_labels --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_series_legend --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_seeded_line --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_cursor_readout_without_output_model --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_query_selection_tooltip --no-fail-fast` -
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative geometry owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving shared geometry out
of the implementation root:

- `ecosystem/fret-plot/src/declarative/geometry.rs` now owns the shared inner-rect and y-axis view-bounds geometry owner,
  including plot inner-rect projection and right-axis view-bounds projection.
- Axis labels, interaction, output, and overlay owners import geometry explicitly.
- `ecosystem/fret-plot/src/declarative.rs` kept panel assembly, paint orchestration, formatting
  helpers, series color policy, and plot state model wiring for this slice; the later style helper
  owner split narrows that current root role.
  Panel paint orchestration stays in `declarative.rs`.
- No public props, optional IMUI adapter policy, retained plot bridge, event routing, output
  publication, formatting, color policy, or crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_uses_controlled_view_bounds --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_updates_output_cursor --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_series_with_right_axis_bounds --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_image_overlay --no-fail-fast` -
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative paint primitives owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving shared Quad paint
helpers out of the implementation root:

- `ecosystem/fret-plot/src/declarative/paint_primitives.rs` now owns the shared Quad primitive owner for line and filled-rect paint helpers,
  including vertical lines, horizontal lines, and filled rectangles.
- Grid, readout, heatmap, and overlay owners import primitives explicitly from
  `paint_primitives.rs`.
- `ecosystem/fret-plot/src/declarative.rs` kept panel assembly, paint orchestration, shared
  geometry helpers, and plot state model wiring for this slice; the later geometry owner split
  narrows that current root role.
  Panel paint orchestration stays in `declarative.rs`.
- No public props, optional IMUI adapter policy, retained plot bridge, event routing, output
  publication, or crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axes_and_grid --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib heatmap_plot_panel_paints_grid_cells --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_cursor_readout_without_output_model --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_plot_image_overlay --no-fail-fast` -
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative grid axes owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving grid/axis baseline
painting out of the implementation root:

- `ecosystem/fret-plot/src/declarative/grid_axes.rs` now owns the grid line, baseline axis, and primary tick label orchestration owner,
  including tick projection, grid line painting, baseline axis painting, and the primary-axis tick
  label paint call.
- `ecosystem/fret-plot/src/declarative.rs` kept panel assembly, paint orchestration, shared paint
  primitives, shared geometry helpers, and plot state model wiring for this slice.
  Panel paint orchestration stays in `declarative.rs`.
- Shared paint primitives stayed in `declarative.rs` for this slice because readout, heatmap, and
  overlay owners still shared those helpers; the later paint primitive owner split narrows that
  current root role.
- No public props, optional IMUI adapter policy, retained plot bridge, event routing, output
  publication, or crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axes_and_grid --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_axis_tick_labels --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_paints_right_axis_tick_labels --no-fail-fast` -
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative output owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving output projection
out of the implementation root:

- `ecosystem/fret-plot/src/declarative/output.rs` now owns output publication, query extraction,
  pointer cursor snapshots, output snapshot construction, and state/default view bounds projection.
  It is the output publication, query extraction, pointer cursor snapshots, and view bounds projection owner.
- `ecosystem/fret-plot/src/declarative.rs` keeps panel assembly, paint orchestration, grid/axis
  painting, shared geometry helpers, and plot state model wiring.
  Panel paint orchestration stays in `declarative.rs`.
- `ecosystem/fret-plot/src/declarative/interaction.rs` imports current-view and pointer snapshot
  helpers from the output owner, keeping event routing out of paint orchestration.
- No public props, optional IMUI adapter policy, retained plot bridge, or crate layering changed in
  this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_uses_controlled_view_bounds --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_updates_output_cursor --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_query_drag_updates_output_query --no-fail-fast` -
  passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative interaction owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving event routing out of
the implementation root:

- `ecosystem/fret-plot/src/declarative/interaction.rs` now owns the legend, draggable, query, box-zoom, pan, and wheel event routing owner,
  including interaction session records, selection overlay records, legend hover projection, and
  pointer-event snapshot projection.
- `ecosystem/fret-plot/src/declarative.rs` kept panel assembly, paint orchestration, output
  publication, view/output snapshot records, shared geometry helpers, and plot state model wiring
  for this slice; the later output owner split narrows that current root role.
- Paint owners stay event-free, and no public props, optional IMUI adapter policy, retained plot
  bridge, or crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_legend --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_query_drag --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_box_zoom --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_pan --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_wheel_zoom --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative tests owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving the plot panel
regression harness out of the implementation root:

- `ecosystem/fret-plot/src/declarative/tests.rs` now owns the declarative plot panel regression tests,
  including the `TestHost`, scene helpers, paint regressions, drag output regressions, and
  linked-cursor/readout regressions.
- The test harness moved out of `declarative.rs`; the implementation root keeps `#[cfg(test)] mod tests;` only.
- No implementation code, public props, optional IMUI adapter policy, retained plot bridge, or
  crate layering changed in this slice.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_paints --no-fail-fast` - passed, 26 tests.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed, 4 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative overlay paint owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving overlay painting out
of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/overlays.rs` is now the overlay re-export hub for
  annotation and paint owners.
- Evidence anchor: overlay re-export hub for annotation and paint owners.
- Private child owners carry annotation helpers, reference lines, draggable shapes, image overlays,
  draggable labels, tag overlays, and text overlays.
- `ecosystem/fret-plot/src/declarative.rs` imports overlay paint entrypoints while panel paint
  orchestration, draggable overlay event routing, output publication, and plot state handling stay
  in the root.
- The overlay paint owner does not own `PlotState`, pointer event handling, output publication,
  `UiHost`, `fret-authoring`, retained plot bridges, or optional IMUI adapter policy; overlay paint stays state-free and event-free,
  and draggable overlay event routing stays in `declarative.rs`.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib line_plot_panel_paints --no-fail-fast` - passed.
- `cargo test -p fret-plot --lib line_plot_panel_drags --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative heatmap owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving heatmap painting out
of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/heatmap.rs` now owns the heatmap and colorbar paint owner,
  including grid cell painting and default colorbar projection, colormap sampling, gradient steps,
  and min/max text labels.
- Heatmap model projection stays in `declarative/model.rs`; `ecosystem/fret-plot/src/declarative.rs`
  imports heatmap paint entrypoints while panel paint orchestration, event output publication, and
  plot state handling stay in the root.
- The heatmap owner does not own `PlotState`, pointer event handling, output publication,
  `UiHost`, `fret-authoring`, retained plot bridges, or optional IMUI adapter policy.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib heatmap_plot_panel_paints_grid_cells_as_declarative_quads --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib heatmap_plot_panel_paints_default_colorbar_on_declarative_path --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib histogram2d_plot_panel_paints_grid_cells_and_default_colorbar_on_declarative_path --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative axis label owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving axis label painting
out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/axis_labels.rs` now owns the axis tick label paint owner,
  including primary and right-axis label painting, y2/y3/y4 lane offsets, text constraints, and
  stable canvas text keys.
- `ecosystem/fret-plot/src/declarative.rs` imports the axis label paint entrypoints while grid and
  baseline axis painting, data/view bounds orchestration, and axis label formatting stays shared in `declarative.rs`
  for annotations, axes, tags, selection, and readout.
- The axis label owner does not own `PlotState`, pointer event handling, output publication,
  `UiHost`, `fret-authoring`, retained plot bridges, or optional IMUI adapter policy.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Fret Plot declarative readout owner split - 2026-06-02

This refresh keeps the optional IMUI plot adapter declarative-only while moving cursor readout
painting and series row projection out of the retained-free paint/event root:

- `ecosystem/fret-plot/src/declarative/readout.rs` now owns the cursor and linked-cursor readout paint owner,
  including crosshair painting, overlay placement, text construction, and series row projection and pinned-series filtering.
- `ecosystem/fret-plot/src/declarative.rs` imports readout paint entrypoints while axis label formatting stays shared in `declarative.rs`
  for annotations, axes, tags, selection, and readout.
- The readout owner does not own `PlotState`, pointer event handling, output publication,
  `UiHost`, `fret-authoring`, retained plot bridges, or optional IMUI adapter policy.

Fresh gates:

- `cargo fmt -p fret-plot` - passed.
- `cargo check -p fret-plot --features imui` - passed with existing dead-code warnings in
  `plot/view.rs`.
- `cargo test -p fret-plot --lib imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast` -
  passed, 1 test.
- `cargo test -p fret-plot --lib line_chart_builder_stays_model_only_on_default_surface --no-fail-fast` -
  passed, 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## DevTools Demo/Metrics/Debug line projection owner split - 2026-06-02

This refresh keeps the Dear ImGui-style Demo/Metrics/Debug route productized in DevTools while
reducing the remaining `native.rs` GUI shell file:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now owns action metadata, action command text,
  selected-bundle readiness projection, full route/metrics/debug guide line projection, GUI panel
  assembly, and the copy-action button row.
- `apps/fret-devtools/src/native.rs` keeps the surrounding Guide tab composition and command
  dispatch for copying the generated action bundle.
- `tools/gate_imui_workstream_source.py` now rejects the Demo/Metrics/Debug guide projection from
  drifting back into `native.rs`.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `cargo check -p fret-devtools` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## TextAssistField element owner split - 2026-06-02

This refresh keeps editor completion policy in `fret-ui-editor` while reducing another root control
module:

- `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now owns child-module wiring, public
  re-exports, and the narrow helper functions shared by element/panel/tests.
- `ecosystem/fret-ui-editor/src/controls/text_assist_field/element.rs` now owns
  `TextAssistField`, constructor/options/accept builders, caller-keyed `into_element(...)`, input
  assist semantics, input-owned key policy, inline/anchored overlay handoff, and final root layout.
- Existing owners remain unchanged: `model.rs` owns options/records, `panel.rs` owns listbox
  rendering, `overlay.rs` owns anchored overlay placement/dismissal, `empty.rs` owns empty state,
  and `accept.rs` owns accept mutation.
- `tools/gate_imui_workstream_source.py` now rejects the public element definition and assembly body
  from drifting back into the root module.

Fresh gates:

- `cargo fmt -p fret-ui-editor` - passed.
- `cargo check -p fret-ui-editor --features imui` - passed.
- `cargo nextest run -p fret-ui-editor --features imui text_assist --no-fail-fast` - passed.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## ColorEdit element owner split - 2026-06-02

This refresh keeps editor color picking policy in `fret-ui-editor` while reducing the remaining
ColorEdit root file ownership:

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs` now owns only child-module wiring, public
  re-exports, and shared color-edit constants used by child modules.
- `ecosystem/fret-ui-editor/src/controls/color_edit/element.rs` now owns `ColorEdit`, constructor
  and options builder methods, caller-keyed `into_element(...)`, root input/swatch assembly,
  drag-drop delivery, popup/tooltip/copy overlay requests, and final layout handoff.
- Existing focused owners remain unchanged: `input.rs` owns hex input, `swatch.rs` owns swatch
  behavior, `layout.rs` owns root layout, `state.rs` owns popup/draft/error models, and `popup/*`
  owns popup content.
- `tools/gate_imui_workstream_source.py` now rejects ColorEdit public element definitions and
  element assembly bodies from drifting back into the root module.

Fresh gates:

- `cargo fmt -p fret-ui-editor` - passed.
- `cargo check -p fret-ui-editor --features imui` - passed.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## TransformEdit model owner split - 2026-06-02

This refresh keeps transform editing policy in `fret-ui-editor` while reducing the remaining
TransformEdit root file ownership:

- `ecosystem/fret-ui-editor/src/controls/transform_edit.rs` now owns only module wiring, public
  re-exports, layout/options types, section identity, and read-only axis outcome vocabulary.
- `ecosystem/fret-ui-editor/src/controls/transform_edit/model.rs` now owns `TransformEdit`,
  `TransformEditPresentations`, constructor/presentation adapters, builder methods, caller-keyed
  `into_element(...)`, and the presentation adoption test.
- `ecosystem/fret-ui-editor/src/controls/transform_edit/element.rs` remains the keyed element
  assembly handoff, while section rendering, section controls, and linked-scale synchronization stay
  in their existing owners.
- `tools/gate_imui_workstream_source.py` now rejects TransformEdit public model definitions and
  presentation tests from drifting back into the root module.

Fresh gates:

- `cargo fmt -p fret-ui-editor` - passed.
- `cargo check -p fret-ui-editor --features imui` - passed.
- `cargo nextest run -p fret-ui-editor --features imui transform_edit --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## VecEdit model owner split - 2026-06-02

This refresh keeps editor controls in `fret-ui-editor` while reducing the remaining VecEdit root
file ownership:

- `ecosystem/fret-ui-editor/src/controls/vec_edit.rs` now owns only VecEdit child-module wiring and
  the stable public re-exports for axis, model, and options types.
- `ecosystem/fret-ui-editor/src/controls/vec_edit/model.rs` now owns `Vec2Edit`, `Vec3Edit`,
  `Vec4Edit`, their constructors, `NumericPresentation` adapters, builder methods, caller-keyed
  `into_element(...)` entrypoints, and the presentation adoption test.
- `ecosystem/fret-ui-editor/src/controls/vec_edit/element.rs` remains the keyed element assembly
  handoff, while `axis.rs`, `layout.rs`, and `options.rs` keep their existing owners.
- `tools/gate_imui_workstream_source.py` now rejects VecEdit public model definitions and
  presentation tests from drifting back into the root module.

Fresh gates:

- `cargo fmt -p fret-ui-editor` - passed.
- `cargo check -p fret-ui-editor --features imui` - passed.
- `cargo nextest run -p fret-ui-editor --features imui vec_edit --no-fail-fast` - passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `python tools\check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## DevTools Demo/Metrics/Debug action metadata owner split - 2026-05-31

This refresh keeps the Demo/Metrics/Debug route productized while reducing the size and ownership
load of `apps/fret-devtools/src/native.rs`:

- `apps/fret-devtools/src/demo_metrics_debug.rs` now owns the action metadata table plus command,
  metadata, and selected-bundle readiness projections.
- `apps/fret-devtools/src/native.rs` keeps the GUI row, copy command dispatch, stateful
  selected-bundle count, and guide-panel rendering.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` read the new module as part of their DevTools GUI source
  checks.
- `tools/gate_imui_workstream_source.py` source-checks both the private metadata owner and the thin
  GUI consumer boundary.

Fresh gates:

- `cargo fmt -p fret-devtools` - passed.
- `python -m py_compile tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py tools\gate_imui_workstream_source.py` - passed.
- `python tools\gate_imui_workstream_source.py` - passed.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast` - passed.
- `python tools\diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` - passed.
- `python tools\diag_gate_imui_product_chain.py --only discovery` - passed.

## Maintenance gate refresh - 2026-05-15

DevTools full clippy is now a current maintenance gate for the P2 diagnostics/devtools surface:

- Gate restored:
  - `cargo clippy -p fret-devtools --all-targets -- -D warnings`
- Evidence anchors:
  - `crates/fret-launch/src/runner/windows_mf_video.rs`
  - `crates/fret-launch/src/runner/desktop/runner/mod.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
  - `crates/fret-ui/src/text/input/widget.rs`
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/debug/virtual_list.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_index.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_thread_cpu_time.rs`
  - `crates/fret-ui/src/tree/prepaint/tests/prepaint_virtual_list_window_update_harness.rs`
- Structural notes:
  - Windows MF native-external importer now matches the AVFoundation runner-local ownership shape
    (`Rc<RefCell<_>>`) instead of using `Arc<Mutex<_>>` without a Send/Sync contract.
  - DevTools clippy blockers in dependent `fret-ui`, `fret-launch`, and `fret-bootstrap` code are
    fixed without adding `allow` attributes.
  - The prepaint fixture harness now reads current view-boundary dirty state
    (`dirty_boundaries` + `boundary_layout_dirty_reason`) instead of stale `dirty_cache_*` fields.
- Guardrails run:
  - `cargo clippy -p fret-devtools --all-targets -- -D warnings` - passed.
  - `cargo nextest run -p fret-ui mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast` - passed.
  - `cargo nextest run -p fret-ui -p fret-launch -p fret-bootstrap --no-fail-fast` - ran 1059 tests:
    1054 passed, 5 failed.
  - `python tools/check_layering.py` - passed.
  - `python tools/report_largest_files.py --top 30 --min-lines 800` - passed; this slice did not
    expand the reported large-file set.
  - `git diff --check` - passed.
- Residual full-nextest failures to keep as a follow-on input:
  - `declarative::tests::core::layout_refines_focus_traversal_availability_after_structural_fallback`
  - `declarative::tests::layout::scroll::scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative`
  - `declarative::tests::layout::scroll::scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge`
  - `declarative::tests::layout::viewport_roots::viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill`
  - `declarative::tests::virtual_list::caching::virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached`

## Evidence anchors (current)

- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_FOOTGUN_AUDIT_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_DEMOTE_DELETE_PLAN_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-15.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-25.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_CONSUMER_WORKFLOW_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-04.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PRODUCT_WORKFLOW_COHERENCE_REVIEW_2026-05-06.md`
- `tools/diag_gate_action_first_authoring_v1.py`
- `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
- `tools/diag-scripts/suites/cookbook-imui-action-basics/suite.json`
- `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json`
- `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-roughness-typing.json`
- `tools/diag-scripts/suites/cookbook-imui-editor-controls-basics/suite.json`
- `tools/diag-scripts/suites/editor-notes-demo/suite.json`
- `tools/diag-scripts/suites/editor-notes-device-shell-demo/suite.json`
- `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json`
- `tools/diag_gate_imui_product_chain.py`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-text-input-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-text-input-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-text-input-picker-a11y-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-picker-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-filter-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-mode-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-command-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-area-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-final-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-editor-cookbook-proof-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-popup-depth-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-preview-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-preview-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-drag-drop-payload-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-reference-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-hsv-picker-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-numeric-readout-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-model-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-numeric-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-picker-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-preview-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-swatches-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-debug-draw-baseline-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-shape-primitives-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-box-select-v1/DESIGN.md`
- `docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
- `docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`
- `docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-command-package-v1/DESIGN.md`
- `docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `docs/workstreams/imui-id-stack-diagnostics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-id-stack-browser-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-html-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-fixture-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json`
- `crates/fret-diag/src/identity_browser.rs`
- `crates/fret-diag/src/identity_browser_html.rs`
- `crates/fret-diag/src/commands/query.rs`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/diagnostics-first-open.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/diagnostics-first-open.md`
- `apps/fretboard/src/demos.rs`
- `apps/fretboard/src/cli/contracts.rs`
- `apps/fretboard/src/cli/help.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value/element.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value/options.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value/scrub_element.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value/typing.rs`
- `ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs`
- `ecosystem/fret-ui-editor/src/controls/slider.rs`
- `ecosystem/fret-imui/src/tests/mod.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `ecosystem/fret-imui/src/tests/models_controls.rs`
- `ecosystem/fret-imui/src/tests/models_combo.rs`
- `ecosystem/fret-imui/src/tests/models_text_basic.rs`
- `ecosystem/fret-imui/src/tests/models_text_lifecycle.rs`
- `ecosystem/fret-imui/src/tests/models_text_identity.rs`
- `ecosystem/fret-imui/src/tests/models_text_picker.rs`
- `ecosystem/fret-imui/src/tests/models_text_filters.rs`
- `ecosystem/fret-imui/src/tests/models_text_modes.rs`
- `ecosystem/fret-imui/src/tests/models_text_commands.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/main.rs`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag_gate_imui_p2_devtools_first_open.py`
- `tools/diag-campaigns/devtools-first-open-smoke.json`

## First-open repro surfaces

Use these when reopening the lane before diving into older notes:

1. Immediate generic/default proof
   - `cargo run -p fretboard-dev -- dev native --demo imui_action_basics --features cookbook-imui`
2. Immediate/editor proof
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
3. Editor notes workbench proof
   - `cargo run -p fret-demo --bin editor_notes_demo`
4. Adaptive editor notes shell proof
   - `cargo run -p fret-demo --bin editor_notes_device_shell_demo`
5. Workspace shell proof
   - `cargo run -p fret-demo --bin workspace_shell_demo`
6. DevTools proof
   - `cargo run -p fret-devtools`
7. Multi-window parity proof
   - `cargo run -p fret-demo --bin docking_arbitration_demo`

These are not the only relevant surfaces, but they give the fastest current read on the lane
without reopening older workstreams first.

## Current focused gates

### Immediate authoring / adapter gates

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-imui`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface`
- `python tools/gate_imui_facade_teaching_source.py`
- `python tools/diag_gate_action_first_authoring_v1.py --only cookbook-imui-action-basics-cross-frontend`

This package now locks the current immediate-mode product message at the source-policy layer:

- the golden pair is named explicitly,
- the nested-vs-root mounting rule stays explicit,
- the static-vs-dynamic stable-identity rule stays explicit,
- the reference/advanced/compatibility roster stays classified,
- the proof budget rule stays frozen before any future helper widening,
- focused item-local shortcuts now span direct pressables, popup/menu triggers, and
  combo/combo-model triggers at the ecosystem layer,
- and repeat keydown stays ignored by default unless `shortcut_repeat=true` is explicitly requested.
- the launched `imui_action_basics` cookbook proof now exercises command palette, declarative,
  GenUI, and IMUI action triggers against one shared typed action handler.

### Closed narrow closeout: child-region depth

- `cargo run -p fret-demo --bin workspace_shell_demo`
- `cargo run -p fret-demo --bin editor_notes_demo`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
- `cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on --no-fail-fast`

This package now proves the closed child-region closeout record owns:

- the current pane-first proof surfaces stay explicit,
- embedded menu + popup composition inside child content already works,
- the bounded `ChildRegionChrome::{Framed, Bare}` slice is executable at both the adapter seam and
  the focused `fret-imui` composition seam,
- and the remaining `BeginChild()`-scale pressure no longer justifies keeping a generic
  implementation queue active in this umbrella.

### Closed narrow closeout: menu/tab policy depth

- `cargo run -p fret-demo --bin imui_interaction_showcase_demo`
- `cargo run -p fret-demo --bin imui_response_signals_demo`
- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_menu_helper_hover_switches_top_level_popup_after_trigger_hover_delay begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics begin_submenu_helper_hover_opens_submenu_after_pointer_entry begin_submenu_helper_hover_switches_sibling_after_open_delay menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_bar_helper_reports_selected_change_and_trigger_edges --no-fail-fast`

This package remains the historical proof floor for the now-closed menu/tab lane:

- top-level menus are click-open and can hover-switch once a menubar session is active,
- submenus open, hover-open, sibling-switch with a basic grace corridor, and report outward
  trigger edges,
- and tab bars currently cover simple selection/panel switching rather than richer shell policy.

### Closed narrow closeout: collection delete action

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_delete_action_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_delete_action_follow_on proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item proof_collection_delete_selection_picks_previous_visible_item_at_end --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps collection delete-selected semantics explicit and app-owned,
- `Delete` / `Backspace` and the explicit button route through one proof-local delete helper,
- next selection plus keyboard active tile reflow stay reviewable at the unit-test layer,
- and broader collection command breadth still does not justify shared helper or runtime widening.

### Closed narrow closeout: collection context menu

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_context_menu_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_context_menu_follow_on proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_zoom_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on proof_collection_layout_metrics_fall_back_before_viewport_binding_exists proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor proof_collection_zoom_request_ignores_non_primary_wheel --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_select_all_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_select_all_follow_on proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile proof_collection_select_all_selection_falls_back_to_first_visible_asset proof_collection_select_all_shortcut_matches_primary_a_only --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_rename_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_rename_follow_on proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_inline_rename_follow_on proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps collection context-menu quick actions explicit and app-owned,
- right-click on item/background routes through one shared popup scope,
- right-click selection adoption plus delete reuse stay reviewable at the unit-test layer,
- collection zoom/layout metrics stay explicit and app-owned on the same proof surface,
- primary+wheel zoom and derived keyboard columns stay reviewable at the unit-test layer,
- collection select-all stays explicit and app-owned on the same proof surface,
- Primary+A plus visible-order-aware select-all stay reviewable at the unit-test layer,
- collection rename plus inline rename stay explicit and app-owned on the same proof surface,
- F2/context-menu rename posture plus label-only commit stay reviewable at the unit-test layer,
- and broader collection command breadth still does not justify shared helper or runtime widening.

### Closed narrow closeout: collection modularization

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on proof_collection_drag_rect_normalizes_drag_direction proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps the collection boundary explicit while delegating implementation to `collection.rs`,
- the collection module still exposes the full app-owned behavior surface and unit-test floor,
- the structural cleanup is reviewable independently from product-depth slices,
- and the next default non-multi-window priority is broader app-owned command-package depth rather than more host-file accretion.

### Closed narrow execution: collection command package

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on proof_collection_duplicate_shortcut_matches_primary_d_only proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only --no-fail-fast`

This package now proves:

- `collection.rs` owns the current broader command-package slices locally on the existing proof surface,
- duplicate-selected plus explicit rename-trigger routing stay app-owned across keyboard, explicit button, and context-menu paths without generic helper widening,
- command status feedback stays app-owned in the collection module,
- the command-package lane is closed without a third verb,
- and the closed second proof-surface record is now the evidence gate before any future
  helper-readiness follow-on can reopen shared collection helpers.

### Closed narrow closeout: collection second proof surface

- `cargo run -p fret-demo --bin editor_notes_demo`
- `cargo run -p fret-demo --bin workspace_shell_demo`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast`

This package now proves:

- the second proof-surface follow-on is closed after command-package closeout,
- `editor_notes_demo.rs` is the primary existing shell-mounted candidate,
- `editor_notes_demo.rs` now carries a `Scene collection` left-rail surface with stable collection
  summary/list test ids,
- `workspace_shell_demo.rs` stays supporting shell-mounted proof evidence,
- no dedicated asset-grid/file-browser demo is introduced yet,
- and shared helper/runtime widening stays closed because the two collection proof surfaces do not
  yet need the same reusable helper shape.

### Editor shell gates

- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo run -p fretboard-dev -- diag suite editor-notes-demo --launch -- cargo run -p fret-demo --bin editor_notes_demo`
- `cargo run -p fretboard-dev -- diag suite editor-notes-device-shell-demo --launch -- cargo run -p fret-demo --bin editor_notes_device_shell_demo`
- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`
- `cargo check -p fret-workspace`
- `cargo nextest run -p fret-ui declarative_internal_drag_region_can_install_route_anchor --no-fail-fast`
- `cargo nextest run -p fret-workspace workspace_pane_tree_installs_workspace_tab_drag_route_anchor --no-fail-fast`
- `cargo nextest run -p fret-workspace workspace_root_drop_after_tab_pointer_up_dispatches_split_and_move --no-fail-fast`
- `cargo nextest run -p fret-workspace pointer_click_on_inactive_tab_dispatches_activate --no-fail-fast`
- `cargo fmt --package fret-ui -- --check`
- `cargo fmt --package fret-workspace -- --check`

This package currently proves:

- `workspace_shell_demo` remains the primary coherent shell proof,
- `editor_notes_demo` remains the minimal shell-mounted rail proof,
- `editor_notes_demo` now has a promoted suite over preserved multiline draft behavior and
  app-owned draft controller commit/discard affordances plus asset selection -> inspector sync,
- `editor_notes_device_shell_demo` has its own promoted suite because it launches a different
  adaptive shell binary and proves desktop rails plus compact drawer reuse of the same editor
  content,
- the launched shell smoke floor now reaches beyond tabstrip-only checks,
- source-level workspace tab drag routing now keeps the root `DRAG_KIND_WORKSPACE_TAB` route anchor
  in `crates/fret-ui` while pane/zone/drop policy stays in `fret-workspace`,
- `PointerUp -> InternalDrag::Drop` can resolve a right-edge pane split from the root-routed
  workspace tab drag and then move the dragged tab into the generated pane,
- and the shell proof set does not silently collapse back into the generic `imui` backlog.

### Workspace shell tab-strip gates

- `cargo nextest run -p fret-workspace`
- `cargo run -p fret-demo --bin workspace_shell_demo --release`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json --dir target/fret-diag/workspace-reorder-first-to-end-2026-05-14-v3 --timeout-ms 240000 --exit-after-run --launch -- target/release/workspace_shell_demo.exe`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke.json --dir target/fret-diag/workspace-row-suppressed-2026-05-14-v3 --timeout-ms 240000 --exit-after-run --launch -- target/release/workspace_shell_demo.exe`
- `python tools/diag_gate_imui_product_chain.py --launched --only workspace-shell --release --out-dir target/imui-product-chain-launched-2026-05-14-workspace-shell-v3`

This package now proves:

- `WorkspaceTabDragState` is anchored at the root model identity, not a transient local model identity, so tab drag state survives pane-tree churn.
- local release on the tab strip claims end-drop and row-local drop before pane-level move/split arbitration can steal the gesture.
- tab-row hover keeps publishing `hovered_pane_tab_rects`, so the split-preview path no longer starves itself when the pointer sits inside the row.
- the reorder-first-to-end smoke now lands on `workspace-shell-pane-pane-a-tab-strip.drop_end` and reorders `doc-a-0` to `pos_in_set=3`.
- the row-suppressed smoke keeps pane B split previews absent while the pointer remains on the source row.
- the launched workspace-shell product chain stays green with `stage_counts: {"passed": 11}`.

Run evidence:

- `target/fret-diag/workspace-reorder-first-to-end-2026-05-14-v3/1778711172824-workspace-shell-demo-tab-reorder-first-to-end-smoke/script.result.json` reports `stage=passed`.
- `target/fret-diag/workspace-row-suppressed-2026-05-14-v3/1778711195977-workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke/script.result.json` reports `stage=passed`.
- `target/imui-product-chain-launched-2026-05-14-workspace-shell-v3/1778711236860/workspace-shell/suite.summary.json` reports `status=passed` and `stage_counts.passed=11`.

The promoted launched suite now freezes this minimum shell coverage:

- tab close / reorder / split preview,
- dirty-close prompt and discard close,
- content-focus restore via Escape,
- and left-rail / file-tree keep-alive.

The 2026-05-13 workspace tab split handoff source gate is not a replacement for the launched
`diag-hardening-smoke-workspace` suite. The launched inactive-tab drag-to-split-right smoke is now
closed with a release demo rebuild plus a packed diagnostics artifact:

```powershell
cargo build -p fret-demo --bin workspace_shell_demo --release
cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-inactive-to-split-right-smoke.json --dir target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15 --timeout-ms 180000 --exit-after-run --pack --ai-packet --launch -- target/release/workspace_shell_demo.exe
cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- target/release/workspace_shell_demo.exe
```

Run evidence:

- `target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15/1778688009999/script.result.json`
  reports `stage=passed`.
- `drag_pointer_until.start` resolved to `x=588.3334,y=14.666666`, hit
  `workspace-shell-pane-pane-a-tab-doc-a-2.chrome`, and set
  `hit_path_contains_intended=true`.
- Step 14 dispatches `workspace.tab.activate.doc-a-2`,
  `workspace.pane.split.horizontal.second.window-1.pane.1`, and
  `workspace.pane.move_active_tab_to.window-1.pane.1`, proving the inactive source tab moved into
  the generated pane rather than moving pane B's active tab.
- Packed share artifact:
  `target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15/share/1778688009999.zip`.

### Diagnostics / tooling gates

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
- `python tools/diag_gate_imui_product_chain.py`
- `python tools/diag_gate_imui_product_chain.py --only discovery`
- `cargo run -p fretboard-dev -- --help`
- `cargo run -p fretboard-dev -- list --help`
- `cargo build -p fret-devtools`
- `cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast`
- `cargo run -p fretboard-dev -- diag doctor campaigns`
- `cargo run -p fretboard-dev -- list tool-apps`
- `cargo run -p fretboard-dev -- list tool-apps --json`

This package currently proves:

- the P2 first-open path starts from CLI-compatible evidence production,
- the P2 diagnostics owner split stays explicit across runtime, tooling, GUI, and MCP surfaces,
- one repo-owned P2 smoke gate now proves the direct first-open loop with a real launched app,
- direct `diag run` leaves named bundle checkpoints and latest-bundle resolution through
  `script.result.json:last_bundle_dir`,
- direct `diag compare` remains a shared artifacts-layer verdict rather than a GUI-only diff mode,
- one bounded campaign root now proves explicit root `diag summarize`,
  aggregate `regression.summary.json` / `regression.index.json`, and `diag dashboard` over the
  same shared contracts,
- one canonical first-open doc now routes diagnostics readers before they open branch/reference
  notes,
- `apps/fret-devtools/src/native.rs` now mirrors that first-open route in the GUI shell via a
  `First-open Evidence Path` panel, so the GUI exposes the canonical doc, GUI branch doc, repo
  preflight, artifacts root, direct run/latest/compare loop, campaign summarize/dashboard loop,
  and bounded P2 smoke gate without inventing a second run model,
- `tools/diag_gate_imui_p2_devtools_first_open.py` now source-checks that GUI first-open projection,
- DevTools GUI and MCP stay aligned as consumers of the same artifacts root,
- `fretboard-dev list tool-apps` exposes the DevTools GUI and MCP launch commands as one
  repo-maintainer discovery surface,
- `fretboard-dev list tool-apps --json` exposes the same `fretboard_tool_apps` schema for
  automation and source-gate checks,
- the default product-chain discovery gate validates that top-level help points to
  `fretboard-dev list tool-apps` and `fretboard-dev list tool-apps --json`, and that `list --help`
  names `tool-apps` as the repo-maintainer tool-app index,
- the default product-chain discovery gate now validates that JSON shape, including `kind`,
  `schema_version`, canonical first-open/GUI docs, repo preflight commands, and GUI/MCP
  command/docs/gate/best-for fields, rather than checking only a few human-text markers,
- and compare remains a shared artifacts-layer contract instead of a GUI-only diff mode.
- captured immediate/runtime identity warnings now have a bounded first-open path through
  `diag query identity-warnings --browser --json`,
- the same identity warning report can be reviewed offline through `--html-out` and smoke-checked
  through `--html-check-out`,
- and the committed schema2 sample bundle lets maintainers exercise that path without launching a
  demo first.

Latest DevTools GUI first-open source projection proof (2026-05-14):

- `cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast`
  passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-gui-source-2026-05-14`
  passed, including the new `fret-devtools gui first-open source` step.
- Run root:
  `target/imui-p2-devtools-first-open-gui-source-2026-05-14/1778733748418`.
- Campaign root:
  `target/imui-p2-devtools-first-open-gui-source-2026-05-14/1778733748418/campaign/campaigns/devtools-first-open-smoke/1778733762096`.

DevTools GUI product-workflow projection follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now projects the shared `imui-product-chain` route in the
  GUI first-open evidence panel: default command, focused discovery command, launched
  `perf-docking` command, `perf-docking-arbitration-steady` suite, product-closure docs, and
  `perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, plus
  `perf-docking/*/trace.chrome.json`.
- The default product-chain discovery gate now source-checks that GUI projection, so
  `fretboard-dev list tool-apps`, `fretboard-dev list tool-apps --json`, and the DevTools GUI
  first-open panel cannot silently diverge on the product workflow route.
- Focused source gates:

```text
cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

## IMUI text picker keyboard owner split - 2026-05-25

Scope: keep `text_picker_controls.rs` as the input/popup orchestration owner while moving picker
keyboard state, active-index movement, and Enter/NumpadEnter commit policy into a focused owner.

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard.rs` now owns
  `InputTextPickerKeyboardState`, `InputTextPickerKeyboardPick`, arrow-key wrap movement through
  `cmdk_selection::next_active_index(...)`, repeat/modifier/IME guards, and keyboard commit writes
  to the text model plus popup-open model.
- `text_picker_controls.rs` keeps current-value reads, candidate filtering, input semantics,
  popup-open orchestration, candidate rendering, pointer selection, response lifecycle merging, and
  active-descendant wiring.
- The split leaves `text_picker_controls.rs` at 300 lines and the new `keyboard.rs` owner at 106
  lines.
- The source gate now rejects keyboard state/handler bodies from returning to
  `text_picker_controls.rs`, while checking that `keyboard.rs` does not grow popup or selectable UI
  composition policy.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui models_text --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui text_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The first two `fret-imui models_text` attempts failed at Windows link time
with unresolved `hashbrown`/`fret_core::input::Event` symbols after the earlier interrupted
`nextest list` commands; `cargo clean -p fret-imui` cleared the stale test artifacts, and the rerun
reported `29 tests run: 29 passed`. The `fret-ui-kit` focused text-controls gate reported
`3 tests run: 3 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. `git diff --check`
reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI debug draw element owner split - 2026-05-25

Scope: keep `debug_draw_controls.rs` as the draw-list API and response/model hub while moving the
final canvas/pressable element assembly into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element.rs` now owns `debug_draw_element`,
  fill-layout policy, canvas cache policy, clip-to-bounds dispatch, and forwarding into
  `paint_debug_draw_commands`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element/behavior.rs` now owns pressable
  wrapping, click response population, and keyboard activation lifecycle marking.
- `debug_draw_controls.rs` still owns `DebugDrawOptions`, `DebugDrawResponse`,
  `ImUiDebugDrawList`, stroke/corner options, command recording, summary construction, and the
  facade entrypoint.
- The split reduces `debug_draw_controls.rs` from 1418 lines to 1285 lines in this worktree and
  keeps the new element owner at 142 lines.
- The source gate now rejects pressable/canvas element assembly and direct paint dispatch from
  `debug_draw_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw_default_element_stays_noninteractive_canvas debug_draw_interaction_wraps_canvas_in_pressable_response_surface debug_draw_options_default_to_clipped_canvas --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI debug draw path-builder owner split - 2026-05-25

Scope: keep `debug_draw_controls.rs` focused on the draw-list API and command recorder while moving
the high-level path DSL into a dedicated owner module.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder.rs` now owns
  `ImUiDebugDrawPath`, line/rect/bezier/arc/elliptical-arc path building, path stroke/fill
  finalization, and the glue to low-level path sampling helpers.
- `debug_draw_controls.rs` still exposes the same `ImUiDebugDrawList::path(...)` authoring entry
  and still owns the command recorder methods.
- The split reduces `debug_draw_controls.rs` from 1151 lines after the element split to 1078 lines;
  the new `path_builder.rs` owner is 220 lines.
- The source gate now rejects path-builder implementation details from `debug_draw_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw_path_builder --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI debug draw draw-list owner split - 2026-05-25

Scope: keep `debug_draw_controls.rs` as the options/response/style entrypoint hub while moving the
command recorder, channel merge, and command/list summary logic into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list.rs` now owns the
  `ImUiDebugDrawList` implementation: `path`, channel split/switch/merge, command summaries,
  list summary, all `add_*` command recorder helpers, and `Default`.
- `debug_draw_controls.rs` keeps type declarations (`DebugDrawOptions`, `DebugDrawResponse`,
  `DebugDrawStrokeStyle`, `DebugDrawRoundCorners`, image/svg option types, `DebugDrawVertex`) and
  the `debug_draw_with_options` facade entrypoint.
- `path_builder.rs` keeps the path DSL, `element.rs` keeps element assembly, `commands.rs` keeps
  command/summary contracts, `paint.rs` now imports low-level path helpers directly from `paths`
  instead of depending on parent-module re-exports.
- The split reduces `debug_draw_controls.rs` from 1078 lines after the path-builder split to 419
  lines; the new `draw_list.rs` owner is 671 lines.
- The source gate now rejects command recorder, channel, summary, and sequential triangle index
  logic from returning to `debug_draw_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw_list --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_channels --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The first `debug_draw_list` attempt timed out while compiling test
artifacts, then passed on rerun with `7 tests run: 7 passed`; `debug_draw_channels` reported
`2 tests run: 2 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.

## IMUI debug draw draw-list shape-method owner split - 2026-05-25

Scope: keep `draw_list.rs` focused on path/channel/summary/clip/image/svg/text command flow while
moving basic ImDrawList-style geometry authoring methods into a shape-method owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes.rs` now owns line,
  polyline, convex/concave polygon fill, rect/quad/triangle/mesh, circle/ngon/ellipse, and bezier
  command recorder methods for `ImUiDebugDrawList`.
- `draw_list.rs` still owns path entry, channel split/switch/merge, summary construction,
  clip-stack commands, image/svg/text command recorders, command count, and `Default`.
- The split reduces `draw_list.rs` from 672 lines after the earlier draw-list split to 308 lines;
  the new `draw_list_shapes.rs` owner is 371 lines.
- The source gate now rejects basic shape recorder methods and sequential triangle index generation
  from returning to `draw_list.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw_list --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_channels --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `debug_draw_list` focused gate reported `7 tests run: 7 passed`; the
`debug_draw_channels` focused gate reported `2 tests run: 2 passed`. `cargo check` reported only
the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI debug draw summary-contract owner split - 2026-05-25

Scope: keep `commands.rs` focused on private debug-draw command payloads and command-to-summary
mapping while moving public summary contracts into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries.rs` now owns
  `DebugDrawCommandKind`, `DebugDrawCommandSummary`, and `DebugDrawListSummary`, including
  accessor-first public metrics plus internal constructors/aggregation helpers.
- `commands.rs` keeps `DebugDrawCommand` payload variants and `summary_with_clip_state(...)` /
  `summary(...)`, using the summary owner instead of defining public summary contracts inline.
- `debug_draw_controls.rs` re-exports the public summary types from `summaries`, preserving the
  existing external `fret_ui_kit::imui::*` API shape.
- The split reduces `commands.rs` from 644 lines before the summary split to 401 lines; the new
  `summaries.rs` owner is 245 lines.
- The source gate now moves opaque summary-struct coverage to `summaries.rs` and rejects public
  summary contracts from returning to `commands.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw_list --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw_channels --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `debug_draw_list` focused gate reported `7 tests run: 7 passed`; the
`debug_draw_channels` focused gate reported `2 tests run: 2 passed`. `cargo check` reported only
the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI debug draw paint-helper owner split - 2026-05-25

Scope: keep `debug_draw_controls/paint.rs` as the command-to-painter dispatcher while moving pure
paint helpers for image opacity/UV validation, rounded image clipping, triangle meshes, and image
scene ops into a smaller helper owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers.rs` now owns
  `normalized_opacity`, `uv_rect_is_valid`, `corner_radii_are_visible`,
  `rounded_rect_corner_radii`, `paint_triangle_mesh`, `paint_image_triangle_mesh`, `paint_image`,
  and `paint_image_region`.
- `paint.rs` keeps `paint_debug_draw_commands` and imports those helpers instead of owning mesh and
  image scene-op emission directly.
- Existing tests now import paint-helper proof functions from `paint_helpers`, keeping unit proof
  close to the new owner.
- The split reduces `paint.rs` from 729 lines after the draw-list split to 589 lines; the new
  `paint_helpers.rs` owner is 148 lines.
- The source gate now rejects opacity/UV/rounded-image/mesh/image helper bodies from returning to
  `paint.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui image_overlay_helpers_sanitize_opacity_and_uv_rects rounded_image_helpers_follow_imgui_path_rect_corner_rules --no-fail-fast
python tools/gate_imui_workstream_source.py
```

DevTools GUI demo/metrics/debug route follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now surfaces a persistent `demo-metrics-debug` route in the
  GUI shell, separate from runtime/API work in `fret-imui`.
- The route names the current editor demos (`imui_editor_proof_demo`, `editor_notes_demo`, and
  `editor_notes_device_shell_demo`) plus existing diagnostics metrics/debug entrypoints:
  `diag stats`, `diag layout-perf-summary`, `diag memory-summary`, `diag triage`, and
  `diag hotspots`.
- Focused source gates:

```text
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

DevTools GUI first-class gate command follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now surfaces a `Gate Commands` block in the first-open GUI
  shell for stale paint/scene, pixels-changed, perf-threshold, and resource-footprint diagnostics
  entrypoints.
- The selected-summary inspector now also consumes the shared `fret-diag` regression-bundle
  follow-up projection, generating concrete commands from the selected `bundle_dir`: `diag stats`,
  `diag layout-perf-summary`, `diag memory-summary`, `diag triage`, `diag hotspots`,
  `diag trace`, visual compare, and footprint compare.
- That projection is now structured: direct bundle-local commands carry concrete `diag_args`, while
  visual/footprint compare commands are marked as baseline-required manual follow-ups. GUI and MCP
  consumers can therefore separate runnable actions from placeholder compare templates.
- `apps/fret-devtools/src/followup.rs` now launches the runnable subset through
  `fret_diag::diag_cmd` on a background job and records in-flight/error status back into the GUI.
  The baseline-required compare commands are rejected by the focused unit gate instead of being
  treated as runnable.
- 2026-05-21 maintenance: baseline-required visual and footprint compare templates now carry
  `target_bundle_dir` from the shared `fret-diag` projection. Regression Workspace exposes
  `Baseline Compare Actions` that accept a baseline bundle/directory or footprint session,
  materialize the existing `diag compare ... --json` / `diag compare ... --footprint --json`
  command, and launch it through the same follow-up runner.
- Compare follow-up result records are keyed to the candidate bundle rather than the baseline, so
  selected-bundle history, summary, copy, and open actions remain attached to the failing evidence
  the maintainer selected.
- Each launched follow-up writes a lightweight `.fret/diag/followups/*.json` result record with
  schema/kind, command metadata, `diag_args`, pass/fail status, optional error, and timing fields.
  The GUI exposes the latest result path so the evidence can be copied without hunting through logs.
- The selected-summary inspector mirrors the latest selected-bundle result JSON inline in a
  `Follow-up Result JSON` section, keeping the quick pass/fail/error/timing read inside the
  DevTools surface.
- The inspector also projects the latest selected-bundle result JSON into a
  `Follow-up Result Summary` section above the raw payload, keeping status, command, duration, and
  error preview scannable in the GUI.
- A bounded `Follow-up Result History` section filters recent GUI-launched follow-up results to the
  selected bundle, preventing a previous bundle's global-last result from being read as current
  selected-summary evidence.
- DevTools startup now restores recent valid `.fret/diag/followups/*.json` records into that same
  bounded history while preserving the selected-bundle filter before any result is shown or copied.
- The history section now renders selectable result entries; selecting an older matching entry
  changes the summary/raw JSON/copy target while preserving newest-first fallback.
- A `Follow-up Result Details` block surfaces the selected result's status, path, command, bundle,
  and error preview, and a copy action exposes the exact command that produced that artifact.
- The selected follow-up JSON artifact can be opened through the platform URL handler via an
  escaped file URL projection, keeping native artifact inspection one click away where supported and
  preserving paths containing spaces, fragments, or non-ASCII bytes.
- The follow-up result copy action resolves the selected bundle's latest history path and refuses
  when no selected-bundle result exists, rather than copying the global last artifact.
- The same inspector can copy the selected bundle's follow-up JSON payload directly, so issue
  reports and AI-assisted triage can use the exact payload shown in the panel.
- This is a DevTools/diagnostics productization slice: it keeps existing `fretboard-dev diag`
  commands visible without moving gate policy into `fret-ui` or `fret-imui`.
- 2026-05-16 maintenance: the same shared projection now includes runnable selected-bundle
  `diag trace <bundle> --json` actions in GUI and MCP surfaces, keeping Chrome trace artifact
  generation in the diagnostics owner lane.
- 2026-05-16 maintenance: GUI-launched trace follow-up result records now include
  `output_artifacts[].path` for the generated `trace.chrome.json`, and the selected-result summary
  and detail blocks surface that artifact path for reuse.
- Focused source gates:

```text
cargo nextest run -p fret-diag regression_bundle_followup_command_lines_use_selected_bundle_dir --no-fail-fast
cargo nextest run -p fret-diag regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
cargo nextest run -p fret-diag regression_bundle_followup_commands_cover_each_selected_bundle --no-fail-fast
cargo nextest run -p fret-devtools regression_followup_command_rejects_baseline_required_commands regression_followup_command_returns_direct_diag_args regression_followup_result_record_has_stable_shape regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_status_and_duration regression_followup_result_summary_lines_project_output_artifacts regression_followup_result_history_summary_filters_to_selected_bundle load_recent_followup_result_history_reads_latest_valid_records regression_followup_result_history_latest_path_prefers_selected_bundle regression_followup_result_history_selected_entry_overrides_latest_when_matching regression_followup_result_history_entry_detail_lines_surface_repro_fields file_url_from_path_projects_native_artifact_paths runnable_followup_command_action_lines_surface_indexed_bundle_commands --no-fail-fast
cargo nextest run -p fret-devtools regression_followup_compare_result_uses_candidate_bundle_dir materialize_baseline_compare_followup_command_fills_diag_args selected_followup_readiness_lines_summarize_next_runnable_command runnable_followup_command_action_lines_surface_indexed_bundle_commands --no-fail-fast
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

DevTools GUI workflow-run productization follow-up (2026-05-21):

- `apps/fret-devtools/src/native.rs` now surfaces a `Workflow Runs` panel in the Guide, separate
  from the gate-command builder and regression follow-up inspector.
- `apps/fret-devtools/src/workflow_run.rs` owns the thin GUI job/result wrapper over
  `fret_diag::diag_cmd`; it does not introduce a GUI-only campaign or suite runtime model.
- The result contract kind is `fret_devtools_workflow_run_result`.
- The initial presets cover `diag campaign validate tools/diag-campaigns/devtools-first-open-smoke.json --json`,
  `diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`, and a
  selected-session `diag suite perf-docking-arbitration-steady ... --devtools-session-id <selected-session> --json`.
- Each GUI-launched workflow writes `.fret/diag/workflow-runs/*.json` with status, timing, command,
  redacted `diag_args`, and selectable/copyable/openable history in the panel. The stored JSON and
  preview command redact `--devtools-token`.
- DevTools startup now restores the latest valid workflow-run result records from
  `.fret/diag/workflow-runs/*.json`, skipping malformed JSON and non-workflow records. Reopening
  the GUI preserves the selected workflow evidence handoff instead of requiring maintainers to
  rerun a suite before loading summary/index artifacts.
- Suite workflow records also include structured `output_artifacts[]` for shared diagnostics
  handoff paths. The selected-session `perf-docking-arbitration-steady --dir ...` workflow records
  `suite.summary.json` and `regression.summary.json`, and the Workflow Result Summary / Details
  panels render those artifact lines before raw JSON. The selected workflow's `suite.summary.json`
  path is copyable/openable directly, and its `regression.summary.json` path is also
  copyable/openable directly from the Workflow Result Details actions, with relative artifact paths
  resolved against the repo root before platform URL opening. The same action cluster can load that
  summary into the existing Regression Workspace selection model, so GUI-launched suite evidence
  immediately feeds the existing bundle follow-up, perf evidence, capability provenance, and
  share-artifact drill-down surfaces.
- The first-open next-action summary distinguishes an aggregate regression index from a loaded
  selected summary, so GUI-launched suite handoff can truthfully report that follow-up actions are
  ready without pretending that `regression.index.json` is present.
- Regression Workspace now includes a compact `Follow-up Readiness` projection over the selected
  summary: selected bundle count, runnable follow-up count, manual compare count, and the first
  runnable command. This makes a workflow-loaded suite summary immediately actionable before the
  maintainer reads the full command list.
- After a selected-bundle follow-up result exists, the first-open next-action summary reports the
  selected follow-up result state and points maintainers at Follow-up Result Summary/History,
  keeping the workflow-suite handoff visible after the next diagnostics command has produced an
  artifact.
- 2026-05-21 maintenance: the Guide now starts with `Recent Evidence`, a compact restored-history
  projection over generated gate, workflow, and selected-bundle follow-up result artifacts. It
  shows the latest artifact in each lane, counts recent failing evidence, includes the full failed
  evidence artifact path and failed follow-up `bundle_dir` in the compact report, and points
  maintainers at the next evidence action without adding a GUI-private artifact store. The block also
  exposes select/copy/open actions for the compact report, first failed evidence artifact, failed
  follow-up bundle directory when present, and its producing command, plus direct copy of the
  restored failed result JSON payload. The block can also rerun failed evidence when the result JSON
  carries runnable structured `diag_args`; workflow reruns always re-materialize the same workflow
  id from the current selected session and current DevTools token instead of reusing stored workflow
  args, while missing args, unknown workflow ids, or missing session keep rerun disabled and now
  surface the concrete unavailable reason in the compact report and first-open header. Display
  command strings do not become an executable protocol. The compact report uses the same
  state-aware rerun decision as the button, and its `recent_evidence_next_action` now projects the
  concrete repair step (rerun, select a session, refresh workflow commands, run a current workflow,
  or inspect the result JSON) instead of a generic history-inspection hint. This keeps copied
  evidence aligned with first-open action availability and keeps first-open failure triage
  artifact-first while reusing the existing
  Gate/Workflow/Follow-up history state. Follow-up selection carries its `bundle_dir` into
  Regression Workspace so selected follow-up history stays aligned with the failed artifact. The
  `First-open Next Actions` header now also reports restored failed evidence plus its rerun command,
  current rerun availability, and the same `recent evidence next` repair step as the Guide report.
  The header also mirrors the Guide's copy/select/rerun actions as first-open shortcut buttons,
  reusing the same command ids and disabled-state rules instead of creating a header-only execution
  path. This keeps the current failure and next action visible and directly reachable before the
  maintainer opens the Guide.
- 2026-05-21 maintenance: `First-open Next Actions` now also reports the selected diagnostics
  session scope. The header distinguishes no session, connected-but-unselected sessions, one
  selected session, and multiple connected sessions where the Session selector retargets inspect,
  bundle, screenshot, and selected-session suite workflow actions. This is a DevTools GUI
  discoverability improvement over the existing selection model, not a transport or `fret-imui`
  contract change. The underlying v1 session rule is now source-tested in `apps/fret-devtools/src/ws.rs`:
  keep a valid selected session, otherwise fall back to the first advertised session, and filter
  session-scoped payloads to that selection while no-selection remains the initial compatibility
  state.
- Workflow Runs also renders `Workflow Handoff Readiness`, a compact next-action projection over
  the selected workflow result, the `regression.summary.json` handoff artifact, and whether that
  summary is already loaded into Regression Workspace. This keeps the GUI thin over
  `fret_diag::diag_cmd` while making the result-to-follow-up transition explicit.
- 2026-05-21 maintenance: the same readiness block now reports `aggregate_index_loaded` and a
  separate `aggregate_next_action`. A ready `regression.index.json` therefore points maintainers at
  `Load workflow regression index` until the aggregate workspace has actually loaded the workflow
  artifact root, instead of treating index existence as equivalent to loaded aggregate state.
- 2026-05-21 maintenance: Workflow Runs now also renders `Workflow Summarize Handoff`. For a
  selected suite result, it derives the shared
  `diag summarize <regression.summary.json> --dir <same-dir> --json` command, exposes copy/run
  actions, and reports `aggregate_index_ready` separately from selected-summary readiness. This
  avoids treating `regression.index.json` as a direct suite artifact while still letting the GUI
  complete the aggregate handoff through the shared diagnostics engine.
- Workflow summarize result records now project both `regression.summary.json` and
  `regression.index.json` through `output_artifacts[]`. The result summary/details panels therefore
  expose the generated aggregate index as a first-class handoff artifact instead of relying on path
  inference from the command string.
- If the workflow `regression.index.json` artifact exists, Workflow Runs can copy/open that
  aggregate index directly or load it into the existing Regression Workspace by setting the
  aggregate artifacts root to the index parent directory and calling the shared refresh path. This
  keeps aggregate browsing on the existing `regression.index.json` consumer instead of adding a
  workflow-private parser.
- Focused gates:

```text
cargo nextest run -p fret-devtools workflow_run_result_record_has_stable_shape_and_redacts_token workflow_run_result_summary_lines_project_status_and_duration workflow_run_result_summary_lines_project_output_artifacts workflow_run_regression_summary_artifact_path_extracts_output_artifact workflow_run_result_history_selects_explicit_path_or_latest workflow_run_result_history_entry_detail_lines_surface_output_artifacts load_recent_workflow_run_result_history_reads_latest_valid_records workflow_run_command_reports_runnable_from_missing_inputs_and_args devtools_workflow_run_lines_surface_campaign_and_suite_entrypoints devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws devtools_first_open_next_action_lines_prioritize_stateful_workflow first_open_recent_evidence_action_specs_gate_disabled_states devtools_recent_evidence_lines_surface_restored_histories recent_evidence_next_action_projects_rerun_and_repair_steps devtools_recent_evidence_lines_use_current_workflow_state_for_rerunnable_status devtools_recent_evidence_lines_surface_failed_followup_bundle_dir recent_failed_evidence_bundle_dir_filters_empty_bundle_dir recent_failed_evidence_rerun_command_uses_structured_diag_args recent_failed_evidence_rerun_command_rejects_redacted_workflow_args recent_failed_evidence_rerun_reason_reports_diag_args_issues recent_failed_evidence_rerun_command_recovers_redacted_workflow_from_current_state recent_failed_evidence_rerun_command_uses_current_workflow_state_over_stored_args recent_failed_evidence_rerun_reason_reports_unregistered_workflow recent_failed_evidence_rerun_command_projects_followup_bundle devtools_recent_failed_evidence_target_prefers_visible_latest_then_history devtools_recent_failed_evidence_target_carries_result_json_payload devtools_recent_evidence_selection_effect_routes_to_existing_history_state file_url_from_path_projects_workflow_artifact_paths --no-fail-fast
cargo nextest run -p fret-devtools workflow_run_result_summary_lines_project_summarize_output_artifacts workflow_run_regression_summary_artifact_path_extracts_output_artifact workflow_summarize_command_from_summary_path_targets_same_dir workflow_regression_index_parent_dir_targets_artifact_root workflow_aggregate_index_loaded_matches_loaded_artifact_root workflow_regression_index_action_ids_cover_copy_open_load workflow_handoff_readiness_lines_project_next_action --no-fail-fast
cargo nextest run -p fret-devtools workflow_summarize_command_from_summary_path_targets_same_dir workflow_handoff_readiness_lines_project_next_action workflow_aggregate_index_loaded_matches_loaded_artifact_root devtools_workflow_run_lines_surface_campaign_and_suite_entrypoints --no-fail-fast
cargo nextest run -p fret-devtools devtools_workflow_run_lines_surface_campaign_and_suite_entrypoints devtools_workflow_commands_mark_suite_ws_missing_without_session devtools_workflow_commands_include_selected_session_for_suite_ws workflow_run_regression_summary_artifact_path_extracts_output_artifact --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
```

DevTools GUI perf-evidence drill-down follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now extracts selected regression summary perf evidence into a
  dedicated `Perf Evidence` section above raw JSON.
- The shared projection owner is now `crates/fret-diag/src/regression_summary.rs`
  (`regression_summary_drilldown`); the GUI only reads the summary JSON and renders the shared
  drill-down fields.
- The drill-down surfaces `perf_summary_json`, `compare_json`, curated metrics such as
  `top_total_time_us`, `top_renderer_encode_scene_us`, `top_renderer_instance_bytes`, and
  `threshold_failures` counts/JSON for selected summaries.
- Focused source gates:

```text
cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence --no-fail-fast
cargo nextest run -p fret-devtools load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast
python tools/gate_imui_workstream_source.py
```

DevTools MCP product-workflow projection follow-up (2026-05-15):

- `apps/fret-devtools-mcp/src/native.rs` now exposes `fret-diag://first-open.md` as a sessionless
  text resource and points MCP server instructions at that resource.
- The MCP first-open resource mirrors the shared `imui-product-chain` route: default command,
  focused discovery command, launched `perf-docking` command, `perf-docking-arbitration-steady`
  suite, product-closure docs, and `perf-docking/regression.summary.json`,
  `perf-docking/check.perf_thresholds.json`, plus `perf-docking/*/trace.chrome.json`.
- `fret_diag_regression_dashboard` now consumes the shared `fret-diag` regression drill-down and
  follow-up command projection, returning bundle dirs, capability provenance, perf evidence, and
  follow-up command lines instead of maintaining a MCP-private regression evidence parser.
- The MCP dashboard result also exposes `runnable_followup_command_lines` and
  `manual_followup_command_lines`, mirroring the GUI's separation between direct bundle-local
  follow-ups and baseline-required compare follow-ups.
- The same result now exposes structured `followup_commands`, `runnable_followup_commands`, and
  `manual_followup_commands` rows with `diag_args`, so AI consumers can run bundle-local actions
  like `trace` without parsing command-line strings.
- The default product-chain discovery gate now source-checks the MCP projection alongside the GUI
  first-open panel, so `fretboard-dev list tool-apps`, the GUI shell, and the MCP adapter cannot
  silently diverge on the product workflow route.
- Focused source gates:

```text
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

### Multi-window hand-feel gates

- `python tools/gate_imui_workstream_source.py`
- `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- `cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`
- `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release`
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics no_frame_pointer_move --no-fail-fast`
- Local refresh evidence: `docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`
- Launched campaign repair evidence: `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`

This package currently proves:

- one bounded P3 campaign now names hovered-window, peek-behind, transparent payload, and
  mixed-DPI follow-drag as one lane-owned package,
- `docking_arbitration_demo` is the launched proof surface for that package,
- the four expectations map to four repo-owned scripts instead of one vague docking smoke story,
- local source-policy, campaign validation, Wayland fallback, window-style capability, script
  roundtrip, and diagnostics predicate gates were refreshed on 2026-05-13,
- the launched bounded P3 campaign now passes 4/4 scripts after the diagnostics runner
  no-frame pointer-move repair in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
- the focused `no_frame_pointer_move` unit gate locks the fallback to active cross-window dock-panel
  or dock-tabs drags with an active pointer session,
- the product-chain perf entrypoint now runs `diag perf perf-docking-arbitration-steady` against
  `docking_arbitration_demo` and verifies `regression.summary.json` records two passing
  `perf_case` items with readable bundle artifacts and a readable shared `layout.perf.summary.v1.json`
  artifact, a readable shared `check.perf_thresholds.json` artifact, empty threshold failures, and
  curated `evidence.extra.metrics` rather than trusting process exit alone,
- the same product-chain perf entrypoint now passes `--trace-real-spans` and requires each
  perf-case bundle to expose a readable `trace.chrome.json` with `kind=perf_trace_chrome`,
  `trace_source=bundle_synthetic_phases_with_extension_spans`, `real_spans_included=true`, a
  positive `real_span_event_count`, and the `fret.perf.spans.v1` extension key,
- and `diag-hardening-smoke-docking` remains the small generic docking smoke entry rather than the
  IMUI lane's new umbrella package.

The first product-chain docking perf run on 2026-05-14 exposed a diagnostics tooling contract bug:
`diag perf` printed human `PERF ...` rows, but its `regression.summary.json` synthesized
`tooling.diag_perf.no_rows` unless `--json` was used. The fix is in `crates/fret-diag/src/diag_perf.rs`:
row evidence is now recorded for summaries regardless of stdout mode, while `--json` only controls
stdout formatting. The follow-up artifact projection repair keeps single-run `bundle` rows visible
as `bundle_artifact` evidence in the regression summary, and the metrics projection keeps
`top_*`, pointer-move, and renderer fields available to DevTools/GUI/MCP first-open summary readers
without opening the large bundle. The focused source gates are:

```text
cargo nextest run -p fret-diag perf_regression_summary_uses_rows_when_stdout_is_human --no-fail-fast
cargo nextest run -p fret-diag perf_row_to_regression_item_uses_single_run_bundle_artifact --no-fail-fast
cargo nextest run -p fret-diag perf_row_to_regression_item_projects_single_run_metrics perf_row_to_regression_item_projects_repeat_stats_metrics --no-fail-fast
```

Latest local docking perf entrypoint evidence (2026-05-14):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-metrics-gate-2026-05-14`
- `target/imui-product-chain-perf-docking-metrics-gate-2026-05-14/1778775354481/perf-docking/regression.summary.json` reports
  `items_total=2`, `passed=2`, and `failed_tooling=0`.
- The two items are `perf_case` rows for
  `docking-arbitration-demo-nary-splitter-drag-perf-large-layout-steady.json` and
  `docking-arbitration-demo-nary-tab-drag-hover-perf-large-layout-steady.json`.
- The product-chain gate now checks the item scripts against
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, requires each item to
  expose a readable `bundle_artifact`, and requires the shared `layout.perf.summary.v1.json` artifact
  to parse as a `layout_perf_summary` for one of the recorded bundles. It also requires curated
  `evidence.extra.metrics` fields such as `top_total_time_us`, pointer-move dispatch/hit-test, and
  renderer encode/instance metrics (`top_renderer_encode_scene_us`,
  `top_renderer_instance_bytes`).

Latest local docking perf threshold evidence (2026-05-15):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-threshold-gate-2026-05-15`
- `target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, `failed_tooling=0`, and `wants_perf_thresholds=true`.
- `target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/check.perf_thresholds.json`
  reports `kind=perf_thresholds`, `observed_aggregate=max`, and `failures=[]`.
- The product-chain gate now launches `diag perf` with conservative CPU/layout/pointer thresholds:
  `--max-top-total-us 20000`, `--max-top-layout-us 10000`, `--max-top-solve-us 10000`,
  `--max-pointer-move-dispatch-us 5000`, `--max-pointer-move-hit-test-us 5000`, and
  `--max-pointer-move-global-changes 0`.
- The gate validates that each regression item exposes readable `compare_json` evidence, that both
  rows in `check.perf_thresholds.json` match
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, and that all row
  threshold sources are `cli`. This turns the previous readable metric projection into a conservative
  product-chain perf threshold gate.

Renderer threshold follow-up evidence (2026-05-15):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15`
- `target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, `failed_tooling=0`, and empty item `threshold_failures`.
- `target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/check.perf_thresholds.json`
  reports `failures=[]` and `threshold_sources` of `cli` for renderer metrics including
  `max_renderer_encode_scene_us`, `max_renderer_upload_us`, `max_renderer_record_passes_us`,
  `max_renderer_encoder_finish_us`, `max_renderer_prepare_text_us`, `max_renderer_prepare_svg_us`,
  `max_renderer_instance_bytes`, and `max_renderer_encode_scene_text_ops`.
- `diag perf` now exposes renderer threshold CLI flags, including
  `--max-renderer-encode-scene-us`, `--max-renderer-upload-us`,
  `--max-renderer-record-passes-us`, `--max-renderer-encoder-finish-us`,
  `--max-renderer-prepare-text-us`, `--max-renderer-prepare-svg-us`,
  `--max-renderer-instance-bytes`, and `--max-renderer-encode-scene-text-ops`.
- Focused source gates:

```text
cargo nextest run -p fret-diag contract_help_mentions_the_migrated_command_surfaces migrated_perf_subset_builds_a_real_perf_context perf_thresholds_json_projects_renderer_thresholds --no-fail-fast
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15
```

Trace attribution gate refresh (2026-05-16):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16`
- The product-chain gate now invokes `diag perf perf-docking-arbitration-steady` with
  `--trace-real-spans`, which requires `--launch` and injects `FRET_DIAG_REAL_SPANS=1` into the
  launched `docking_arbitration_demo` process unless the caller explicitly overrides it.
- The gate requires each regression item bundle to have a sibling `trace.chrome.json` and validates
  `kind=perf_trace_chrome`, `trace_source=bundle_synthetic_phases_with_extension_spans`,
  `real_spans_included=true`, positive `real_span_event_count`, non-empty `traceEvents`, and the
  `fret.perf.spans.v1` extension key.
- Runtime capture repair: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` now owns
  `UiRealPerfSpanCaptureV1`, including the `FRET_DIAG_REAL_SPANS` env gate, sub-microsecond
  rounding, and the service flush into `fret.perf.spans.v1`. Both the shared
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs` path and the custom
  `apps/fret-examples/src/docking_arbitration_demo.rs` render path use that helper, so launched
  perf-docking bundles do not lose real spans by bypassing the golden-path driver.
- Service coverage:
  `record_snapshot_includes_recorded_real_perf_spans_extension` verifies that recorded spans land
  in the next diagnostics snapshot extension before trace export reads it.
- Trace exporter repair: `crates/fret-diag/src/trace.rs` now keeps consuming
  `fret.perf.spans.v1` even when the synthetic `total_time_us`/phase counters are zero; the
  regression test is `chrome_trace_keeps_real_span_extension_when_synthetic_stats_are_zero`.
- This is still a bounded `perf-docking` product-chain attribution gate. It does not close broad
  smoothness attribution across every editor workload.

Focused source gates:

```text
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics perf_span_capture_preserves_sub_microsecond_phase perf_span_capture_ignores_zero_duration_phase record_snapshot_includes_recorded_real_perf_spans_extension --no-fail-fast
cargo nextest run -p fret-diag chrome_trace_keeps_real_span_extension_when_synthetic_stats_are_zero --no-fail-fast
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16-debug
```

Debug trace probe evidence (2026-05-16):

- Direct command, intentionally without release threshold flags:
  `target/debug/fretboard-dev.exe diag perf perf-docking-arbitration-steady --dir target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking --repeat 1 --warmup-frames 5 --trace-real-spans --reuse-launch --env FRET_DOCK_ARB_PRESET=large --env FRET_DOCK_ARB_NO_PERSIST=1 --env FRET_DOCK_ARB_DISALLOW_DROP_TARGETS=1 --launch -- target/debug/docking_arbitration_demo.exe`
- Output traces:
  `target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking/1778897554296/trace.chrome.json`
  (`real_span_event_count=40`) and
  `target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking/1778897571346/trace.chrome.json`
  (`real_span_event_count=45`).
- Both traces validate with `_validate_docking_perf_trace` and report
  `trace_source=bundle_synthetic_phases_with_extension_spans`,
  `real_spans_included=true`, and first real events from `docking_arbitration_demo`.

Canonical release gate evidence (2026-05-16):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix`
- Output:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, and `failed_tooling=0`.
- Threshold artifact:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/check.perf_thresholds.json`
  reports `failures=[]`.
- Trace artifacts:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/1778898759498/trace.chrome.json`
  (`real_span_event_count=40`) and
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/1778898765184/trace.chrome.json`
  (`real_span_event_count=45`) both report
  `trace_source=bundle_synthetic_phases_with_extension_spans` and `real_spans_included=true`.

DevTools GUI perf-threshold preset closure (2026-05-16):

- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns the product-chain docking perf preset
  used by the GUI generated gate form: `perf-docking-arbitration-steady`, repeat `1`, warmup `5`,
  aggregate `max`, and the full CPU/layout/pointer/renderer threshold flag set mirrored from
  `tools/diag_gate_imui_product_chain.py`.
- `apps/fret-devtools/src/native.rs` renders first-class inputs for top/layout/solve,
  pointer-move dispatch/hit-test/global-change thresholds, renderer encode/upload/record/finish,
  text/SVG prepare, instance bytes, and encode-scene text ops, then delegates command generation
  and `diag_args` validation back to the shared `fret-diag` projection.
- Perf regression summaries now keep attribution follow-ups runnable: new `diag perf` rows include
  `bundle_dir`, and the shared regression-summary drill-down recovers bundle roots from older
  `bundle_artifact` / threshold failure `evidence_bundle` paths for DevTools stats/triage/hotspots
  follow-up commands.
- 2026-05-16 maintenance: the same selected-bundle projection now includes `diag trace <bundle>
  --json`, so failing perf-threshold bundles can produce trace artifact metadata from the same
  GUI/MCP follow-up surface as stats, triage, and hotspots.
- 2026-05-16 maintenance: the GUI follow-up result schema now records trace output artifacts
  explicitly, so `trace.chrome.json` becomes part of the selected-result summary/detail evidence
  rather than a path the user has to infer from the bundle directory.
- 2026-05-21 maintenance: the selected-summary inspector can copy or open the selected trace
  artifact directly. The action resolves `trace_report.trace_chrome_json_path` first, falls back to
  the `trace.chrome.json` output artifact row, and resolves relative paths against the repo root
  before clipboard or platform URL handling.
- The shared follow-up projection now emits commands for every selected bundle root, with stable
  first-bundle command ids for GUI run buttons and indexed labels/ids for additional
  threshold-failure bundles shown to GUI/MCP consumers.
- The DevTools selected-summary inspector now renders runnable follow-up command actions from that
  shared projection, so indexed threshold-failure bundle commands can be launched from the GUI
  instead of only copied from the command text block.
- Focused source gates:

```text
cargo nextest run -p fret-diag devtools_gate_perf_threshold_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_includes_runnable_diag_args devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers devtools_gate_perf_threshold_product_chain_defaults_are_runnable --no-fail-fast
cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required regression_bundle_followup_commands_cover_each_selected_bundle perf_row_to_regression_item_uses_single_run_bundle_artifact perf_row_to_regression_item_marks_threshold_failures --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-devtools runnable_followup_command_action_lines_surface_indexed_bundle_commands regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_output_artifacts regression_followup_trace_artifact_path_prefers_trace_report regression_followup_trace_artifact_path_falls_back_to_output_artifacts file_url_from_path_projects_trace_artifact_paths regression_followup_result_history_entry_detail_lines_surface_repro_fields load_recent_followup_result_history_reads_latest_valid_records load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
```

DevTools/product workflow discovery follow-up (2026-05-15): `fretboard-dev list tool-apps` now
prints a `workflow: imui-product-chain` row, and `fretboard-dev list tool-apps --json` exposes the
same route under `product_workflows`. The default discovery gate validates the default
`python tools/diag_gate_imui_product_chain.py` command, the focused
`python tools/diag_gate_imui_product_chain.py --only discovery` command, the launched
`python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release`
command, `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, and the expected
`perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, and
`perf-docking/*/trace.chrome.json` artifacts so DevTools-style consumers can surface the
product-chain evidence path without hard-coding GUI-only knowledge.

DevTools demo/metrics/debug discovery follow-up (2026-05-21): `fretboard-dev list tool-apps` now
prints a `route: demo-metrics-debug` row, and `fretboard-dev list tool-apps --json` exposes the
same route under `first_open_routes`. The route groups the editor proof/editor notes/device shell
demos separately from the `diag stats`, `diag layout-perf-summary`, `diag memory-summary`,
`diag triage`, `diag hotspots`, and `diag trace` commands. This keeps the Dear ImGui-style
Demo/Metrics/Debug entrypoint discoverable from CLI/JSON consumers rather than only from the
DevTools GUI guide panel.
Focused gates passed locally for this slice:

```text
cargo fmt -p fretboard-dev --check
cargo nextest run -p fretboard-dev tool_apps_list_names_first_open_routes tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py tools/gate_imui_workstream_source.py
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
python tools/gate_imui_workstream_source.py
git diff --check
```

DevTools demo/metrics/debug trace drill-down follow-up (2026-05-21): the same
`demo-metrics-debug` route is now projected through CLI, JSON, DevTools GUI, and MCP first-open
surfaces with `diag trace <bundle-or-dir> --json` alongside stats/layout/memory/triage/hotspots.
This keeps trace artifact handoff visible from the first-open Demo/Metrics/Debug route while
leaving perf implementation work in the diagnostics/perf lanes. Focused gates passed locally for
this follow-up:

```text
cargo fmt -p fretboard-dev -p fret-devtools -p fret-devtools-mcp --check
cargo nextest run -p fretboard-dev tool_apps_list_names_first_open_routes tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
cargo build -p fretboard-dev -p fret-devtools -p fret-devtools-mcp
python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py tools/gate_imui_workstream_source.py
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
python tools/gate_imui_workstream_source.py
git diff --check
```

Goal completion audit refresh (2026-05-15):
`GOAL_COMPLETION_AUDIT_2026-05-15.md` keeps the umbrella in maintenance and explicitly not
complete. The strict blockers remain real-host Wayland compositor acceptance for `DW-P1-linux-003`,
DevTools GUI productization / always-available demo-metrics-debug discoverability, and broader perf
attribution/smoothness outside the bounded `perf-docking` entrypoint.

The 2026-05-13 launched bounded campaign result is `campaign: ok` at
`target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778655473217`, with a post-documentation
verification rerun also green at
`target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778656624160`. This closes the generic
bounded-campaign gap, but not Linux Wayland compositor acceptance or every platform-specific
real-host hand-feel risk.

The 2026-05-16 `M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md` note broadens the M17 local
policy-skip gate into a Windows plus Linux/X11 sidecar matrix. Both probes stop at
`skipped_policy` before script execution, so the evidence strengthens local admission posture
without claiming `DW-P1-linux-003` real-host Wayland acceptance.

The 2026-05-17 `M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md` note freezes that interpretation
in the docking source gate: `DW-P1-linux-003` must remain in progress, the manual Wayland
acceptance checkbox must remain open, and the M5 runbook stays the next true closure path until a
real Wayland compositor evidence note exists.

### Lane hygiene gates

- `python tools/gate_imui_workstream_source.py`
- `python tools/diag_gate_docking_wayland_policy_skip.py`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json > /dev/null`
- `rg -n "imui_hello_demo|fret-examples-imui|--package fret-demo|--package fret-examples-imui" docs/examples/README.md apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md`

## Remaining gates that should become real before claiming closure

### P0 launched authoring proof

Status: landed as a focused gate.

The source-policy/doc gates prove that:

- first-party docs/examples teach the frozen golden pair,
- reference proofs stay explicitly classified as non-default,
- helper widening requires the frozen two-surface proof budget,
- and the launched `imui_action_basics` smoke exercises command palette, declarative, GenUI, and
  IMUI triggers through one typed action handler.

Focused command:

```text
python tools/diag_gate_action_first_authoring_v1.py --only cookbook-imui-action-basics-cross-frontend
```

Focused editor-control visual gate:

```text
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --json
cargo run -p fretboard-dev -- diag suite cookbook-imui-editor-controls-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics
```

Latest local action evidence (2026-04-28): `PASS (run_id=1777376310911)`, packed at
`target/dfa-v1/1777376303772/i/share/1777376310911.zip`.

Latest local editor-control smoke evidence (2026-05-13): `PASS (run_id=1778653020152)`, direct run
artifact root:
`target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6`.
The documented suite command also passed on 2026-05-13 with both scripts:

- smoke: `PASS ... (run_id=1778653340628)`
- roughness typing: `PASS ... (run_id=1778653344599)`

The suite summary at `target/fret-diag/suite.summary.json` reported `scripts_with_evidence: 2` and
`warning_issues: 0` for both bundles.

The captured first-contact editor-control artifacts are:

- layout sidecar:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020648-cookbook-imui-editor-controls-basics-smoke.layout/layout.taffy.v1.json`
- screenshot:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/screenshots/1778653020668-cookbook-imui-editor-controls-basics-smoke/window-4294967297-tick-34-frame-33.png`
- final bundle:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020746-cookbook-imui-editor-controls-basics-smoke/bundle.schema2.json`
- roughness typing bundle:
  `target/fret-diag/1778653344759-cookbook-imui-editor-controls-roughness-typing/bundle.schema2.json`

Latest launched generic-action evidence (2026-05-14): `PASS (run_id=1778703206445)`, packed at
`target/imui-product-chain-launched-2026-05-14-generic-action-action-route-fallback/1778702675441/generic-action/1778702675548/i/share/1778703206445.zip`.
This run exercises command palette, declarative, GenUI DropdownMenu, and IMUI triggers through the
same typed action handler after `fret-ui` began honoring explicit action-route fallback roots for
view/app-owned action handlers. The source gate is:

```text
cargo nextest run -p fret-ui action_availability_snapshot_does_not_scan_unfocused_subtree action_availability_snapshot_matches_no_focus_dispatch_subtree_fallback --no-fail-fast --jobs 1
cargo nextest run -p fret --lib app_ui_unit_action_handler_publishes_available_command_snapshot_by_default app_ui_unit_action_handler_publishes_available_snapshot_when_focus_exists locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view --no-fail-fast --jobs 1
python tools/diag_gate_imui_product_chain.py --launched --only generic-action --release --out-dir target/imui-product-chain-launched-2026-05-14-generic-action-action-route-fallback
```

### Product-chain discovery gate

Status: landed as a lightweight maintainer gate.

The default product-chain gate validates discovery plus promoted script/suite/campaign inputs across
`imui_action_basics`, `imui_editor_controls_basics`, `imui_editor_proof_demo`,
`editor_notes_demo`, `editor_notes_device_shell_demo`, `workspace_shell_demo`,
`docking_arbitration_demo` through the `imui-p3-multiwindow-parity` campaign manifest,
`perf-docking-arbitration-steady` as the docking perf entrypoint,
DevTools/diagnostics first-open, and the IMUI source gates. It does not
replace the individual launched gates; it keeps the cross-app product chain discoverable and
validated without forcing a single `diag campaign` launch target onto unrelated apps.

Focused command:

```text
python tools/diag_gate_imui_product_chain.py
```

Latest local default product-chain evidence (2026-05-14):

- Command: `python tools/diag_gate_imui_product_chain.py`
- Result: passed.
- Added coverage: the default gate now runs
  `diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`, so the
  discovered docking proof surface has a manifest-shape check in the same maintainer command as the
  cookbook, editor proof, editor notes, workspace shell, DevTools discovery, and IMUI source gates.
- Added coverage: the discovery step now also validates
  `fretboard-dev list tool-apps --json` as the stable first-open DevTools GUI/MCP machine-readable
  map, including repo preflight and per-tool command/docs/gate/best-for fields.
- Added coverage: the same JSON now exposes a `product_workflows` entry for
  `imui-product-chain`, including the default product-chain command, the focused discovery-only
  command, the launched `perf-docking` command, the promoted
  `perf-docking-arbitration-steady` suite, and the expected
  `perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, and
  `perf-docking/*/trace.chrome.json` evidence artifacts.
- Added coverage: the same discovery step now validates `fretboard-dev --help` and
  `fretboard-dev list --help`, so the tool-app index itself stays discoverable from the first CLI
  help screens.
- Added coverage: the default lightweight gate now validates the
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json` scripts, while the
  explicit launched `perf-docking` product-chain slice verifies the perf regression summary shape,
  item bundle artifacts, shared layout perf summary artifact, shared `check.perf_thresholds.json`
  artifact, conservative CLI thresholds, empty threshold failures, and lightweight summary metrics.

Use `--launched` when the local machine should also execute the existing launched proof commands
sequentially across the cookbook, editor proof, editor notes, and workspace shell surfaces:

```text
python tools/diag_gate_imui_product_chain.py --launched --only generic-action,editor-controls,editor-proof,editor-notes,editor-notes-device-shell,workspace-shell,perf-docking
```

For the editor-notes product slice alone:

```text
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes,editor-notes-device-shell
```

Use `--reuse-built` for heavy `fret-demo` binaries when the relevant `target/debug` or
`target/release` executable already exists; this keeps the launched proof focused on diagnostics
behavior instead of `cargo run` build-lock timing.

Latest local editor-notes product-chain evidence (2026-05-14):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes --out-dir target/imui-product-chain-editor-notes-selection-sync-2026-05-14-r3 --timeout-ms 240000 --poll-ms 50`
- Run root:
  `target/imui-product-chain-editor-notes-selection-sync-2026-05-14-r3/1778735909022`
- `editor-notes/suite.summary.json` reports `status=passed`, `stage_counts.passed=3`,
  `scripts_with_evidence=3`, and `warning_issues=0` for all three script lint outputs.
- The third script,
  `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json`, proves
  left-rail asset selection updates collection summary, inspector field values, and app-owned
  summary-command status across Material -> Key Light -> Camera -> Material.
- Root-cause fix:
  `ecosystem/fret-selector/src/ui.rs` now includes `ModelId` before revision in model-backed
  selector dependency signatures, so switching between same-revision models recomputes derived UI
  values instead of replaying stale cache entries.

Previous combined editor-notes/editor-notes-device-shell proof (2026-05-14):

- Run root:
  `target/imui-product-chain-editor-notes-launched-2026-05-14-reuse/1778729721045`
- `editor-notes/suite.summary.json` reported `status=passed`, `stage_counts.passed=2`, and
  `scripts_with_evidence=2`; `editor-notes-device-shell/suite.summary.json` reported
  `status=passed`, `stage_counts.passed=1`, and `scripts_with_evidence=1`.

Follow-up accessibility repair evidence (2026-05-14):

- Cause:
  `editor_notes_device_shell_demo` exposed the shared modal backdrop/barrier as a full-window
  unlabeled `button` semantics node. The fix stays in the headless policy layer:
  `ecosystem/fret-ui-kit/src/primitives/dialog.rs` hides shared modal barriers from the
  accessibility tree while leaving them pointer-invokable, and
  `ecosystem/fret-ui-kit/src/primitives/select.rs` applies the same policy to Select's
  pointer-up-guard barrier.
- Source gates:
  `cargo nextest run -p fret-ui-kit modal_barrier_is_hidden_from_accessibility_tree_but_still_invokable select_pointer_up_guard_barrier_is_hidden_from_accessibility_tree --no-fail-fast`
- Launched proof:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes-device-shell --out-dir target/imui-product-chain-editor-notes-device-shell-a11y-2026-05-14 --timeout-ms 240000 --poll-ms 50`
- Run root:
  `target/imui-product-chain-editor-notes-device-shell-a11y-2026-05-14/1778731960670`
- `editor-notes-device-shell/suite.summary.json` reports `status=passed`,
  `stage_counts.passed=1`, `scripts_with_evidence=1`, and `warning_issues=0`.
- `check.lint.json` for
  `1778731966234-editor-notes-device-shell-demo.mobile-drawer-open` reports
  `counts_by_code=[]`, `findings=[]`, `error_issues=0`, and `warning_issues=0`.

### P3 multi-window parity gate

The checklist and bounded package are now both explicit:

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` freezes the runner-owned parity budget,
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` freezes the lane-owned bounded package,
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json` is the canonical P3 campaign manifest.

Future work should replace or refine items inside that bounded package rather than inventing
another parallel P3 gate entry.

### Selector mechanism gate

- `cargo nextest run -p fret-selector --features ui deps_builder_model_rev_includes_model_identity_before_revision --no-fail-fast`
- This locks the real `ElementContext` + `ModelStore` path so same-revision model switches still
  invalidate selector memoization correctly.

## Maintenance gate refresh - 2026-05-15 follow-up

Scope: close the `fret-ui` layout/view-cache regressions left by the previous affected gate without
changing the IMUI layer split. The fixes stay in `crates/fret-ui` mechanism code:

- `crates/fret-ui/src/tree/commands.rs` refreshes window command action availability after
  post-layout runtime snapshot refinement by clearing the cached availability signature before
  publishing snapshots.
- `crates/fret-ui/src/tree/dispatch/window.rs` treats the post-wheel scroll-handle invalidation pass
  as the final baseline consumer, so non-retained virtual lists schedule their one-shot view-cache
  rerender immediately after a wheel-driven visible-window escape.
- `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` keeps scroll deep-scan validation
  from trusting a synthetic content-bounds barrier root as the authoritative extent when descendants
  provide the real frontier.
- `crates/fret-ui/src/layout/engine/flow.rs` carries definite parent flex-axis information into
  wrapper fill promotion, so viewport-root auto wrappers can promote to fill under a definite
  cross-axis without globally stretching shrink-wrapped wrappers.

Focused repro gates:

```text
cargo nextest run -p fret-ui layout_refines_focus_traversal_availability_after_structural_fallback scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached --no-fail-fast
```

Result: passed, `5 tests run: 5 passed`.

Affected/full maintenance gates:

```text
cargo fmt -p fret-ui
cargo nextest run -p fret-ui -p fret-launch -p fret-bootstrap --no-fail-fast
cargo clippy -p fret-devtools --all-targets -- -D warnings
python tools/check_layering.py
python tools/report_largest_files.py --top 30 --min-lines 800
git diff --check
```

Result: passed. The affected nextest gate reported `1059 tests run: 1059 passed`. The largest-file
report remains a drift watchlist only for this slice; no new large-file expansion was introduced
outside the touched `fret-ui` mechanism files.

## DevTools gate profile owner split - 2026-05-15 follow-up

Scope: continue DevTools GUI productization without widening `fret-imui` or turning
`apps/fret-devtools` into a diagnostics-policy owner.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns the shared DevTools gate taxonomy for
  stale paint/scene, pixels-changed, perf thresholds, resource-footprint thresholds, and
  resource-footprint compare profiles.
- `apps/fret-devtools/src/native.rs` now renders the first-open `Gate Commands` panel from
  `fret_diag::devtools_gate_profile_lines(...)`, keeping the GUI as a thin consumer of the shared
  diagnostics projection.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks both the GUI consumer and the
  shared profile owner, so the first-open gate catches drift without requiring GUI-owned command
  constants.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
```

Result: passed. The `fret-diag` nextest gate reported `2 tests run: 2 passed`; the `fret-devtools`
nextest gate reported `1 test run: 1 passed`; the DevTools first-open discovery gate completed
successfully after rebuilding `fretboard-dev` and validating tool-app discovery, GUI source, shared
gate profile source, and first-open docs. `python tools/diag_gate_imui_product_chain.py --only
discovery` also passed after validating the broader product-chain source gates, and
`python tools/report_largest_files.py --top 30 --min-lines 800` remains a drift watchlist only.

## DevTools gate profile copy actions - 2026-05-15 follow-up

Scope: make the first-open `Gate Commands` projection an explicit per-profile action surface before
adding profile-specific parameter forms or launch/run behavior.

- `apps/fret-devtools/src/native.rs` now renders a `Copy command` button for every shared
  `fret-diag` DevTools gate profile.
- The GUI still consumes `devtools_gate_profiles_v1()` / `devtools_gate_profile_lines(...)`; gate
  taxonomy remains in `crates/fret-diag/src/devtools_gate_profiles.rs`.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the copy action surface and the shared
  profile owner separately.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `1 test run: 1 passed`; the `fret-diag`
nextest gate reported `2 tests run: 2 passed`; both source/discovery gates completed successfully.

## DevTools script-target gate command builder - 2026-05-15 follow-up

Scope: move the first gate profile parameter form from raw command templates toward a selected,
copyable, concrete command while keeping command construction in `fret-diag`.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now exposes script-target profile ids and
  `devtools_gate_script_target_command_line(...)` for stale paint/scene and pixels-changed
  profiles, with structured `diag_args` and `missing_inputs` for the next run/launch slice.
- `apps/fret-devtools/src/native.rs` now renders a script-target gate profile selector,
  `script.json` and `test-id` inputs, command preview, and `Copy generated command` action inside
  the first-open `Gate Commands` panel.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the shared command builder API and GUI
  action surface.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The `fret-diag` nextest gate reported `7 tests run: 7 passed`; the
`fret-devtools` nextest gate reported `1 test run: 1 passed`; both source/discovery gates completed
successfully.

## DevTools script-target gate runner - 2026-05-15 follow-up

Scope: turn the generated script-target gate command into a GUI-runnable action while keeping gate
policy and command construction in `fret-diag`.

- `apps/fret-devtools/src/gate_run.rs` now owns the GUI background job wrapper for script-target
  gate runs. It executes the structured `diag_args` from
  `DevtoolsGateScriptTargetCommandV1`, not the copied shell command string.
- `apps/fret-devtools/src/native.rs` wires `Run generated command`, in-flight/error/result-path
  state, and an inline result JSON preview into the `Gate Commands` builder.
- Gate run results are written to `.fret/diag/gate-runs/*.json` with the stable
  `fret_devtools_gate_run_result` kind, command line, diag args, status, error, and timing fields.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check both `native.rs` and `gate_run.rs` so the
  product-chain discovery gates cover the runner module and the GUI surface together.

Focused gates:

```text
cargo nextest run -p fret-devtools gate_run_result_record_has_stable_shape devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `2 tests run: 2 passed`; the `fret-diag`
nextest gate reported `7 tests run: 7 passed`; both DevTools discovery/source gates completed
successfully; `cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings`,
`python tools/check_layering.py`, and `git diff --check` passed. `git diff --check` reported only
the existing CRLF normalization warning for `tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools generated gate result history - 2026-05-15 follow-up

Scope: finish the stale paint/scene and pixels-changed generated-gate loop by making result
artifacts selectable and reusable from the GUI.

- `apps/fret-devtools/src/gate_run.rs` now projects gate result artifacts into bounded in-memory
  history entries plus summary/detail helper lines. DevTools startup now restores recent valid
  `.fret/diag/gate-runs/*.json` records into that same history, skipping malformed and non-gate
  JSON so generated-gate evidence survives a GUI restart.
- `apps/fret-devtools/src/native.rs` now renders generated gate result details, summary, history,
  raw JSON, selected-result copy actions, and a platform URL open action.
- The result history remains GUI state over `.fret/diag/gate-runs/*.json`; diagnostics gate policy
  and command construction still live in `crates/fret-diag`.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the history, copy/open actions, and summary
  projection in addition to the background runner.

Focused gates:

```text
cargo nextest run -p fret-devtools gate_run_result_record_has_stable_shape gate_run_result_summary_lines_project_status_and_duration load_recent_gate_run_result_history_reads_latest_valid_records load_recent_gate_run_result_history_prefers_record_time_over_file_mtime gate_run_result_history_selects_explicit_path_or_latest devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `5 tests run: 5 passed`; the
`fret-diag` nextest gate reported `7 tests run: 7 passed`; both source/discovery gates completed
successfully; `cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings`,
`python tools/check_layering.py`, and `git diff --check` passed. `git diff --check` reported only
the existing CRLF normalization warning for `tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools perf threshold generated gate builder - 2026-05-15 follow-up

Scope: extend the generated-gate GUI loop from script-target stale/pixels gates to the first
thresholded perf gate without making the GUI parse shell strings or own diagnostics policy.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now exposes a shared
  `DevtoolsGateCommandV1` plus `DevtoolsGatePerfThresholdCommandInputV1` and
  `devtools_gate_perf_threshold_command(...)` for `diag perf` threshold runs.
- `apps/fret-devtools/src/native.rs` now includes `perf-thresholds` in the generated gate builder,
  renders target/repeat/warmup/aggregate/threshold inputs, and reuses the existing generated gate
  runner plus `.fret/diag/gate-runs/*.json` result history.
- The legacy script-target API name remains as a type alias over the generic command shape, so the
  existing stale paint/scene and pixels-changed UI path stays source-compatible while the shared
  command model stops pretending every generated gate is script-target-only.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the perf-threshold command projection, GUI
  test ids, and helper split.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_includes_runnable_diag_args devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates gate_run_result_record_has_stable_shape gate_run_result_summary_lines_project_status_and_duration gate_run_result_history_selects_explicit_path_or_latest --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-diag` nextest gate reported `8 tests run: 8 passed`; the
`fret-devtools` nextest gate reported `4 tests run: 4 passed`; both source/discovery gates
completed successfully when run sequentially; `cargo clippy -p fret-diag -p fret-devtools
--all-targets -- -D warnings`, `python tools/check_layering.py`, and `git diff --check` passed.
`git diff --check` reported only the existing CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools resource footprint generated gate builder - 2026-05-15 follow-up

Scope: close the remaining first-class DevTools gate UI item by making resource-footprint threshold
commands real, structured, and GUI-runnable without shell parsing.

- `crates/fret-diag/src/cli/contracts/commands/repro.rs` now exposes the documented
  `--max-working-set-bytes`, `--max-peak-working-set-bytes`, and
  `--max-cpu-avg-percent-total-cores` options.
- `crates/fret-diag/src/cli/cutover.rs` now passes those options into
  `ResourceFootprintThresholds`, so `diag repro` writes/enforces `check.resource_footprint.json`
  instead of advertising inert flags.
- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns
  `DevtoolsGateResourceFootprintThresholdCommandInputV1` and
  `devtools_gate_resource_footprint_threshold_command(...)`.
- `apps/fret-devtools/src/native.rs` now includes `resource-footprint-thresholds` in the generated
  gate builder and reuses the same generated gate runner/result history. The launch input is a
  single argv item, not a shell command string.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the CLI contract, cutover mapping, shared
  command projection, GUI test ids, and helper split.

Focused gates:

```text
cargo nextest run -p fret-diag repro_contract_captures_resource_footprint_thresholds contract_help_mentions_the_migrated_command_surfaces high_risk_main_lane_help_has_drift_guards devtools_gate_resource_footprint_threshold_command_preserves_placeholders_until_filled devtools_gate_resource_footprint_threshold_command_includes_runnable_diag_args devtools_gate_resource_footprint_threshold_command_quotes_paths_and_rejects_invalid_numbers --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates gate_run_result_record_has_stable_shape --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-diag` nextest gate reported `6 tests run: 6 passed`; the
`fret-devtools` nextest gate reported `2 tests run: 2 passed`; both source/discovery gates
completed successfully when run sequentially; `cargo clippy -p fret-diag -p fret-devtools
--all-targets -- -D warnings`, `python tools/check_layering.py`, and `git diff --check` passed.
`git diff --check` reported only the existing CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools live inspect overlay payload closure - 2026-05-15 follow-up

Scope: close the M6 live-inspect gap without widening `fret-imui` or moving interaction policy into
`fret-ui`. The fix makes the existing `inspect.hover` / `inspect.focus` receiver contract real and
adds the missing overlay hook/summary projection:

- `crates/fret-diag-protocol/src/lib.rs` now owns `UiInspectHoverV1`, `UiInspectFocusV1`,
  `UiInspectNodeSummaryV1`, `UiInspectOverlayHookV1`, `UiOverlayRootHintV1`, and
  `UiOverlaySummaryV1`.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs` publishes changed
  `inspect.hover`, `inspect.focus`, and `overlay.summary` payloads over the diagnostics WS bridge,
  including hovered/focused node bounds, viewport bounds, barrier roots, blocking roots, and
  topmost interactive root hints.
- `apps/fret-devtools/src/native.rs` now renders structured `Live Inspect Hover Bounds`,
  `Live Inspect Overlay Hooks`, and raw inspect payload panels instead of only showing hover/focus
  JSON blobs.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the protocol/runtime/GUI split so the
  first-open gates catch future raw-JSON-only regressions.

Focused gates:

```text
cargo nextest run -p fret-diag-protocol live_inspect_payloads_roundtrip_bounds_and_overlay_summary --no-fail-fast
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" inspect_node_summary_v1_includes_bounds_and_root_hint overlay_summary_v1_reports_barrier_and_blocking_roots --no-fail-fast
cargo nextest run -p fret-devtools inspect_hover_bounds_lines_project_bounds_and_selector inspect_hover_bounds_lines_missing_bounds_returns_none inspect_overlay_hook_lines_project_overlay_summary --no-fail-fast
cargo clippy -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib -- -D warnings
cargo clippy -p fret-devtools -p fret-diag-protocol --all-targets -- -D warnings
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/check_layering.py
git diff --check
```

Result: passed. The protocol nextest gate reported `1 test run: 1 passed`; the bootstrap focused
gate reported `2 tests run: 2 passed`; the DevTools focused gate reported `3 tests run: 3 passed`;
both discovery/source gates completed successfully; layering and diff whitespace checks passed.
Note: the full bootstrap test-target clippy command
`cargo clippy -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --all-targets -- -D warnings`
currently also hits pre-existing `items_after_test_module` warnings in diagnostics script-step test
modules; this slice uses the lib clippy gate for the changed runtime path and leaves that broader
test-target lint debt as a separate cleanup input.

## DevTools UI gallery dogfood workflow closure - 2026-05-15 follow-up

Scope: close the M6 DevTools dogfood workflow gap with one concrete authoring loop that stays on
shared diagnostics contracts instead of adding a GUI-only campaign model.

- `apps/fret-devtools/src/native.rs` now renders a `Dogfood Workflow` block in the first-open
  shell. The block names the `ui-gallery-button-dogfood` path: open `fret-ui-gallery`, pick a
  Button-page selector, generate or apply the selector into a script, run with `diag run --pack`,
  pack a selected bundle, and open `tools/fret-bundle-viewer`.
- The visible path references existing script evidence:
  `tools/diag-scripts/ui-gallery-lite-smoke.json` and
  `tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json`.
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md` now records the
  same concrete loop, and
  `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M6 dogfood item
  complete.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the GUI surface and canonical command
  markers so future edits do not silently hide the dogfood route.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_dogfood_workflow_lines_surface_ui_gallery_loop --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The `fret-devtools` focused nextest gate reported `1 test run: 1 passed`; both
DevTools discovery/source gates completed successfully.

## DevTools semantics tree scalability closure - 2026-05-15 follow-up

Scope: close the M6 tree scalability item without widening `fret-imui` or adding a GUI-only live
tree transport. The slice locks two DevTools invariants with code tests and source gates:

- `apps/fret-devtools/src/native.rs` continues to render the Semantics tab through
  `VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)` with `items_revision = rows_key`.
- `apps/fret-devtools/src/semantics.rs` now computes visible rows with an explicit stack instead of
  recursive DFS, preventing stack overflow on deeply nested 50k-node semantics trees.
- `apps/fret-devtools/src/ws.rs` extracts `live_semantics_request_decision`, proving unchanged
  selected-node live detail polling stays at 1Hz while selection changes and manual refreshes still
  request immediately.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the VirtualList, iterative row projection,
  1Hz throttle, and focused test names.

Focused gates:

```text
cargo nextest run -p fret-devtools compute_rows_handles_50k_flat_semantics_nodes compute_rows_handles_50k_deep_semantics_tree_without_recursion compute_rows_search_forces_visible_ancestor_path_on_large_tree live_semantics_request_decision_throttles_unchanged_selection_to_one_hz live_semantics_request_decision_allows_selection_change_and_manual_refresh --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The focused `fret-devtools` nextest run reported `5 tests run: 5 passed`; both
DevTools discovery/source gates passed after the source guards were corrected to read
`apps/fret-devtools/src/semantics.rs` explicitly. The follow-up quality gates also passed:
`cargo clippy -p fret-devtools --all-targets -- -D warnings`, `python tools/check_layering.py`, and
`git diff --check` (with only the known CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`).

## DevTools MCP AI scenario doc closure - 2026-05-15 follow-up

Scope: close the M7 MCP end-to-end AI scenario doc item while keeping MCP as a diagnostics
consumer over shared CLI/GUI artifacts, not a new IMUI runtime surface.

- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md` now records the
  end-to-end AI path: enable inspect, pick a stable selector, choose/fork a script, run one or more
  scripts, aggregate regression summaries when needed, pack the latest bundle, and open the offline
  viewer.
- The same doc names the artifact resources and freshness contract:
  `fret-diag://first-open.md`, selected-session bundle/regression resources, resource
  subscriptions, and resource update notifications.
- `apps/fret-devtools-mcp/src/native.rs` already owns the matching tool/resource implementation
  anchors: inspect, pick, scripts list, run script/file/batch, regression summarize/dashboard, pack
  latest bundle, pack zip bytes, latest bundle dump, compare, first-open resource, and resource
  update notifications.
- `tools/diag_gate_imui_p2_devtools_first_open.py` now source-checks this doc plus the MCP
  implementation anchors through the `devtools mcp ai scenario doc` step, so the AI scenario cannot
  silently drift away from the actual tool surface.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M7 AI scenario
  doc parent item complete.

Focused gate:

```text
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. The gate reported the new `devtools mcp ai scenario doc` step and completed the
first-open discovery check successfully.

## DevTools cross-cutting hygiene closure - 2026-05-15 follow-up

Scope: close the DevTools hygiene checklist items that protect architecture boundaries rather than
add new GUI scope.

- `tools/diag_gate_imui_p2_devtools_first_open.py` now runs a `devtools cross-cutting hygiene`
  discovery check.
- The check validates `bundle.json` forward compatibility from both sides of the contract:
  `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md` requires unknown fields to be
  ignored, while `tools/fret-bundle-viewer/README.md`,
  `tools/fret-bundle-viewer/lib/parser.ts`, and `tools/fret-bundle-viewer/lib/zip.ts` keep the
  offline viewer on best-effort parsing and `bundle.json` / `bundle.schema2.json` / zip inputs.
- The check validates the policy boundary: `crates/fret-ui/README.md` remains the mechanism-layer
  contract, and the gate fails if DevTools-specific policy markers are added under
  `crates/fret-ui/src`.
- The check validates stable selector guidance: the DevTools workstream doc, GUI default selector
  state, `test_id` selector option, UI-gallery preferred selector, and `devtools.gate.test_id`
  input all stay aligned.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` now marks the three
  cross-cutting hygiene items complete with this gate as evidence.

Focused gate:

```text
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. This closes the hygiene checklist only; broader DevTools GUI product maturity,
real-host Wayland acceptance, and full perf/smoothness attribution remain outside this slice.

## DevTools secondary tree views closure - 2026-05-15 follow-up

Scope: close the DevTools M0 secondary tree entrypoints without widening the runtime protocol or
claiming full native layout/element snapshots.

- `apps/fret-devtools/src/native.rs` now adds `Layout` and `Elements` tabs beside the default
  `Semantics` tree in the left Inspect Workspace.
- The new tabs are lazily materialized from the active tab, so adding secondary tree views does not
  build three 50k-row virtual-list projections in the same frame.
- `apps/fret-devtools/src/semantics.rs` keeps one shared tree index and adds projection labels:
  layout rows surface parent + bounds + role + `test_id`; element rows surface semantics-node
  identity plus authoring relationships (`labelled_by`, `described_by`, `controls`).
- Search now covers node id, `parent=<id>`, and bounds text, so the secondary views are useful for
  layout and identity debugging without adding a new bundle schema.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks the secondary tabs, lazy active-tab
  construction, projection labels, and focused tests.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M0 layout/element
  tree items complete with the explicit caveat that these are semantics-derived secondary views,
  not full layout-engine or declarative runtime snapshots.

Focused gates:

```text
cargo nextest run -p fret-devtools compute_rows_search_matches_id_parent_and_bounds secondary_tree_labels_surface_layout_and_identity_fields --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. This removes the stale M0 secondary-view TODOs while keeping broader DevTools GUI
product maturity, real-host Wayland acceptance, and full perf/smoothness attribution open.

## IMUI source gate owner-anchor refresh - 2026-05-15 follow-up

Scope: repair the active IMUI source gate after the DevTools first-class gate commands moved to the
shared diagnostics owner.

- `tools/gate_imui_workstream_source.py` now checks `crates/fret-diag/src/devtools_gate_profiles.rs`
  for the first-class stale/pixels/perf/resource-footprint gate taxonomy, structured command
  builders, evidence names, and focused tests.
- `apps/fret-devtools/src/native.rs` remains a GUI consumer of
  `devtools_gate_profiles_v1()` / `devtools_gate_profile_lines(...)` rather than re-owning command
  template constants.
- `crates/fret-diag/src/regression_summary.rs` remains a follow-up command projection consumer of
  `crate::util::shell_quote_arg`, and the source gate now checks the quoting helper in
  `crates/fret-diag/src/util.rs`.
- `GOAL_COMPLETION_AUDIT_2026-05-15.md` now includes the explicit sentence
  "GUI productization is still not complete" so the overall editor-grade goal remains open until
  broader always-available tooling evidence exists.

Focused gate:

```text
python tools/gate_imui_workstream_source.py
```

Result: passed. This is a gate-anchor repair only; it does not claim new DevTools GUI maturity.

## DevTools first-open guide posture - 2026-05-16 follow-up

Scope: reduce first-open cognitive load in `apps/fret-devtools` without changing diagnostics
contracts or moving policy into `fret-imui`.

- `apps/fret-devtools/src/native.rs` now defaults `Evidence & Results` to a `Guide` tab instead of
  an empty raw `Pick` payload tab.
- The header now renders a stateful `First-open Next Actions` summary for target/session status,
  script inventory, regression aggregate state, and artifacts root.
- The full first-open evidence path, UI-gallery dogfood workflow, demo/metrics/debug route, and
  gate-command reference panels still exist in `apps/fret-devtools/src/native.rs`, but they render
  inside the `Guide` tab so the first viewport stays summary-first.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks this posture alongside the older
  first-open discovery, gate-command, live-inspect, and secondary-tree source anchors.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_first_open_next_action_lines_prioritize_stateful_workflow devtools_first_open_lines_surface_canonical_paths devtools_dogfood_workflow_lines_surface_ui_gallery_loop devtools_demo_metrics_debug_lines_surface_canonical_routes devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed locally. This is a DevTools GUI productization slice only; it keeps the editor-grade
goal open for broader always-available tooling maturity, real-host Wayland hand-feel, and full
perf/smoothness attribution.

## Perf text clean-geometry owner split - 2026-05-21 follow-up

Scope: keep the editor-grade perf/smoothness evidence chain pointed at the correct runtime owner
without moving layout policy into `fret-imui`.

- `docs/workstreams/text-clean-geometry-stability-v1/` is now the closed text clean-geometry
  boundary record split out of `scroll-optimization-v1`.
- `docs/workstreams/scroll-optimization-v1/HANDOFF.md` and `WORKSTREAM.json` now point the text
  follow-on at that lane instead of inviting more clean-geometry widening inside the scroll
  umbrella.
- The current runtime boundary remains conservative: `TextWrap::None` text with stable cached
  metrics can skip authoritative layout, while wrapped or height-changing text rejects as
  `text_reflow`.
- Clean-geometry rejection diagnostics now carry additive `detail`; the first text detail proof
  distinguishes wrapped text as `text_wrap_not_none` instead of collapsing every text sub-cause into
  the same generic `text_reflow` bucket.
- The UI Gallery text-measure-overlay resize script passed with real bundle evidence at
  `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/bundle.schema2.json`;
  that bundle contains `clean_geometry_solve_skip_rejection.detail` values
  `text_wrap_not_none` and `text_overflow_not_clip`.
- shadcn `CardDescription` remains full-width `TextWrap::Word`; recipe authoring is not used as the
  perf lever.

Focused gates:

```text
cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics clean_geometry_rejection_detail_is_additive --no-fail-fast
cargo fmt -p fret-ui -p fret-bootstrap --check
python -m json.tool docs/workstreams/text-clean-geometry-stability-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/scroll-optimization-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. This is an owner-split, boundary-lock, and diagnostics closeout slice only;
full editor perf/smoothness attribution remains open in the dedicated perf lanes.

## Editor Canvas paint replay closeout - 2026-05-23 follow-up

Scope: record the bounded editor-paint owner-lane closeout without moving performance pressure into
`fret-imui`.

- `docs/workstreams/editor-canvas-paint-replay-slice-v1/` is now closed.
- Implementation landed in `ecosystem/fret-code-editor/src/editor/paint/scene.rs` and focused
  regression coverage in `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
- The r59 Windows RTX4090 target-machine pass produced:
  - baseline validation:
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`;
  - attribution validation:
    `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`;
  - artifact verification:
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/artifact-verification.summary.json`;
  - closeout:
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`.
- The closeout kept checked-in baselines unchanged and retained `canvas-paint-replay` as the
  verified owner.
- This is editor paint / perf-lane progress only; it does not justify `fret-imui` helper growth or
  close broad editor smoothness attribution.

Focused evidence commands from the closed lane:

```text
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay prepaint_row_scene_replay_plan_handles_plain_cached_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context prepaint_row_scene_replay_plan_rejects_plain_rows_when_fg_changes --features syntax-rust --no-fail-fast
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r59
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r59-attrib --with-paint-perf
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260523-r59 --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260523-r59 --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib
```

Result: passed and recorded in
`docs/workstreams/editor-canvas-paint-replay-slice-v1/CLOSEOUT_AUDIT_2026-05-23.md`. Broader
DevTools GUI maturity, real-host Wayland hand-feel, and full perf/smoothness attribution remain
outside this slice.

## DevTools MCP recent-evidence bridge - 2026-05-21 follow-up

Scope: keep AI/MCP first-open diagnostics aligned with the GUI `Recent Evidence` product surface
without adding a MCP-private rerun model.

- `apps/fret-devtools-mcp/src/native.rs` now exposes `fret_diag_recent_evidence`, a read-only MCP
  tool that scans the same `.fret/diag/gate-runs`, `.fret/diag/workflow-runs`, and
  `.fret/diag/followups` result records restored by the GUI.
- The report returns latest gate/workflow/follow-up evidence, the first failed result, failing
  counts, bundle dir when present, a compact human summary, and a next-action hint.
- `fret-diag://first-open.md` now points AI clients at that bridge, while workflow rerun decisions
  remain GUI-owned because safe workflow reruns require the current selected session and current
  token rather than stored historical `diag_args`.
- `fret-diag://recent-evidence.json` now exposes the same report as a sessionless MCP resource, so
  clients that start from the resource list can discover restored GUI-launched evidence without a
  tool-first detour.
- GUI and MCP first-open evidence now select the newest failed result across gate/workflow/follow-up
  lanes from result JSON `finished_unix_ms` / `started_unix_ms` first, with timestamped result
  paths as a compatibility fallback and old untimestamped records retaining the legacy fallback
  order. This keeps reopened DevTools and AI clients pointed at the most recent failure instead of
  a category-priority failure.
- Restored Gate, Workflow, and Follow-up histories, plus the MCP recent-evidence lane scan, now
  parse valid result JSON before sorting and use `finished_unix_ms` / `started_unix_ms` before file
  mtime and path fallback. Copied or synced `.fret/diag` artifacts therefore keep their recorded
  recency when DevTools or MCP reopen them.
- The same status normalization is now explicit in both GUI and MCP paths: empty status, `-`, and
  `passed` regardless of case are not failures. This prevents placeholder or externally cased
  restored records from lighting up the first-open failed-evidence controls.
- The sessionless resource list now comes from `sessionless_resource_specs()`, with
  `sessionless_resource_specs_include_first_open_and_recent_evidence` locking both
  `fret-diag://first-open.md` and `fret-diag://recent-evidence.json` into the list/template source of
  truth.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md` and
  `diag-devtools-gui-v1.md` document the same owner split.

Focused gates:

```text
cargo fmt -p fret-devtools -p fret-devtools-mcp --check
cargo nextest run -p fret-devtools load_recent_gate_run_result_history_prefers_record_time_over_file_mtime load_recent_workflow_run_result_history_prefers_record_time_over_file_mtime load_recent_followup_result_history_prefers_record_time_over_file_mtime recent_evidence_status_failed_ignores_empty_placeholder_and_passed_case devtools_recent_failed_evidence_target_prefers_visible_latest_then_history devtools_recent_failed_evidence_target_falls_back_to_lane_order_without_timestamps devtools_recent_failed_evidence_target_prefers_result_json_time_over_path_time --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain build_recent_evidence_report_reads_gui_result_records load_recent_evidence_entries_prefers_record_time_over_file_mtime recent_evidence_status_is_failing_ignores_empty_placeholder_and_passed_case recent_evidence_resource_text_matches_report_shape build_recent_evidence_report_prefers_latest_failed_result_across_lanes sessionless_resource_specs_include_first_open_and_recent_evidence mcp_server_instructions_point_to_first_open_resource parse_resource_uri_accepts_recent_evidence_resource --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json > $null
python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/gate_imui_workstream_source.py
git diff --check
```

Result: passed locally. The focused `fret-devtools` recent-evidence/history gate reported 7 passed tests.
The `fret-devtools-mcp` focused nextest gate reported 9 passed tests, both
DevTools first-open/source gates passed, the workstream catalog and JSON checks passed, and
`git diff --check` passed with only the existing CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`. This is a DevTools/MCP first-open
productization slice only; broader GUI product maturity, real-host Wayland hand-feel, and full
perf/smoothness attribution remain open.

## Goal completion audit refresh - 2026-05-25

Scope: close the latest IMUI-side implementation slices without claiming external host acceptance
or broad product maturity as complete.

- `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-25.md`
  records the canonical workbench, Demo/Metrics/Debug, ListBox, plot adapter, style/theme preset
  picker, and table owner-split closeout evidence.
- The umbrella goal remains open for real-host Wayland hand-feel (`DW-P1-linux-003`), broader
  DevTools GUI productization, full perf/smoothness attribution, and broad porting sugar.
- The corresponding narrow lanes are closed with `scope_kind: closeout` and
  `default_action: start_follow_on`, so new work should start from owner-specific follow-ons rather
  than reopening those lane records.

## IMUI porting sugar layout follow-up - 2026-05-25

Scope: restore the small Dear ImGui porting helpers that reduce layout boilerplate without adding
an implicit window cursor or widening `fret-imui`.

- `fret-ui-kit::imui` now owns explicit closure-scoped `items`, `same_line`, `spacing`, `dummy`, and
  `indent` helpers.
- Default item spacing is theme-driven:
  - `component.imui.item_spacing_x_px` fallback `8px`
  - `component.imui.item_spacing_y_px` fallback `4px`
  - `component.imui.indent_spacing_px` fallback `21px`
- `fret-imui` remains a thin authoring frontend; the helpers are exposed through
  `UiWriterImUiFacadeExt` and `ImUiFacade`, both in `fret-ui-kit`.
- The parity audit now records this as explicit porting sugar, not a resurrection of Dear ImGui's
  global layout cursor.

Focused gates:

```text
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens --no-fail-fast
```

Result: passed locally. The layout test proves `same_line` uses the X spacing token, `items` and
`spacing` use the Y spacing token, `dummy(size)` preserves explicit size in both same-line and item
flow contexts, and `indent` uses the indent token.

## IMUI facade container-method owner split - 2026-05-25

Scope: keep the restored porting sugar and structural container wrappers from making
`facade_writer.rs` the long-term owner for container construction policy.

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods.rs` now owns the shared
  trait/facade container-method bodies for `items`, `same_line`, `dummy`, `spacing`, `indent`,
  horizontal/vertical/grid/scroll/child-region, list box, tab bar, table, menu bar, and virtual
  list.
- `UiWriterImUiFacadeExt` stays source-compatible but now delegates container methods to
  `container_methods::*` with `build_focus = None`.
- Inherent `ImUiFacade` container wrappers keep focus-tracker forwarding by passing their cloned
  `build_focus` into the same helper functions.
- The source gate now rejects direct `layout_sugar::*` and heavy container-builder calls from
  `facade_writer.rs` / `facade_writer/container_wrappers.rs`, so the trait hub and facade wrapper
  file stay thin.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
```

Result: `cargo check -p fret-ui-kit --features imui` passed locally after the split. The check
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI facade core owner split - 2026-05-25

Scope: keep `facade_writer.rs` as the public trait/default-method hub while moving the
`ImUiFacade` writer object, focus-capture helper, keyed child construction, and disabled-scope
wrapper into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core.rs` now owns `ImUiFacade`, its
  `UiWriter` implementation, `record_focusable`, `id`/`push_id`/`for_each_keyed`, `cx_mut`, `add`,
  and inherent `disabled_scope`.
- `facade_writer.rs` re-exports `ImUiFacade` and keeps `UiWriterImUiFacadeExt` as the public
  extension trait hub, including trait default methods that delegate to component/policy owners.
- `ImUiFacade` fields remain internal to `crate::imui` (`pub(in crate::imui)`) so existing IMUI
  element builders can construct nested facades without exposing writer internals outside the
  module boundary.
- The split reduces `facade_writer.rs` from 1420 lines after the previous facade/container splits
  to 1290 lines; the new `facade_core.rs` owner is 134 lines.
- The source gate now rejects `ImUiFacade` struct/core impl and `UiWriter` impl from returning to
  `facade_writer.rs`, and verifies the field visibility does not become fully public.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
python tools/gate_imui_workstream_source.py
```

## IMUI facade scope-method owner split - 2026-05-25

Scope: keep `facade_writer.rs` as the public trait/default-method hub while moving trait-level
scope construction for `UiWriterImUiFacadeExt::push_id` and
`UiWriterImUiFacadeExt::disabled_scope` into a smaller owner module.

- `ecosystem/fret-ui-kit/src/imui/facade_writer/scope_methods.rs` now owns trait-level keyed child
  construction, result capture, disabled-scope wrapping, disabled depth guards, pointer blocking,
  and focus traversal gating for any `UiWriterImUiFacadeExt` implementor.
- `facade_writer.rs` keeps the public trait methods and delegates to
  `scope_methods::push_id(...)` / `scope_methods::disabled_scope(...)`.
- `facade_core.rs` still owns the concrete `ImUiFacade` writer object and its inherent
  `push_id`/`disabled_scope`; this split only removes the trait-default implementation body from
  the trait hub.
- The split reduces `facade_writer.rs` from 1290 lines after the facade-core split to 1218 lines;
  the new `scope_methods.rs` owner is 98 lines.
- The source gate now rejects keyed result capture, disabled depth guards, pointer blocking, and
  focus traversal gate bodies from returning to `facade_writer.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
python tools/gate_imui_workstream_source.py
```

## IMUI facade basic-item owner split - 2026-05-25

Scope: keep `facade_writer.rs` as the public trait/default-method hub while moving simple item
construction into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/facade_writer/basic_items.rs` now owns trait-level `text`,
  `text_wrapped`, `bullet_text_with_options`, `debug_draw_with_options`, `separator`, and
  `separator_text_with_options` implementation bodies.
- `facade_writer.rs` keeps the public `UiWriterImUiFacadeExt` method roster and delegates those
  methods through `basic_items::*`, so authoring code remains source-compatible.
- The split reduces `facade_writer.rs` from 1218 lines after the scope-method split to 1205 lines;
  the new `basic_items.rs` owner is 70 lines.
- The source gate now rejects direct text element construction, bullet/separator/debug-draw policy
  calls, and separator line container construction from returning to `facade_writer.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui_text --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI facade test owner split - 2026-05-25

Scope: keep `facade_writer.rs` as the public trait/default-method roster while moving its local
text contract tests into a separate test owner file.

- `ecosystem/fret-ui-kit/src/imui/facade_writer/tests.rs` now owns the local `TestWriter` harness
  and the `imui_text_item_is_single_line_and_shrinkable` /
  `imui_text_wrapped_is_explicit_wrapping_text` contract tests.
- `facade_writer.rs` keeps only `#[cfg(test)] mod tests;` and no longer embeds test harness code at
  the bottom of the trait hub.
- The split reduces `facade_writer.rs` from 1205 lines after the basic-item split to 1122 lines;
  the new `tests.rs` owner is 83 lines.
- The source gate now rejects local test harness and text contract test bodies from returning to
  `facade_writer.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui_text --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI disclosure test owner split - 2026-05-25

Scope: keep `disclosure_controls.rs` as the disclosure implementation owner while moving its local
contract tests into a separate test owner file.

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/tests.rs` now owns the local `TestWriter`
  harness plus collapsing-header, tree-node semantics, palette, row-label, and indicator-glyph
  tests.
- `disclosure_controls.rs` keeps only `#[cfg(test)] mod tests;` and no longer embeds its local test
  harness at the bottom of the implementation file.
- The split reduces `disclosure_controls.rs` from 877 lines to 615 lines; the new disclosure
  `tests.rs` owner is 261 lines.
- The source gate now rejects local test harness and disclosure contract test bodies from returning
  to `disclosure_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui disclosure_controls::tests --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI disclosure visual owner split - 2026-05-25

Scope: keep `disclosure_controls.rs` focused on open-state, pressable behavior, response
population, and content mounting while moving header visuals into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual.rs` now owns disclosure a11y mapping,
  content padding, header row layout, indicator/label shared text roles, and disclosure palette
  resolution.
- `disclosure_controls.rs` delegates to
  `visual::disclosure_a11y(...)`, `visual::header_row(...)`, and
  `visual::disclosure_content_padding(...)`, while keeping shortcut handling, context-menu
  requests, hover-delay response fields, and open/toggled response population in the main owner.
- `disclosure_controls/tests.rs` now imports `AnyElement`, `ElementContext`, `Theme`, `Color`,
  `Px`, and `SemanticsRole` explicitly instead of relying on the parent implementation imports.
- The split reduces `disclosure_controls.rs` from 576 lines after the test split to 365 lines; the
  new `visual.rs` owner is 222 lines.
- The source gate now rejects visual text-role, palette, a11y, and header-row bodies from returning
  to `disclosure_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui disclosure_controls::tests --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `6 tests run: 6 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu-family test owner split - 2026-05-25

Scope: keep `menu_family_controls.rs` as the menu-family implementation owner while moving its
local visual contract test into a separate test owner file.

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/tests.rs` now owns the local
  `menu_trigger_visual_uses_button_label_text_role` test and its bounds helper.
- `menu_family_controls.rs` keeps only `#[cfg(test)] mod tests;` and no longer embeds the visual
  proof at the bottom of the implementation file.
- The split reduces `menu_family_controls.rs` from 894 lines to 849 lines; the new menu-family
  `tests.rs` owner is 44 lines.
- The source gate now rejects the local visual test body from returning to
  `menu_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_trigger_visual_uses_button_label_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI menu-family visual owner split - 2026-05-25

Scope: keep `menu_family_controls.rs` focused on menubar/menu/submenu state policy while moving
menu trigger chrome and label construction into a visual owner.

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/visual.rs` now owns
  `menu_trigger_visual`, including active/disabled foreground selection, accent background
  selection, trigger padding, radius, and shared `text_button_label(...)` role.
- `menu_family_controls.rs` delegates trigger row rendering through `visual::menu_trigger_visual`
  and keeps menubar open/close, active-row, popup, and submenu policy in the original owner.
- The split reduces `menu_family_controls.rs` from 849 lines after the test split to 804 lines; the
  new menu-family `visual.rs` owner is 60 lines.
- The source gate now rejects menu trigger chrome construction and shared button-label text
  construction from returning to `menu_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_trigger_visual_uses_button_label_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `1 test run: 1 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu-family trigger owner split - 2026-05-25

Scope: keep `menu_family_controls.rs` focused on menu/submenu state flow while moving the menu-bar
trigger pressable wiring, active-trigger response population, keyboard shortcut handling, and
menubar row registration into a trigger owner.

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger.rs` now owns
  `menu_trigger_with_options`, including visible-label identity parsing, pressable a11y props,
  activate/shortcut handling, menubar row sync, arrow-key opening, and response population.
- `menu_family_controls.rs` delegates trigger construction through
  `trigger::menu_trigger_with_options(...)` and keeps menu open/close, popup anchoring, submenu
  state selection, and popup rendering flow in the original owner.
- The split reduces `menu_family_controls.rs` from 804 lines after the visual split to 640 lines;
  the new menu-family `trigger.rs` owner is 179 lines.
- The source gate now rejects trigger pressable wiring, active-trigger behavior installation, and
  visible-label identity parsing from returning to `menu_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_trigger_visual_uses_button_label_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `1 test run: 1 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu-family submenu-state owner split - 2026-05-25

Scope: keep `menu_family_controls.rs` focused on submenu flow while moving the popup submenu model
clearing and selection writes into a state owner.

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state.rs` now owns submenu model
  clearing, pending-open reset, pointer-grace reset, focus retry reset, open-timer reset, and
  selected-submenu writes.
- `menu_family_controls.rs` delegates submenu state changes through
  `submenu_state::clear_imui_submenu(...)` and `submenu_state::select_imui_submenu(...)`, leaving
  trigger click interpretation and popup open/close flow in the original owner.
- The split reduces `menu_family_controls.rs` from 640 lines after the trigger split to 510 lines;
  the new menu-family `submenu_state.rs` owner is 143 lines.
- The source gate now rejects submenu model mutation details from returning to
  `menu_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_trigger_visual_uses_button_label_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `1 test run: 1 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu item visual/test owner split - 2026-05-25

Scope: keep `menu_controls.rs` focused on menu item pressable/activation flow while moving shared
menu item text visuals and local contract tests into narrower owners.

- `ecosystem/fret-ui-kit/src/imui/menu_controls/visual.rs` now owns
  `menu_item_label_text`, `menu_item_shortcut_text`, and `menu_item_indicator_text`, preserving the
  shared list-row label, control readout, and chrome glyph text roles.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/tests.rs` now owns the local visual-role and
  pressable-root contract tests.
- `menu_controls.rs` delegates row text construction through `visual::*` and keeps shortcut/action
  activation, popup close behavior, menubar trigger-row sync, and response population in the
  original owner.
- The split reduces `menu_controls.rs` from 596 lines to 459 lines; the new `visual.rs` owner is
  25 lines and the new `tests.rs` owner is 123 lines.
- The source gate now rejects menu item visual helpers and local test bodies from returning to
  `menu_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_controls::tests --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `4 tests run: 4 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu item element owner split - 2026-05-25

Scope: keep `menu_controls.rs` as the public menu-item API wrapper while moving pressable element
assembly, activation/shortcut handling, menubar row switching, and visual row assembly into an
element owner.

- `ecosystem/fret-ui-kit/src/imui/menu_controls/element.rs` now owns
  `menu_item_element_with_pressable_hook_inner`, including pressable a11y props, enabled/action
  gating, popup-close handling, item-local shortcut dispatch, popup-menu keyboard navigation,
  menubar horizontal-arrow switching, response population, and row child assembly.
- `menu_controls.rs` delegates to `element::menu_item_element_with_pressable_hook_inner(...)` and
  keeps the menu-item public helper wrappers plus label identity scoping.
- The split reduces `menu_controls.rs` from 459 lines after the visual/test split to 199 lines; the
  new `element.rs` owner is 281 lines.
- The source gate now rejects pressable wiring, command dispatch, menubar switching, and row
  assembly details from returning to `menu_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui menu_controls::tests --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `4 tests run: 4 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI text-controls test owner split - 2026-05-25

Scope: keep `text_controls.rs` as the text input/textarea implementation owner while moving its
local chrome contract tests into a separate test owner file.

- `ecosystem/fret-ui-kit/src/imui/text_controls/tests.rs` now owns the local `TestWriter` harness,
  text-input lookup helpers, text-area lookup helpers, and the compact IMUI chrome tests for
  `input_text_model_with_options` and `textarea_model_with_options`.
- `text_controls.rs` keeps only `#[cfg(test)] mod tests;` and no longer embeds local chrome proof
  code at the bottom of the implementation file.
- The split reduces `text_controls.rs` from 658 lines to 526 lines; the new text-controls
  `tests.rs` owner is 131 lines.
- The source gate now rejects local test harness, lookup helpers, and text chrome contract tests
  from returning to `text_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui text_controls::tests --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI text-controls focus/style/policy owner split - 2026-05-25

Scope: keep `text_controls.rs` as the text input/textarea API and element-assembly owner while
moving select-all focus timing, IMUI text chrome, and text command key policy into narrower owners.

- `ecosystem/fret-ui-kit/src/imui/text_controls/focus.rs` now owns
  `ImuiTextFocusSelectionState`, select-all-on-focus timer arming/canceling, transient event
  recording, and redraw request policy.
- `ecosystem/fret-ui-kit/src/imui/text_controls/style.rs` now owns text input chrome,
  textarea chrome, default input text style, and the fixed input layout tokens.
- `ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands.rs` now owns input history,
  completion, undo/redo, textarea submit, and textarea cancel key policy.
- `text_controls.rs` delegates to those owners and keeps model reads, response lifecycle
  population, filter plumbing, props assembly, and element mounting in one thin implementation
  owner.
- The split reduces `text_controls.rs` from 488 lines after the intermediate owner-file skeleton to
  209 lines; the new focused owners are `focus.rs` at 71 lines, `style.rs` at 95 lines, and
  `policy_commands.rs` at 122 lines.
- The source gate now rejects focus-selection state, chrome style bodies, and command-policy bodies
  from returning to `text_controls.rs`, while also checking the three new owner files directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui text_controls::tests --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. The
`fret-ui-kit` focused nextest gate reported `3 tests run: 3 passed`, including the two
text-controls chrome tests; `fret-imui models_text` reported `29 tests run: 29 passed`. The
`fret-imui` picker keyboard assertions now check the active-descendant relation for keyboard
highlight instead of treating transient highlight as selected semantics.

## IMUI text option-contract owner split - 2026-05-25

Scope: keep `options/controls.rs` as the general control option roster while moving text input,
text picker, text filter, custom filter, and textarea option contracts into a focused text-option
owner.

- `ecosystem/fret-ui-kit/src/imui/options/controls/text.rs` now owns `InputTextMode`,
  `InputTextFilters`, `InputTextCustomFilter`, `InputTextPickerFilter`,
  `InputTextPickerOptions`, `InputTextOptions`, `TextAreaSubmitKey`, and `TextAreaOptions`.
- `options/controls.rs` re-exports those text option types from the new owner, preserving the
  existing `fret_ui_kit::imui::*` public surface while keeping button/selectable/image/switch/
  slider/combo option contracts in the general controls roster.
- The split reduces `options/controls.rs` from 724 lines to 416 lines; the new `text.rs` owner is
  316 lines.
- The source gate now rejects text filter helpers, picker defaults, text command policy fields, and
  textarea submit defaults from returning to `options/controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui models_text --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_image_item_smoke --test imui_selectable_smoke --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. `fret-imui models_text` reported `29 tests run: 29 passed`; the
`fret-ui-kit` option smoke gate reported `6 tests run: 6 passed`; `cargo check` reported only the
existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code. An exploratory nextest filter form ran zero tests and was
replaced by the explicit `--test` command recorded above.

## IMUI boolean switch owner split - 2026-05-25

Scope: keep `boolean_controls.rs` focused on checkbox/radio item behavior plus the shared radio
indicator while moving switch active-trigger behavior into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs` now owns
  `switch_model_with_options`, switch active-trigger install/population, switch a11y,
  shortcut toggle policy, changed/clicked transient handling, and the On/Off badge row.
- `boolean_controls.rs` re-exports `switch_model_with_options` under the existing internal path,
  and keeps checkbox/radio label identity, checkbox model mutation, radio click response, context
  menu key handling, and the radio indicator visual.
- The split reduces `boolean_controls.rs` from 450 lines to 313 lines; the new `switch.rs` owner
  is 148 lines.
- The source gate now rejects switch active-trigger bodies from returning to
  `boolean_controls.rs` and checks the new switch owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui switch_model checkbox_changed_is_delivered_once_and_updates_model checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox button_family_variants_and_radio_mount_with_expected_bounds base_control_state_changes_keep_outer_bounds_stable control_disabled_state_changes_keep_outer_bounds_stable --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused boolean-control nextest gate reported `7 tests run: 7 passed`;
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code.

## IMUI floating-window resize-handle owner split - 2026-05-25

Scope: keep `floating_window_resize.rs` focused on resize drag snapshots, geometry clamping,
state updates, and snapped size/position output while moving resize-handle stack assembly,
handle layout, cursor policy, pointer capture, and pointer drag event wiring into a dedicated
owner.

- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles.rs` now owns
  `resize_stack_element`, handle layout for the eight resize handles, pointer-region construction,
  left-button drag start, pointer capture/release, cursor updates, resize activation events, and
  `update_immediate_move(...)` cancellation handling.
- `floating_window_resize.rs` re-exports `resize_stack_element` under the existing internal path
  for `floating_window_shell.rs`, but no longer owns handle layout or pointer event bodies.
- The split reduces `floating_window_resize.rs` from 462 lines to 238 lines; the new
  `handles.rs` owner is 229 lines.
- The source gate now rejects pointer-region handle bodies from returning to
  `floating_window_resize.rs` and checks the new handle owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui floating_window_resizable_false_hides_resize_handles floating_window_resizes_when_dragging_corner_handle floating_window_resizes_from_left_updates_origin_and_width floating_window_title_bar_double_click_toggles_collapsed floating_window_activate_on_click_can_be_disabled_for_resize_handles --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused floating-window nextest gate reported `5 tests run: 5 passed`;
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code.

## IMUI tab-family visual/test owner split - 2026-05-25

Scope: keep `tab_family_controls.rs` focused on selected-tab model normalization, tab trigger
activation/shortcut handling, active-trigger response population, and panel selection while moving
tab trigger visuals plus local visual proof into narrower owners.

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/visual.rs` now owns `tab_trigger_visual`,
  including foreground selection, hover background, selected underline, trigger padding, and the
  shared `text_button_label(...)` role.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/tests.rs` now owns the
  `tab_trigger_visual_uses_button_label_text_role` proof and its bounds helper.
- `tab_family_controls.rs` delegates trigger body rendering through
  `visual::tab_trigger_visual(...)` and keeps tab selection, keyboard shortcut handling, lifecycle
  updates, response population, and panel semantics in the original owner.
- The split reduces `tab_family_controls.rs` from 488 lines to 388 lines; the new `visual.rs`
  owner is 66 lines and the new `tests.rs` owner is 39 lines.
- The source gate now rejects tab trigger visual bodies and local visual tests from returning to
  `tab_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui tab_trigger_visual_uses_button_label_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `1 test run: 1 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI tab-family trigger owner split - 2026-05-25

Scope: keep `tab_family_controls.rs` focused on tab-bar orchestration, selected-tab normalization,
and panel/list semantics while moving trigger pressable behavior into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger.rs` now owns
  `render_tab_trigger`, `BuiltTabTrigger`, pressable props/a11y construction, activate/shortcut
  dispatch, lifecycle instant marking, clicked transient capture, active-trigger response
  population, and delegation to `visual::tab_trigger_visual(...)`.
- `tab_family_controls.rs` delegates trigger construction through `trigger::render_tab_trigger(...)`
  and keeps tab item collection, focus target selection, tab-list semantics, selected panel
  mounting, and selected-model normalization in the original owner.
- The split reduces `tab_family_controls.rs` from 388 lines after the visual/test split to 267
  lines; the new `trigger.rs` owner is 124 lines.
- The source gate now rejects pressable props, shortcut handlers, active-trigger response
  population, and trigger structs from returning to `tab_family_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui tab_trigger_visual_uses_button_label_text_role --no-fail-fast
cargo nextest run -p fret-imui tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger tab_bar_helper_reports_selected_change_and_trigger_edges --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The first `fret-ui-kit` visual nextest attempt timed out while waiting on
the parallel `fret-imui` cargo lock; the single rerun passed with `1 test run: 1 passed`. The
`fret-imui` tab behavior focused gate reported `3 tests run: 3 passed`; `cargo check` reported only
the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI table-column visibility menu owner split - 2026-05-25

Scope: keep `table_column_visibility.rs` as the public state/snapshot/API wrapper while moving
table-column visibility menu composition into a dedicated owner without changing public paths.

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu.rs` now owns header context-menu
  trigger selection, menu item group composition, checkbox menu item dispatch, response changed/
  edited propagation, visible-label filtering, and stable test-id suffix generation.
- `table_column_visibility.rs` keeps the public state/snapshot types and public helper function
  names, delegating the menu helper bodies through `menu::*` so existing callers keep the same API.
- The split reduces `table_column_visibility.rs` from 535 lines to 430 lines; the new `menu.rs`
  owner is 151 lines.
- The source gate now rejects popup/menu-item dispatch, label filtering, and test-id slug logic from
  returning to `table_column_visibility.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui table_column_visibility --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused nextest gate reported `11 tests run: 11 passed`; `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI popup modal owner split - 2026-05-25

Scope: keep `popup_overlay.rs` focused on popup store/open/close, menu/context orchestration, and
menu overlay requests while moving modal dialog overlay assembly into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` now owns modal open keep-alive,
  centered dialog panel placement, backdrop construction, outside-press/Escape dismissal policy,
  focus target capture, and `OverlayRequest::modal` submission.
- `popup_overlay.rs` keeps `open_popup*`, `close_popup`, popup menu construction, context-menu
  anchoring, and a thin `begin_popup_modal_with_options(...)` wrapper that preserves the existing
  private API path.
- The split reduces `popup_overlay.rs` from 512 lines to 363 lines; the new `modal.rs` owner is
  170 lines.
- The source gate now rejects modal dismiss/backdrop/panel/focus/request bodies from returning to
  `popup_overlay.rs` while keeping menu/context orchestration in the original owner.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui popup_hover::lifecycle_modal popup_hover::context_basics --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The focused `fret-imui` popup gate reported `6 tests run: 6 passed`; the
`fret-ui-kit` popup overlay perf/source smoke reported `1 test run: 1 passed`. `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI popup menu owner split - 2026-05-25

Scope: keep `popup_overlay.rs` as the popup lifecycle/API wrapper while moving popup-menu policy,
panel construction, popper placement, and dismissible-menu overlay request wiring into a dedicated
owner.

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu.rs` now owns `ImUiMenuNavState`,
  `ImUiPopupMenuPolicyState`, submenu policy sync, popup panel construction, menu-child rendering,
  popper placement from `environment_viewport_bounds(...)`, and
  `menu_root::dismissible_menu_request_with_modal_and_dismiss_handler(...)`.
- `popup_overlay.rs` keeps `popup_open_model`, `drop_popup_scope`, `open_popup*`, `close_popup`,
  context-menu anchoring, and thin menu/modal wrappers. It re-exports the internal menu state types
  under the same `popup_overlay::*` path so sibling IMUI modules keep their existing imports.
- The split reduces `popup_overlay.rs` from 363 lines after the modal split to 102 lines; the new
  `menu.rs` owner is 287 lines.
- The `imui_perf_guard_smoke` popup source check now reads `popup_overlay/menu.rs`, matching the
  new owner for the environment-viewport popper contract.
- The source gate now rejects popup menu policy/build/request bodies from returning to
  `popup_overlay.rs` while keeping modal-only bodies in `modal.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui popup_hover --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds --no-fail-fast
python tools/gate_imui_workstream_source.py
```

Result: passed locally for the listed focused gates. The full `fret-imui` popup-hover gate reported
`21 tests run: 21 passed`; the `fret-ui-kit` popup overlay perf/source smoke reported
`1 test run: 1 passed`. The hit-test close-popup regression now accepts the target menu-item
pressable or one of its descendants, matching runtime hit routing while still proving that the
subsequent click closes the popup.

## IMUI table render owner split - 2026-05-25

Scope: keep `table_controls.rs` as the table builder/API hub while moving the render tail and
shared cell layout helpers into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now owns `render_table`, row/column
  test-id suffixing, palette resolution, `table_cell_layout`, `table_cell_padding`, `empty_cell`,
  and `pack_cell_children`.
- `table_controls.rs` still collects rows/cells and preserves the public authoring shape, but it
  now delegates table rendering through `render::render_table` and multi-child cell packing through
  `render::pack_cell_children`.
- `table_controls/body.rs` and `table_controls/header.rs` import the shared cell helpers from
  `render::{...}`, so body/header keep their specialized row and header interaction owners without
  pulling the render tail back into the API hub.
- The source gate now rejects `render_table`, cell layout helpers, palette resolution, pinned body
  wrappers, and header trigger internals from `table_controls.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui hidden_table_columns_do_not_render_header_body_or_response horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups table_header_label_uses_shared_table_cell_text_role table_sort_indicator_uses_shared_chrome_glyph_text_role --no-fail-fast
python tools/gate_imui_workstream_source.py
```

## IMUI table header trigger/resize owner split - 2026-05-25

Scope: keep `table_controls/header.rs` as the table-header coordination owner while moving the two
mechanism-heavy header internals into focused child owners.

- `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger.rs` now owns sortable/plain header
  pressable trigger construction, pressable a11y props, and sortable header visual
  hover/focus/pressed chrome.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger/behavior.rs` now owns sortable
  header keyboard activation lifecycle marking, clicked transient draining for plain headers, and
  active-trigger response population.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/resize.rs` now owns table column resize
  handle constants, pointer-region construction, resize cursor capture, pointer drag start/move/up
  handling, resize drag response population, and resize handle visual chrome.
- `header.rs` keeps visible-label parsing, sortable a11y label construction, public header label
  and sort-indicator text roles, sortable/plain cell assembly, and final cell wrapping.
- The split reduces `header.rs` from 453 lines to 204 lines; the new `trigger.rs` owner is 150
  lines and the new `resize.rs` owner is 118 lines.
- The source gate now rejects trigger/resize implementation bodies from returning to `header.rs`
  and checks the new owner files directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui hidden_table_columns_do_not_render_header_body_or_response horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups table_header_label_uses_shared_table_cell_text_role table_sort_indicator_uses_shared_chrome_glyph_text_role --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused table gate reported `4 tests run: 4 passed`. `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI debug draw paint-shape owner split - 2026-05-25

Scope: keep `debug_draw_controls/paint.rs` as the command-stream dispatcher while moving the
path/shape/text paint branches into a narrower owner module.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes.rs` remains the command-stream
  dispatcher and still exposes `paint_debug_draw_shape_command(...)`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths.rs` now owns path-based
  line/polyline/polygon/outline/quad/triangle/circle/ngon/ellipse/bezier paint branches.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/rects.rs` now owns filled rect
  and vertex-color rect scene-op emission.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/text.rs` now owns debug-draw text
  emission and canvas text constraints.
- `paint.rs` keeps clip depth balancing, image/svg command dispatch, rounded-image clipping, and
  explicit delegation to `paint_shapes::paint_debug_draw_shape_command(...)` for shape commands.
- The follow-up split reduces `paint_shapes.rs` from 451 lines to 196 lines; the new owners are
  `paths.rs` at 395 lines, `rects.rs` at 37 lines, and `text.rs` at 33 lines.
- The source gate now rejects path/rect/text paint bodies from returning to `paint_shapes.rs` while
  keeping image opacity/UV/rounded-image helpers in `paint_helpers.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw gate reported `39 tests run: 39 passed`. `cargo
check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI debug draw test owner split - 2026-05-25

Scope: keep debug-draw behavior proof close to the debug-draw owner while removing the oversized
single test module that mixed element, draw-list, path-builder, paint-helper, style, and path
primitive contracts.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests.rs` is now a 44-line test facade with
  shared helpers plus submodule declarations only.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/element.rs` owns canvas/pressable
  element tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list.rs` is now a 4-line facade
  for draw-list-specific test owners.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/commands.rs` owns command
  recorder, image overlay, mesh, and concave-polygon command tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/summaries.rs` owns command
  summary, list summary, and clip-stack tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/channels.rs` owns channel
  split/switch/merge tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/path_builder.rs` owns path-builder
  stroke/fill, rounded rect, Bezier, arc, elliptical arc, and invalid-finish tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/paint_helpers.rs` owns opacity/UV and
  rounded-image helper tests.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/paths.rs` and
  `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/style.rs` own primitive path and stroke
  style policy tests.
- The source gate now checks the test facade, draw-list test subfacade, and each test owner
  directly, preventing the large mixed test bodies from returning to
  `debug_draw_controls/tests.rs` or `tests/draw_list.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code.

## IMUI debug draw command summary projection owner split - 2026-05-25

Scope: keep `debug_draw_controls/commands.rs` focused on recorded command payload variants while
moving command-to-summary and clip-stack projection into a dedicated child owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs` now owns the private
  `DebugDrawCommand` payload enum and declares the summary projection child module.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection.rs` now owns
  `summary_with_clip_state(...)`, the internal `summary()` projection, point/vertex/index/triangle
  count mapping, image command mapping, and clip-stack depth/rect projection.
- The split reduces `commands.rs` from 396 lines after the previous debug-draw command split to
  192 lines; the new `summary_projection.rs` owner is 207 lines.
- The source gate now rejects summary projection bodies from returning to `commands.rs` and checks
  that `summary_projection.rs` does not regain command payload definitions or image/svg option
  imports.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI debug draw path paint owner split - 2026-05-25

Scope: keep `debug_draw_controls/paint_shapes/paths.rs` as a narrow facade while moving stroked
path painting, filled path painting, and shared canvas path dispatch into focused owners.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths.rs` is now an 11-line
  facade that declares `common`, `filled`, and `stroked` and re-exports the existing path-paint
  functions to `paint_shapes.rs`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked.rs` now owns line,
  polyline, rect, quad, triangle, circle, ngon, ellipse, and Bezier stroked path paint branches.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled.rs` now owns
  convex/concave polygon fill, quad fill, triangle fill, circle fill, ngon fill, and ellipse fill
  path paint branches.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/common.rs` now owns the
  shared `paint_path(...)` canvas dispatch helper.
- The split reduces `paint_shapes/paths.rs` from 395 lines after the previous paint-shape split to
  11 lines; the new owners are `stroked.rs` at 264 lines, `filled.rs` at 123 lines, and
  `common.rs` at 21 lines.
- The source gate now rejects paint bodies from returning to the facade and checks stroked, filled,
  and common dispatch owner boundaries directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI debug draw path geometry owner split - 2026-05-25

Scope: keep `debug_draw_controls/paths.rs` focused on low-level path primitive construction while
moving point sampling and rounded-rect path construction into dedicated owners.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs` now owns the primitive path roster:
  stroke point minimums, polyline/convex/concave paths, triangle/quad/circle/ngon/ellipse paths,
  and native quadratic/cubic Bezier path commands.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling.rs` now owns default segment
  selection, arc/elliptical-arc point sampling, and quadratic/cubic Bezier point sampling helpers.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects.rs` now owns `rect_path(...)`,
  rounded-rect point generation, corner masking, and the local rect max-point helper.
- `paths.rs` re-exports the moved helpers under the existing internal module path, preserving
  `path_builder.rs` and paint-shape owner call sites.
- The split reduces `paths.rs` from 363 lines to 154 lines; the new owners are `sampling.rs` at 115 lines and `rects.rs` at 110 lines.
- The source gate now rejects sampling/rounded-rect bodies from returning to `paths.rs` and checks
  the new owner boundaries directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI debug draw draw-list shape facade split - 2026-05-25

Scope: keep the `ImUiDebugDrawList` shape-authoring API unchanged while moving shape recorder method
families out of the mixed `draw_list_shapes.rs` owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes.rs` is now a 4-line facade
  declaring `linear`, `meshes`, `round`, and `beziers`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear.rs` now owns line,
  polyline, convex/concave fill, rect, quad, and triangle command recorder methods.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/meshes.rs` now owns
  triangle-list and explicit triangle-mesh command recorder methods, including sequential index
  generation for triangle lists.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/round.rs` now owns circle,
  ngon, and ellipse command recorder methods.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/beziers.rs` now owns
  quadratic and cubic Bezier command recorder methods.
- The split reduces `draw_list_shapes.rs` from 338 lines to 4 lines; the new owners are
  `linear.rs` at 176 lines, `round.rs` at 118 lines, `beziers.rs` at 65 lines, and `meshes.rs` at
  30 lines.
- The source gate now rejects command recorder method bodies from returning to the facade and checks
  each method-family owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI debug draw draw-list runtime facade split - 2026-05-25

Scope: keep the `ImUiDebugDrawList` runtime API unchanged while moving channel, summary, clip,
image, SVG/text, and core list behavior out of the mixed `draw_list.rs` owner.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list.rs` is now a 6-line facade declaring
  `channels`, `clips`, `core`, `images`, `summaries`, and `svg_text`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/core.rs` now owns `path(...)`,
  `command_count`, `is_empty`, and `Default`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/channels.rs` now owns channel
  split/switch/merge and the internal `for_each_command_with_channel(...)` traversal helper.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/summaries.rs` now owns command and
  aggregate list summary projection.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/clips.rs` now owns clip stack command
  recorders.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images.rs` now owns image, image
  region, image quad, rounded image, and image triangle mesh command recorders.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/svg_text.rs` now owns SVG image,
  SVG mask icon, and text command recorders.
- The split reduces `draw_list.rs` from 309 lines to 6 lines; the new owners are `images.rs` at 145
  lines, `channels.rs` at 66 lines, `svg_text.rs` at 51 lines, `core.rs` at 34 lines,
  `summaries.rs` at 25 lines, and `clips.rs` at 14 lines.
- The source gate now rejects runtime method bodies from returning to the facade and checks each
  draw-list owner family directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI debug draw stroked path paint owner split - 2026-05-25

Scope: keep `paint_shapes/paths/stroked.rs` as the stroked path-paint facade while moving linear,
round, and Bezier paint branches into focused owners.

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked.rs` is now a
  13-line facade that re-exports `linear`, `round`, and `beziers`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear.rs` now
  owns line, polyline, rect, quad, and triangle stroked path paint branches.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/round.rs` now owns
  circle, ngon, and ellipse stroked path paint branches.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/beziers.rs` now
  owns quadratic and cubic Bezier stroked path paint branches.
- The split reduces `stroked.rs` from 276 lines to 13 lines; the new owners are `linear.rs` at 138
  lines, `round.rs` at 89 lines, and `beziers.rs` at 60 lines.
- The source gate now rejects stroked paint bodies from returning to the facade and checks the
  linear/round/Bezier owner boundaries directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused debug-draw nextest gate reported `39 tests run: 39 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI selectable visual/test owner split - 2026-05-25

Scope: keep `selectable_controls.rs` focused on selectable pressable behavior, activation
shortcuts, context-menu request events, popup arrow navigation, and response population while
moving row visual policy plus local unit tests into narrower owners.

- `ecosystem/fret-ui-kit/src/imui/selectable_controls/visual.rs` now owns
  `SelectablePalette`, `selectable_row_element(...)`, selected/hover/highlight foreground and
  background resolution, row padding, row radius, and shared list-row text construction.
- `ecosystem/fret-ui-kit/src/imui/selectable_controls/tests.rs` now owns selectable palette and
  row-label text-role unit tests.
- `selectable_controls.rs` keeps label identity, semantics, pressable item behavior, keyboard
  activation shortcut handling, context-menu keyboard requests, popup menu navigation, and response
  lifecycle population.
- The split reduces `selectable_controls.rs` from 378 lines to 184 lines; the new `visual.rs`
  owner is 83 lines and `tests.rs` is 114 lines.
- The source gate now rejects palette bodies, direct row text construction, and test bodies from
  returning to `selectable_controls.rs`, while checking that `visual.rs` stays free of activation
  and keyboard behavior.

Focused gates:

```text
cargo fmt -p fret-ui-kit -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui selectable_controls::tests --no-fail-fast
cargo nextest run -p fret-imui selectable_activate_shortcut_is_scoped_to_focused_item selectable_activate_shortcut_preserves_popup_arrow_nav --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `fret-ui-kit` selectable focused gate reported
`3 tests run: 3 passed`; the `fret-imui` behavior gate reported `2 tests run: 2 passed`. `cargo
check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI drag/drop source-store-target owner split - 2026-05-25

Scope: keep `drag_drop.rs` as the stable internal module path while moving immediate-mode
drag/drop store lifecycle, drag-source handlers, drop-target response resolution, and local tests
into focused owners.

- `ecosystem/fret-ui-kit/src/imui/drag_drop/store.rs` now owns the per-app drag/drop store model,
  active payloads, delivered payloads, stale-session pruning, source response projection, active
  payload lookup, and one-tick delivery expiry.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/source.rs` now owns `drag_source_with_options(...)`,
  including trigger-id gating, optional cross-window drag promotion, pointer-move active payload
  registration, pointer-up delivery write, and source response projection.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/target.rs` now owns `drop_target_with_options(...)`,
  including delivered payload extraction, active preview projection, hover-target tracking, and
  `DropTargetResponse` population.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/tests.rs` now owns the no-trigger-id source/target
  local unit tests.
- `drag_drop.rs` is now an 8-line module facade preserving the existing internal
  `drag_drop::drag_source_with_options` and `drag_drop::drop_target_with_options` paths.
- The split moves the former 401-line implementation into `source.rs` at 135 lines, `store.rs` at
  161 lines, `target.rs` at 63 lines, and `tests.rs` at 65 lines.
- The source gate now rejects store/source/target/test bodies from returning to `drag_drop.rs`, and
  separately checks that store stays free of event-handler policy while source and target stay in
  their respective responsibilities.

Focused gates:

```text
cargo fmt -p fret-ui-kit -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui drag_drop --no-fail-fast
cargo nextest run -p fret-imui drag_drop_helper_previews_and_delivers_payload --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The first `fret-ui-kit drag_drop` focused run reported
`3 tests run: 3 passed` and exposed one local unused-import warning in the moved tests; after
removing that import, the rerun again reported `3 tests run: 3 passed`. The `fret-imui` payload
preview/delivery gate reported `1 test run: 1 passed`. `cargo check` reported only the existing
`fret-ui` warnings for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead
code.

## IMUI floating-surface layer owner split - 2026-05-25

Scope: keep `floating_surface.rs` focused on floating-area state, drag surface wiring, and shared
floating-window ids while moving layer registration, z-order state, snapshot reuse, sorting, and
layer container assembly into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/floating_surface/layer.rs` now owns
  `FloatWindowLayerMarker`, `FloatWindowLayerZOrder`, z-order snapshot reuse, child registration,
  activation bring-to-front, layer sorting, and the full-viewport floating-layer container.
- `floating_surface.rs` keeps `FloatingAreaState`, `FloatWindowState`, drag/resize kind ids,
  `floating_area_element(...)`, and `floating_area_drag_surface_element(...)`, delegating
  layer-child registration and bring-to-front handling through `layer::*`.
- The split reduces `floating_surface.rs` from about 432 lines to 331 lines; the new `layer.rs`
  owner is 164 lines.
- The source gate now rejects layer marker/z-order/sorting bodies from returning to
  `floating_surface.rs`, checks the new layer owner directly, and verifies that drag-surface
  pointer policy stays out of `floating_surface/layer.rs`.
- `imui_perf_guard_smoke::floating_layer_z_order_does_not_clone_vec_each_frame` now reads
  `floating_surface/layer.rs`, so the existing source smoke follows the new owner instead of the
  old aggregate module.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui floating_layer_z_order_does_not_clone_vec_each_frame --no-fail-fast
cargo nextest run -p fret-imui floating_area_bring_to_front_updates_hit_test_order floating_layer_bring_to_front_updates_hit_test_order floating_window_focus_on_click_can_be_independent_from_z_order_activation floating_window_activate_on_click_can_be_disabled_for_content floating_window_activate_on_click_can_be_disabled_for_resize_handles --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The first `fret-ui-kit` perf-smoke attempt timed out while waiting on the
build directory lock; the rerun with the same filter reported `1 test run: 1 passed`. The
`fret-imui` floating z-order/activation focused gate reported `5 tests run: 5 passed`. `cargo
check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI menu-family submenu owner split - 2026-05-25

Scope: keep `menu_family_controls.rs` focused on menubar policy state, menu-bar assembly, and
top-level `begin_menu` orchestration while moving `begin_submenu` trigger/popup orchestration into
a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu.rs` now owns
  `begin_submenu_with_options(...)`, submenu menu-item trigger construction, submenu popper
  geometry hints, open/close selection routing through `submenu_state`, and nested popup-menu
  mounting.
- `menu_family_controls.rs` re-exports `begin_submenu_with_options` under the existing internal
  path, and keeps menubar policy models, `menu_bar_element(...)`, top-level `begin_menu`, trigger
  sync, and tests module wiring.
- The split reduces `menu_family_controls.rs` from 510 lines to 351 lines; the new `submenu.rs`
  owner is 167 lines.
- The source gate now rejects submenu trigger/body logic from returning to
  `menu_family_controls.rs`, checks the new submenu owner directly, and keeps menubar trigger-row
  policy out of `submenu.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui begin_submenu --no-fail-fast
cargo nextest run -p fret-imui menu_and_submenu_helpers_report_toggle_and_trigger_edges --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `begin_submenu` focused gate reported `7 tests run: 7 passed`; the
response-edge focused gate reported `1 test run: 1 passed`. `cargo check` reported only the
existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code.

## IMUI table-column visibility state/test owner split - 2026-05-25

Scope: keep `table_column_visibility.rs` as the public response/wrapper hub while moving pure
runtime visibility state, persistence-friendly snapshots, and local unit tests into dedicated
owners.

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state.rs` now owns
  `ImUiTableColumnVisibilityState`, `TableColumnVisibilitySnapshot`,
  `TableColumnVisibilityEntry`, private override storage, snapshot restore, visibility toggling,
  and `apply_to_columns(...)`.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/tests.rs` now owns the local
  state/snapshot/menu-label unit tests.
- `table_column_visibility.rs` re-exports the public state/snapshot types under the same
  `fret_ui_kit::imui::*` surface, and keeps only menu options, response accessors, public helper
  wrappers, and `#[cfg(test)] mod tests;`.
- The split reduces `table_column_visibility.rs` from 506 lines to 176 lines; the new `state.rs`
  owner is 217 lines and `tests.rs` is 124 lines.
- The source gate now rejects state/snapshot impls and inline local tests from returning to
  `table_column_visibility.rs`, checks the state owner directly, and keeps menu/widget policy out
  of `state.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui table_column_visibility --no-fail-fast
cargo nextest run -p fret-imui table_column_visibility --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `fret-ui-kit` table-column visibility focused gate reported
`11 tests run: 11 passed`; the `fret-imui` behavior gate reported `4 tests run: 4 passed`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code.

## IMUI tooltip text/test owner split - 2026-05-25

Scope: keep `tooltip_overlay.rs` focused on trigger gates, placement, hoverable-content policy,
overlay request wiring, and dismissal while moving text-body helper policy plus local unit tests
into dedicated owners.

- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/text.rs` now owns
  `tooltip_text_with_options(...)` and `tooltip_body_text(...)`, including the compact paragraph
  role used for simple tooltip copy.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/tests.rs` now owns the no-trigger-id guard,
  compact paragraph text-role proof, and default-options proof.
- `tooltip_overlay.rs` re-exports `tooltip_text_with_options` under the existing internal path,
  and keeps pointer-move open gating, Radix tooltip interaction updates, popper layout, panel
  assembly, overlay request creation, and dismiss handling.
- The split reduces `tooltip_overlay.rs` from 401 lines to 283 lines; the new `text.rs` owner is
  30 lines and `tests.rs` is 103 lines.
- The source gate now rejects text helper bodies and inline tests from returning to
  `tooltip_overlay.rs`, while keeping Radix interaction/update/request logic out of `text.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui tooltip --no-fail-fast
cargo nextest run -p fret-imui hovered_for_tooltip_requires_stationary_and_delay_short_even_when_disabled --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `fret-ui-kit` tooltip focused gate reported
`32 tests run: 32 passed`; the `fret-imui` disabled tooltip-hover focused gate reported
`1 test run: 1 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.

## IMUI child-region response owner split - 2026-05-25

Scope: keep `response/widgets.rs` focused on shared widget response families while moving
child-region resize response contracts and tests into a dedicated owner.

- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region.rs` now owns
  `ChildRegionResponse`, `ChildRegionResizeXResponse`, `ChildRegionResizeYResponse`, resize
  mutators, drag accessors, and min/max clamped size projection helpers.
- `response/widgets.rs` re-exports the child-region response family under the existing internal
  module path and continues to own disclosure, combo, text-picker, tab, table, and virtual-list
  response families.
- The split reduces `response/widgets.rs` from 540 lines to 351 lines; the new child-region
  response owner is 198 lines.
- The opaque public-struct gate now points the child-region response family at the new owner, and
  the source gate rejects child-region response bodies from returning to `response/widgets.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast
cargo nextest run -p fret-imui child_region --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
```

Result: passed locally. The `fret-ui-kit` child-region response focused gate reported
`4 tests run: 4 passed`; the `fret-imui` child-region behavior gate reported
`6 tests run: 6 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.

## IMUI button/image option-contract owner split - 2026-05-25

Scope: keep `options/controls.rs` as the general control-option roster while moving button and
image-item option contracts into a dedicated pure-data owner.

- `ecosystem/fret-ui-kit/src/imui/options/controls/button_image.rs` now owns
  `ButtonArrowDirection`, `ButtonVariant`, `ButtonOptions`, `ImageItemVariant`, and
  `ImageItemOptions`, including button/image defaults plus image-item builder helpers.
- `options/controls.rs` re-exports those types under the existing internal module path and
  continues to own disclosure, selectable, boolean, combo, slider, and combo-model option
  contracts alongside the existing text owner re-export.
- The split reduces `options/controls.rs` from 416 lines after the text option split to 307 lines;
  the new `button_image.rs` owner is 114 lines.
- The source gate now rejects button/image option contracts and default bodies from returning to
  `options/controls.rs`, while keeping selectable/boolean/combo/text option families out of
  `button_image.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_image_item_smoke --no-fail-fast
cargo nextest run -p fret-imui button_family_variants_and_radio_mount_with_expected_bounds --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally after rerunning the nextest gates serially. The first parallel nextest
attempts timed out under concurrent build load; the serial reruns reported `3 tests run: 3 passed`
for `fret-ui-kit` button/image smoke and `1 test run: 1 passed` for the `fret-imui` button-family
geometry gate. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. `git diff --check`
reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI disclosure option-contract owner split - 2026-05-25

Scope: keep `options/controls.rs` as the general control-option roster while moving disclosure
option contracts into a dedicated pure-data owner.

- `ecosystem/fret-ui-kit/src/imui/options/controls/disclosure.rs` now owns
  `CollapsingHeaderOptions` and `TreeNodeOptions`, including default-open, tree level, set
  position, leaf/selected, content test-id, and item-local shortcut defaults.
- `options/controls.rs` re-exports those types under the existing internal module path and
  continues to own tab, selectable, boolean, combo, slider, and combo-model option contracts
  alongside the existing button/image and text owner re-exports.
- The split reduces `options/controls.rs` from 307 lines after the button/image split to 238 lines;
  the new `disclosure.rs` owner is 72 lines.
- The source gate now rejects disclosure option contracts and tree defaults from returning to
  `options/controls.rs`, while keeping selectable/boolean/combo/text/button/image option families
  out of `disclosure.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_disclosure_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui disclosure_controls::tests --no-fail-fast
cargo nextest run -p fret-imui collapsing_header_activate_shortcut_is_scoped_to_focused_trigger tree_node_activate_shortcut_preserves_shift_f10_context_menu_request tree_node_children_stack_vertically_inside_open_parents --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The `fret-ui-kit` disclosure smoke gate reported
`1 test run: 1 passed`; the `fret-ui-kit` disclosure-controls focused gate reported
`6 tests run: 6 passed`; the `fret-imui` disclosure/tree behavior gate reported
`3 tests run: 3 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. `git diff --check`
reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI boolean option-contract owner split - 2026-05-25

Scope: keep `options/controls.rs` as the general control-option roster while moving checkbox,
radio, and switch option contracts into a dedicated pure-data owner.

- `ecosystem/fret-ui-kit/src/imui/options/controls/boolean.rs` now owns `CheckboxOptions`,
  `RadioOptions`, and `SwitchOptions`, including focusability, accessibility label, test-id, and
  item-local shortcut defaults.
- `options/controls.rs` re-exports those types under the existing internal module path and
  continues to own tab, selectable, combo, slider, and combo-model option contracts alongside the
  existing button/image, disclosure, and text owner re-exports.
- The split reduces `options/controls.rs` from 238 lines after the disclosure split to 162 lines;
  the new `boolean.rs` owner is 79 lines.
- The source gate now rejects boolean option contracts from returning to `options/controls.rs`,
  while keeping selectable/slider/combo/text/button/image/disclosure option families out of
  `boolean.rs`.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-imui checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox switch_model_activate_shortcut_is_scoped_to_focused_switch base_control_state_changes_keep_outer_bounds_stable --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The `fret-ui-kit` button/adapter focused gate reported
`4 tests run: 4 passed`; the `fret-imui` checkbox/switch/control-geometry gate reported
`3 tests run: 3 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. `git diff --check`
reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI remaining control option facade split - 2026-05-25

Scope: finish turning `options/controls.rs` into a pure module facade by moving the remaining
tab, selectable, combo, and value option contracts into dedicated owners.

- `ecosystem/fret-ui-kit/src/imui/options/controls/tab.rs` now owns `TabItemOptions`, including
  default-selected, panel test-id, and item-local shortcut defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/selection.rs` now owns `SelectableOptions`,
  including highlighted-but-not-selected semantics, popup-close model, accessibility role defaults,
  and item-local shortcut defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/combo.rs` now owns `ComboOptions` and
  `ComboModelOptions`, including popup defaults, placeholder defaults, and item-local shortcut
  defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/value.rs` now owns `SliderOptions`, including
  min/max/step defaults.
- `options/controls.rs` is now a 21-line facade containing only module declarations and re-exports.
  The new owners are 27 lines (`tab.rs`), 41 lines (`selection.rs`), 60 lines (`combo.rs`), and
  24 lines (`value.rs`).
- The source gate now rejects all control option struct/default bodies from returning to
  `options/controls.rs`, and keeps each option family out of the other owner files.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --test imui_selectable_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui selectable_controls::tests tab_trigger_visual_uses_button_label_text_role --no-fail-fast
cargo nextest run -p fret-imui tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger tab_bar_helper_reports_selected_change_and_trigger_edges selectable_activate_shortcut_is_scoped_to_focused_item selectable_activate_shortcut_preserves_popup_arrow_nav combo_activate_shortcut_is_scoped_to_focused_trigger combo_model_activate_shortcut_is_scoped_to_focused_trigger combo_can_commit_selection_with_selectable_rows slider_f32_model_reports_changed_once_after_pointer_input --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The `fret-ui-kit` combo/selectable smoke gate reported
`3 tests run: 3 passed`; the `fret-ui-kit` selectable/tab focused gate reported
`4 tests run: 4 passed`; the `fret-imui` tab/selectable/combo/slider behavior gate reported
`9 tests run: 9 passed`. `cargo check` reported only the existing `fret-ui` warnings for
`unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code. `git diff --check`
reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI container test owner split - 2026-05-25

Scope: keep `containers.rs` focused on structural container element assembly while moving local
layout/test-id contract tests into a dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/containers.rs` now keeps the child-building helper plus
  horizontal, vertical, scroll, and grid container element builders.
- `ecosystem/fret-ui-kit/src/imui/containers/tests/mod.rs` now owns the shared local test harness,
  with `tests/layout.rs` covering layout forwarding and `tests/identity.rs` covering outer
  `test_id` plus scroll viewport `test_id` contracts.
- `containers.rs` is a 146-line implementation file with only `#[cfg(test)] mod tests;` for the
  local unit-test module; the split container test owners keep layout and identity assertions in
  separate files.
- The source gate now rejects local test harness and container contract test bodies from returning
  to `containers.rs`, and checks the split test owners directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::containers::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`4 tests run: 4 passed, 731 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI multi-select test owner split - 2026-05-25

Scope: keep `multi_select.rs` focused on the immediate multi-select state model, controllable model
hook, and click/range selection behavior while moving local model tests into a dedicated test
owner.

- `ecosystem/fret-ui-kit/src/imui/multi_select.rs` still owns `ImUiMultiSelectState`,
  `multi_select_use_model(...)`, `multi_selectable_with_options(...)`, and the private
  click/toggle/range selection helpers.
- `ecosystem/fret-ui-kit/src/imui/multi_select/tests.rs` now owns the plain-click,
  primary-modifier toggle, ordered-selection normalization, missing-anchor repair, and shift-range
  selection tests.
- `multi_select.rs` is now a 182-line implementation file with only `#[cfg(test)] mod tests;` for
  the local test module; `multi_select/tests.rs` is a 113-line test owner.
- The source gate now rejects the local test fixture and multi-select contract test bodies from
  returning to `multi_select.rs`, while preserving the existing no-`BeginMultiSelect` and opaque
  state guards.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::multi_select::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`6 tests run: 6 passed, 729 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI virtual-list test owner split - 2026-05-25

Scope: keep `virtual_list_controls.rs` focused on virtual-list element assembly, runtime option
projection, row wrapping, row height resolution, and row test-id generation while moving local row
height/overflow tests into a dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs` still owns `virtual_list_element(...)`,
  runtime option projection, row packing/wrapping, fixed/known/measured row height behavior, and
  row test-id derivation.
- `ecosystem/fret-ui-kit/src/imui/virtual_list_controls/tests.rs` now owns the fixed-height,
  known-height, and measured-row overflow contract tests.
- `virtual_list_controls.rs` is now a 183-line implementation file with only
  `#[cfg(test)] mod tests;` for the local test module; `virtual_list_controls/tests.rs` is an
  82-line test owner.
- The source gate now rejects the local row-height test bodies from returning to
  `virtual_list_controls.rs`, while checking the split test owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::virtual_list_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`3 tests run: 3 passed, 732 skipped` after waiting on an existing package-cache file lock.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI control chrome test owner split - 2026-05-25

Scope: keep `control_chrome.rs` focused on shared IMUI chrome constants, palette resolution, chrome
props, row/stack props, and compact text helpers while moving local text-role contract tests into a
dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` still owns button/field chrome, compact
  control/fill/caption text helpers, row/stack props, and `pill(...)`.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/tests.rs` now owns the button-label and fill-label
  text role tests, including single-line, shrinkable, inherited foreground behavior.
- `control_chrome.rs` is now a 235-line implementation file with only `#[cfg(test)] mod tests;`
  for the local test module; `control_chrome/tests.rs` is a 59-line test owner.
- The source gate now rejects local text-role test bodies from returning to `control_chrome.rs`,
  while checking the split test owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
CARGO_INCREMENTAL=0 cargo nextest run -p fret-ui-kit --features imui imui::control_chrome::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The first focused nextest attempt failed while compiling after `rustc-LLVM
ERROR: IO failure on output stream: no space on device`; `cargo clean -p fret-ui-kit` freed about
1.0GiB, and the rerun with `CARGO_INCREMENTAL=0` reported `2 tests run: 2 passed, 733 skipped`.
`cargo check` reported only the existing `fret-ui` warnings for `unstable-retained-bridge`
check-cfg and `current_effective_opacity` dead code. `git diff --check` reported only the
pre-existing line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI image-item test owner split - 2026-05-25

Scope: keep `image_item_controls.rs` focused on response-bearing image-item assembly, chrome
selection, image prop projection, and sanitization helpers while moving local helper tests into a
dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs` still owns `image_item_with_options(...)`,
  pressable image/button behavior, chrome selection, image prop projection, size sanitization,
  opacity normalization, and UV validation.
- `ecosystem/fret-ui-kit/src/imui/image_item_controls/tests.rs` now owns the size/opacity/UV
  sanitization test and the image-props fill-box projection test.
- `image_item_controls.rs` is now a 173-line implementation file with only
  `#[cfg(test)] mod tests;` for the local test module; `image_item_controls/tests.rs` is a 35-line
  test owner.
- The source gate now rejects local helper test bodies from returning to `image_item_controls.rs`,
  while checking the split test owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::image_item_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`2 tests run: 2 passed, 733 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI table control test owner split - 2026-05-25

Scope: keep `table_controls.rs` focused on row/cell collection and table API wrappers while moving
local table header text-role, hidden-column, and horizontal-scroll contract tests into a dedicated
test owner.

- `ecosystem/fret-ui-kit/src/imui/table_controls.rs` still owns `ImUiTable`, `ImUiTableRow`,
  row/cell collection, and delegation to `render::render_table(...)` / `render::pack_cell_children(...)`.
- `ecosystem/fret-ui-kit/src/imui/table_controls/tests.rs` now owns the table header label text
  role, sort indicator glyph role, hidden column render/response, and horizontal-scroll wrapper
  tests.
- `table_controls.rs` is now a 137-line implementation file with only `#[cfg(test)] mod tests;`
  for the local test module; `table_controls/tests.rs` is a 154-line test owner.
- The source gate now rejects the local table test harness and table test bodies from returning to
  `table_controls.rs`, while checking the split test owner directly across both existing table
  source-gate sections.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::table_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. `cargo check` and focused nextest both waited on existing cargo file locks,
then completed. The focused `fret-ui-kit` nextest gate reported
`4 tests run: 4 passed, 731 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI combo test owner split - 2026-05-25

Scope: keep `combo_controls.rs` focused on combo trigger behavior, popup open/close orchestration,
response lifecycle projection, and visual composition while moving local trigger a11y-label tests
into a dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs` still owns `combo_with_options(...)`, trigger
  pressable behavior, activation shortcuts, popup menu handoff, response state projection, and
  `combo_trigger_a11y_label(...)`.
- `ecosystem/fret-ui-kit/src/imui/combo_controls/tests.rs` now owns the two
  `combo_trigger_a11y_label(...)` format tests.
- `combo_controls.rs` is now a 213-line implementation file with only `#[cfg(test)] mod tests;`
  for the local test module; `combo_controls/tests.rs` is a 9-line test owner.
- The source gate now rejects the local combo a11y-label test bodies from returning to
  `combo_controls.rs`, while checking the split test owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::combo_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The first focused nextest attempt timed out while another same-worktree
`fret-demo` check and broad `cargo fmt --check` held cargo locks; after those processes ended, the
rerun reported `2 tests run: 2 passed, 733 skipped`. `cargo check` reported only the existing
`fret-ui` warnings for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead
code. `git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI floating-window title-bar test owner split - 2026-05-25

Scope: keep `floating_window_title_bar.rs` focused on title-bar row assembly, drag/collapse/close
interaction delegation, and close-glyph text construction while moving the local close-glyph
text-role test into a dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` still owns
  `floating_window_title_bar_row(...)` and `floating_window_close_glyph_text(...)`.
- 2026-05-27 follow-up: `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar/behavior.rs`
  now owns title-bar double-click collapse signaling, Escape close behavior, and close-button
  activation wiring, while the root title-bar file delegates those behaviors.
- `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar/tests.rs` now owns the close-glyph
  shared chrome-glyph text-role test.
- `floating_window_title_bar.rs` remains the title-bar assembly owner with only `mod behavior;` and
  `#[cfg(test)] mod tests;` child modules; `floating_window_title_bar/tests.rs` remains the local
  close-glyph test owner.
- The source gate now rejects the local close-glyph test body from returning to
  `floating_window_title_bar.rs`, checks the split test owner directly, and rejects inline
  title-bar key/press behavior bodies from returning to the root title-bar file.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::floating_window_title_bar::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`1 test run: 1 passed, 734 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.

## IMUI editor proof collection geometry owner split - 2026-06-02

Scope: keep the canonical editor proof collection surface app-owned while splitting pure
collection geometry, layout, drag-rect, and primary-wheel zoom math out of the large
`collection.rs` render/command owner.

- `apps/fret-examples/src/imui_editor_proof_demo/collection/geometry.rs` now owns collection grid
  fallback constants, layout metrics, drag-rect normalization, rect intersection, local rect
  projection, primary-wheel zoom anchoring, and the focused geometry test floor.
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` keeps collection assets, models,
  render assembly, command package behavior, inline rename, context menu, drag/drop, and drop
  status wiring.
- `tools/gate_imui_workstream_source.py` now rejects the geometry helper bodies and geometry tests
  returning to the parent collection module while source-checking the split geometry owner.
- This remains a demo-local proof-surface refactor: no public `fret-imui`, `fret-ui-kit::imui`,
  `fret-ui-editor`, docking, runner, or diagnostics API changed.

Focused gates:

```text
cargo fmt -p fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples proof_collection --no-fail-fast
cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. `cargo check -p fret-examples` reported only the existing unrelated
`fret-plot` / `fret-chart` dead-code warnings.

## IMUI editor proof collection readout owner split - 2026-06-02

Scope: keep the canonical editor proof collection surface app-owned while splitting pure
collection readout/status string construction out of the large `collection.rs` behavior and render
owner.

- `apps/fret-examples/src/imui_editor_proof_demo/collection/readouts.rs` now owns selection,
  visible-order, active-tile, asset-count, command-package, select-all, inline-rename,
  duplicate, and delete readout/status strings.
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` keeps collection assets, selection
  mutation, keyboard navigation, command behavior, inline rename behavior, context menu behavior,
  drag/drop, and render assembly.
- `tools/gate_imui_workstream_source.py` now requires the `readouts` child module and rejects the
  moved readout/status function bodies returning to the parent collection module.
- This remains a demo-local proof-surface refactor: no public `fret-imui`, `fret-ui-kit::imui`,
  `fret-ui-editor`, docking, runner, or diagnostics API changed.

Focused gates:

```text
cargo fmt -p fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples proof_collection --no-fail-fast
cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The first concurrent nextest attempt timed out while both commands compiled
`fret_examples`; the cargo/rustc child processes were allowed to exit naturally, then the gates were
rerun serially. `proof_collection` reported `26 tests run: 26 passed`; the modularization surface
test reported `1 test run: 1 passed`. `cargo check -p fret-examples` reported only the existing
unrelated `fret-chart` and `fret-plot` dead-code warnings.

## Fresh resume verification for closed 2026-05-25 IMUI slices - 2026-05-25

Scope: after context recovery, re-check the current worktree evidence for the seven new closed
follow-ons named by `GOAL_COMPLETION_AUDIT_2026-05-25.md`: canonical editor workbench route,
Demo/Metrics/Debug route, ListBox container proof, optional plot adapter proof, style/theme preset
picker proof, and table header/body owner splits.

Focused gates:

```text
python -m json.tool docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
cargo fmt -p fret-examples -p fret-demo -p fret-ui-editor -p fret-ui-kit -p fret-imui -p fret-plot -- --check
cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast
cargo check -p fret-demo --bin imui_editor_workbench_demo
git diff --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles table_resizable_column_api_compiles --no-fail-fast
cargo nextest run -p fret-imui list_box_container_stamps_semantics_scroll_and_hosts_selectables table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast
cargo check -p fret-plot --features imui
cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast
cargo check -p fret-ui-editor --features imui
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
cargo nextest run -p fret-examples --test editor_notes_device_shell_surface --no-fail-fast
cargo nextest run -p fret-examples parse_editor_theme_preset_key --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
```

Result: passed locally. `check_workstream_catalog.py` validated 443 dedicated directories and 47
standalone markdown files. `gate_imui_workstream_source.py` reported `[gate] ok`. The workbench
golden-path test rerun reported `2 tests run: 2 passed`; the first attempt had timed out during
initial compilation and left Cargo/rustc child processes, which were observed until they exited
before the successful rerun. `fret-demo` workbench check passed. `fret-ui-kit` table smoke reported
`2 tests run: 2 passed`; the focused `fret-imui` ListBox/table gate reported
`4 tests run: 4 passed`; the optional `fret-plot` adapter gate reported `1 test run: 1 passed`;
the full `fret-ui-editor --features imui` gate reported `189 tests run: 189 passed`; the device
shell surface gate reported `2 tests run: 2 passed`; the theme preset key gate reported
`2 tests run: 2 passed`. `git diff --check` reported no whitespace errors and only the existing
line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`. Cargo warnings remained
limited to existing `fret-ui` check-cfg/dead-code warnings and unrelated `fret-chart` /
`fret-plot` dead-code warnings.

## IMUI label-identity test owner split - 2026-05-25

Scope: keep `label_identity.rs` focused on parsing Dear ImGui-style visible label / stable identity
suffixes while moving local parser tests into a dedicated test owner. This completes the current
root-level IMUI inline-test cleanup pass.

- `ecosystem/fret-ui-kit/src/imui/label_identity.rs` still owns `ImUiLabelParts` and
  `parse_label_identity(...)`.
- `ecosystem/fret-ui-kit/src/imui/label_identity/tests.rs` now owns the six parser contract tests
  for plain labels, `##` hidden suffixes, hidden labels, `###` stable identities, and triple-hash
  precedence.
- `label_identity.rs` is now a 16-line implementation file with only `#[cfg(test)] mod tests;` for
  the local test module; `label_identity/tests.rs` is a 37-line test owner.
- The source gate now rejects parser test bodies from returning to `label_identity.rs`, while
  checking the split test owner directly.
- A root-file scan now finds no remaining `#[test]` bodies under
  `ecosystem/fret-ui-kit/src/imui/*.rs`; future IMUI root-file owner work can assume local tests
  live in sibling `tests.rs` owners.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::label_identity::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`6 tests run: 6 passed, 729 skipped` after waiting on existing package-cache locks. `cargo check`
reported only the existing `fret-ui` warnings for `unstable-retained-bridge` check-cfg and
`current_effective_opacity` dead code. `git diff --check` reported only the pre-existing
line-ending warnings for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## IMUI bullet-text test owner split - 2026-05-25

Scope: keep `bullet_text_controls.rs` focused on bullet row construction, indicator/test-id
projection, and compact paragraph text assembly while moving the local text-role contract test into
a dedicated test owner.

- `ecosystem/fret-ui-kit/src/imui/bullet_text_controls.rs` still owns
  `bullet_text_with_options(...)` and `bullet_text_element(...)`.
- `ecosystem/fret-ui-kit/src/imui/bullet_text_controls/tests.rs` now owns the shared compact
  paragraph role test for bullet labels.
- `bullet_text_controls.rs` is now a 73-line implementation file with only
  `#[cfg(test)] mod tests;` for the local test module; `bullet_text_controls/tests.rs` is a 55-line
  test owner.
- The source gate now rejects the local bullet label text-role test body from returning to
  `bullet_text_controls.rs`, while checking the split test owner directly.

Focused gates:

```text
cargo fmt -p fret-ui-kit -p fret-imui -- --check
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui imui::bullet_text_controls::tests --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Result: passed locally. The focused `fret-ui-kit` nextest gate reported
`1 test run: 1 passed, 734 skipped`. `cargo check` reported only the existing `fret-ui` warnings
for `unstable-retained-bridge` check-cfg and `current_effective_opacity` dead code.
`git diff --check` reported only the pre-existing line-ending warnings for `Cargo.lock` and
`apps/fret-examples/src/lib.rs`.
