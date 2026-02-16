pub(crate) const DOC_INTRO: &str = r#"
## Goals

This is an **editor-grade UI** gallery app used to:

- Validate that `fret-ui-shadcn` / `fret-ui-kit` / ecosystem components work under real composition.
- Provide a component-doc-site browsing experience (left navigation, right preview + docs).

Phase 1 intentionally uses hardcoded doc strings to validate the interaction path end-to-end.
"#;

pub(crate) const USAGE_INTRO: &str = r#"
```rust
// Native
cargo run -p fret-ui-gallery

// Web (dedicated harness)
cd apps/fret-ui-gallery-web
trunk serve --open
// open: http://127.0.0.1:8080/?page=button

// Web (via fret-demo-web host)
cd apps/fret-demo-web
trunk serve --open
// open: http://127.0.0.1:8080/?demo=ui_gallery&page=button
```
"#;

pub(crate) const DOC_LAYOUT: &str = r#"
## LayoutRefinement + stack

The gallery shell is a common editor-like layout:

- Fixed-width left navigation (scrollable)
- Right content area (scrollable)

In Fret, this is typically expressed with:

- `LayoutRefinement`: width/height/min/max/fill constraints
- `stack::{hstack,vstack}`: row/column composition & alignment
- `Theme` tokens: design system values like spacing/color/radius
"#;

pub(crate) const USAGE_LAYOUT: &str = r#"
```rust
let root = stack::hstack(
    cx,
    stack::HStackProps::default()
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_stretch(),
    |_cx| vec![sidebar, content],
);
```
"#;

pub(crate) const DOC_MOTION_PRESETS: &str = r#"
## Motion presets (theme token overrides)

Fret’s motion system is **token-driven**: durations/easings/spring params are read from the global
`Theme` using string keys.

This page provides a small set of “effect presets” by applying a `ThemeConfig` patch on top of the
active theme preset:

- shadcn-scoped keys (`duration.shadcn.motion.*`, `easing.shadcn.motion.*`)
- canonical cross-ecosystem keys (`duration.motion.*`, `easing.motion.*`, `number.motion.spring.*`)
- stack reflow tokens (Sonner-style toasts): `duration.motion.stack.shift`, `duration.motion.stack.shift.stagger`

Notes:

- shadcn recipes typically prefer `*.shadcn.motion.*` keys first; canonical keys act as a fallback.
- The `Reduced motion (0)` preset forces token durations to zero for quick UX comparisons.
- Diagnostics gates should use `--fixed-frame-delta-ms 16` for deterministic motion.
"#;

pub(crate) const USAGE_MOTION_PRESETS: &str = r#"
```rust
use fret_ui::{Theme, ThemeConfig};

Theme::with_global_mut(app, |theme| {
    theme.apply_config_patch(&ThemeConfig {
        durations_ms: std::collections::HashMap::from([(
            "duration.motion.presence.enter".to_string(),
            160,
        )]),
        ..ThemeConfig::default()
    });
});
```
"#;

pub(crate) const DOC_MAGIC_MARQUEE: &str = r#"
## Marquee (fret-ui-magic)

This page is an early **creative authoring** demo:

- Drives time via the runner-owned frame clock (ADR 0240).
- Requests continuous frames while animating (no hidden timers).
- Respects reduced-motion preferences (deterministic fallback).

Phase 0 intentionally keeps the API small and explicit: `wrap_width` is provided by the author
instead of relying on dynamic measurement.
"#;

pub(crate) const USAGE_MAGIC_MARQUEE: &str = r#"
```rust
use fret_ui_magic as magic;

let props = magic::MarqueeProps {
    wrap_width: Px(1200.0),
    speed_px_per_s: 80.0,
    ..Default::default()
};

let marquee = magic::marquee(cx, props, |cx| vec![/* repeated content */]);
```
"#;

pub(crate) const DOC_MAGIC_LENS: &str = r#"
## Lens (fret-ui-magic)

This is a Phase 0 “creative parity” demo inspired by MagicUI:

- Uses `MaskLayer` (ADR 0239) to alpha-mask a zoomed copy of the subtree.
- Uses `VisualTransform` to scale about the current pointer position.
- Phase 0 keeps the implementation simple by **duplicating the subtree** (base + zoomed copy).

Non-goals (v1):

- Perfect React/Motion animation parity.
- Supporting interactive children without duplicated state.
"#;

pub(crate) const USAGE_MAGIC_LENS: &str = r#"
```rust
use fret_ui_magic as magic;

let lens = magic::lens(cx, magic::LensProps::default(), |cx| {
    vec![cx.text(\"...\")]
});
```
"#;

pub(crate) const DOC_MAGIC_BORDER_BEAM: &str = r#"
## BorderBeam (fret-ui-magic)

This is a Phase 0 “creative parity” demo inspired by MagicUI:

- Animates a moving highlight around the border using a runner-owned frame clock (ADR 0240).
- Renders glow via `GaussianBlur` + additive compositing groups (ADR 0247).
- Uses `Paint::RadialGradient` for Phase 0; a future revision may switch to a Tier B material kind.

Non-goals (v1):

- Pixel-perfect CSS conic-gradient parity.
- Exposing custom shader code at the component layer.
"#;

pub(crate) const USAGE_MAGIC_BORDER_BEAM: &str = r#"
```rust
use fret_ui_magic as magic;

let card = magic::border_beam(cx, magic::BorderBeamProps::default(), |cx| {
    vec![cx.text(\"...\")]
});
```
"#;

pub(crate) const DOC_MAGIC_DOCK: &str = r#"
## Dock (fret-ui-magic)

This is a Phase 0 “creative parity” demo inspired by MagicUI / macOS dock-style affordances:

- Items are magnified based on pointer proximity.
- The implementation uses a pointer region + a small model to store the last pointer position.
- Hover is used to gate magnification without needing pointer-leave callbacks.

Non-goals (v1):

- Pixel-perfect spring smoothing and bounce physics (can be layered on later).
- A full “roving focus + typeahead” dock interaction policy.
"#;

pub(crate) const USAGE_MAGIC_DOCK: &str = r#"
```rust
use fret_ui_magic as magic;

let dock = magic::dock(cx, magic::DockProps::default(), |cx| {
    vec![
        cx.text(\"A\"),
        cx.text(\"B\"),
        cx.text(\"C\"),
    ]
});
```
"#;

pub(crate) const DOC_MAGIC_PATTERNS: &str = r#"
## Patterns (fret-ui-magic)

This page demonstrates a small set of “creative parity” pattern backgrounds built on Tier B
procedural materials (ADR 0235).

- Patterns are expressed as `Paint::Material { id, params }` on `ContainerProps.background_paint`.
- `MaterialId`s are renderer-controlled and registered via an app-owned `VisualCatalog` (ADR 0245).
- Determinism is explicit: authors provide `seed` (and optionally `phase`), and no hidden time is used.

Non-goals (v1):

- Pixel-perfect CSS/SVG pattern parity.
- Arbitrary shader authoring at the component layer.
"#;

pub(crate) const USAGE_MAGIC_PATTERNS: &str = r#"
```rust
use fret_ui_magic as magic;

let dot = magic::dot_pattern(cx, magic::DotPatternProps::default(), |cx| {
    vec![cx.text(\"DotGrid\")]
});
```
"#;

pub(crate) const DOC_MAGIC_SPARKLES_TEXT: &str = r#"
## SparklesText (fret-ui-magic)

This is a Phase 0 “SparklesText-like” wrapper inspired by MagicUI.

The v1 implementation:

- draws a deterministic sparkle field procedural material (Tier B, ADR 0235),
- animates using the runner-owned frame clock (ADR 0240) when motion is allowed,
- respects `prefers-reduced-motion` (static sparkle field),
- uses additive compositing (ADR 0247) to layer sparkles over the child content.

Non-goals (v1):

- Clipping sparkles to glyph alpha (requires a richer alpha mask substrate than v1 gradient masks).
"#;

pub(crate) const USAGE_MAGIC_SPARKLES_TEXT: &str = r#"
```rust
use fret_ui_magic as magic;

let sparkle = magic::sparkles_text(cx, magic::SparklesTextProps::default(), |cx| {
    vec![cx.text(\"Sparkles\")]
});
```
"#;

pub(crate) const DOC_MAGIC_BLOOM: &str = r#"
## Bloom (fret-ui-kit recipe)

This demo is a “bloom-like” example built from:

- `EffectStep::ColorMatrix` (luma-to-alpha),
- `EffectStep::AlphaThreshold`,
- `EffectStep::GaussianBlur`,
- additive compositing groups (ADR 0247).

It is meant as a Tier B authoring example (ADR 0236) rather than a pixel-perfect reproduction of any
single web implementation.
"#;

pub(crate) const USAGE_MAGIC_BLOOM: &str = r#"
```rust
use fret_ui_kit::declarative::bloom::{BloomPanelProps, bloom_panel};

let panel = bloom_panel(cx, BloomPanelProps::default(), |cx| {
    vec![cx.text(\"...\")]
});
```
"#;

pub(crate) const DOC_MAGIC_CARD: &str = r#"
## MagicCard (fret-ui-magic)

This is a Phase 0 “creative parity” demo inspired by MagicUI:

- Pointer-follow radial gradient chrome (background + border).
- Built on the declarative `ContainerProps` paint surface (ADR 0233) using `Paint::RadialGradient`.
- Uses `PointerRegion` move hooks for pointer tracking (ADR 0238).

Non-goals (v1):

- Perfect visual parity with CSS/WebKit gradients.
- Complex blending/mask semantics beyond the current kernel contracts.
"#;

pub(crate) const USAGE_MAGIC_CARD: &str = r#"
```rust
use fret_ui_magic as magic;

let card = magic::magic_card(cx, magic::MagicCardProps::default(), |cx| {
    vec![cx.text(\"...\")]
});
```
"#;

pub(crate) const DOC_VIEW_CACHE: &str = r#"
## View Cache (experimental)

This page is a **stress + acceptance** harness for GPUI-style cached subtree execution.

When view-cache mode is enabled, a `ViewCache` wrapper can become a cache boundary:

- model/global invalidations stop at the nearest cache root,
- paint-cache is only allowed for cache roots (so they can range-replay),
- on cache-hit, the runtime may skip executing the child render closure and reuse the previous retained subtree.

The goal is "editor-grade smoothness" with fewer full-tree re-renders, while keeping correctness (state + invalidation).
"#;

pub(crate) const USAGE_VIEW_CACHE: &str = r#"
```rust
let subtree = cx.view_cache(ViewCacheProps::default(), |cx| {
    // expensive subtree build here
    vec![cx.text("...")]
});
```
"#;

pub(crate) const DOC_HIT_TEST_TORTURE: &str = r#"
## Hit Test (torture harness)

This page exists to stress the runtime's pointer hit-testing path under editor-grade UI workloads:

- many hit-testable regions (pointer listeners),
- stable layout (no relayout on pointer move),
- pointer moves that change the hovered target every frame (to defeat path caching),
- ability to A/B test spatial indices (e.g. bounds tree) against the fallback traversal.

The goal is to make `top_hit_test_time_us` large enough to be a meaningful slice of the frame budget so
we can validate improvements and gate regressions.
"#;

pub(crate) const USAGE_HIT_TEST_TORTURE: &str = r#"
```rust
// Customize the stress level.
// Defaults are chosen to create a large number of PointerRegion interaction records.
//
// - stripes: hit target changes frequently (defeats hit-test path cache).
// - noise: many tiny pointer regions that should never be hit, but are expensive for fallback scans.
std::env::set_var("FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES", "256");
std::env::set_var("FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE", "50000");
```
"#;

pub(crate) const DOC_HIT_TEST_ONLY_PAINT_CACHE_PROBE: &str = r#"
## Hit-test-only paint-cache probe

This page is a focused probe for the `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` experiment:

- pointer moves intentionally trigger `Invalidation::HitTestOnly` on a cache-eligible subtree,
- layout stays stable,
- paint output stays stable.

Goal: make the new diagnostics counters non-zero in a deterministic script so A/B results are
causally attributable to the gate path.
"#;

pub(crate) const USAGE_HIT_TEST_ONLY_PAINT_CACHE_PROBE: &str = r#"
```rust
// Recommended run flags for this probe:
// - start directly on the probe page
// - disable gallery view-cache wrappers to keep paint-cache gating simple
std::env::set_var("FRET_UI_GALLERY_START_PAGE", "hit_test_only_paint_cache_probe");
std::env::set_var("FRET_UI_GALLERY_VIEW_CACHE", "0");
std::env::set_var("FRET_UI_GALLERY_VIEW_CACHE_SHELL", "0");
```
"#;

pub(crate) const DOC_VIRTUAL_LIST_TORTURE: &str = r#"
## Virtual List (torture harness)

This page exists to validate "editor-grade scrolling feel" under realistic composition:

- 10k+ virtualized rows
- row focus + selection-like interactions
- scroll-to-item correctness (anchor preservation + measured heights)
- a small inline text input inside the list

The goal is not to ship a component; it is to provide a deterministic surface for performance
instrumentation and regression scripts (GPUI parity workstream).
"#;

pub(crate) const USAGE_VIRTUAL_LIST_TORTURE: &str = r#"
```rust
let handle = VirtualListScrollHandle::new();

let list = cx.virtual_list_keyed(
    len,
    VirtualListOptions::new(Px(28.0), 8),
    &handle,
    |i| i as ItemKey,
    |cx, i| row(cx, i),
);
```
"#;

pub(crate) const DOC_UI_KIT_LIST_TORTURE: &str = r#"
## List (UI kit torture harness)

This page is an ecosystem-level harness for `fret-ui-kit::declarative::list`.

It intentionally uses the **retained-host** VirtualList path (ADR 0177) to validate that:

- scroll can update membership under cache-hit reuse,
- crossing window boundaries does not require dirtying the parent cache-root,
- correctness remains stale-paint safe under view-cache + shell reuse.
"#;

pub(crate) const USAGE_UI_KIT_LIST_TORTURE: &str = r#"
Run with view-cache enabled:

- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`

Script:

- `tools/diag-scripts/ui-gallery-ui-kit-list-window-boundary-scroll.json`
"#;

pub(crate) const DOC_CODE_VIEW_TORTURE: &str = r#"
## Code View (torture harness)

This page is a stress surface for **large scrollable text/code documents**.

It is intended to back the GPUI parity workstream:

- validate scroll stability (no “stale paint” / “UI looks not refreshed” regressions)
- identify when code/text surfaces should become **prepaint-windowed** (ADR 0175)
- provide a deterministic bundle capture target for perf investigations
"#;

pub(crate) const USAGE_CODE_VIEW_TORTURE: &str = r#"
```rust
use fret_code_view::CodeBlock;

let code = Arc::<str>::from("...");
let block = CodeBlock::new(code).language("rust").show_line_numbers(true);
```
"#;

pub(crate) const DOC_CODE_EDITOR_MVP: &str = r#"
## Code Editor (MVP)

This page hosts a v1 MVP for a **paint-driven, windowed code editor surface**:

- Owns its buffer + selection state (ecosystem crate, not `fret-ui`).
- Uses a `TextInputRegion` seam to receive `TextInput` / `Ime` events while it draws its own text.
- Focuses on validating the input/IME contract and scroll stability before performance work.
"#;

pub(crate) const USAGE_CODE_EDITOR_MVP: &str = r#"
```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle};

let handle = CodeEditorHandle::new("fn main() {}\n");
let editor = CodeEditor::new(handle).into_element(cx);
```
"#;

pub(crate) const DOC_CODE_EDITOR_TORTURE: &str = r#"
## Code Editor (torture harness)

This page is a stress surface for the **windowed, paint-driven code editor**.

Goals:

- validate scroll stability (no “stale paint” / “looks not refreshed” regressions),
- validate text blob caching stays bounded to the visible window,
- provide a deterministic target for perf investigations.
"#;

pub(crate) const USAGE_CODE_EDITOR_TORTURE: &str = r#"
```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle, CodeEditorTorture};
use fret_core::Px;

let handle = CodeEditorHandle::new("...\n");
let editor = CodeEditor::new(handle)
    .overscan(128)
    .torture(CodeEditorTorture::auto_scroll_bounce(Px(8.0)))
    .into_element(cx);
```
"#;

pub(crate) const DOC_MARKDOWN_EDITOR_SOURCE: &str = r#"
## Markdown editor (source mode)

This page is a v0 **source-mode** Markdown editor milestone:

- edit Markdown as plain text (no WYSIWYG),
- validate `fret-code-editor` interaction control (edit vs read-only),
- validate Markdown syntax highlighting (best-effort) and wrap stability,
- optionally validate a live preview rendered by `fret-markdown`.
"#;

pub(crate) const USAGE_MARKDOWN_EDITOR_SOURCE: &str = r##"
```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle};

let handle = CodeEditorHandle::new("# Hello\n\n- Item\n");
let editor = CodeEditor::new(handle).into_element(cx);
```
"##;

pub(crate) const DOC_TEXT_SELECTION_PERF: &str = r#"
## Text selection (perf diagnostics)

This page is a small diagnostic harness for **large selection highlight** behavior:

- A long multi-line text blob is selected end-to-end.
- The paint path queries selection rects **clipped to the current viewport**.

The goal is to track the number of generated selection rectangles and ensure it scales with
visible lines (viewport height), not total document length.
"#;

pub(crate) const USAGE_TEXT_SELECTION_PERF: &str = r#"
```rust
// Scroll with the mouse wheel over the demo surface.
// The overlay shows the current clipped selection-rect count.
```
"#;

pub(crate) const DOC_TEXT_BIDI_RTL_CONFORMANCE: &str = r#"
## Text BiDi / RTL (conformance harness)

This page exists to sanity-check text geometry queries under BiDi/RTL strings:

- `TextService::hit_test_point` (mouse → caret)
- `TextService::caret_rect` (caret → rect)
- `TextService::selection_rects*` (selection highlight)

It is intentionally a **diagnostic surface**, not a component demo.
"#;

pub(crate) const USAGE_TEXT_BIDI_RTL_CONFORMANCE: &str = r#"
```rust
// Click or drag in the diagnostic panel:
// - normal click: set caret + anchor (collapsed selection)
// - shift-click / drag: extend selection from the current anchor
//
// Try the selectable samples above to validate editor-like text selection on BiDi strings.
```
"#;

pub(crate) const DOC_TEXT_MIXED_SCRIPT_FALLBACK: &str = r#"
## Text mixed-script fallback (bundled fonts)

This page is a small correctness harness for **mixed-script fallback**:

- Latin (UI font)
- CJK (common fallback)
- Emoji (color font)

It is designed to remain meaningful even when system fonts are disabled (see `FRET_TEXT_SYSTEM_FONTS=0`),
by explicitly injecting the bundled font set on first render.
"#;

pub(crate) const USAGE_TEXT_MIXED_SCRIPT_FALLBACK: &str = r#"
```rust
// Expectation:
// - missing/tofu glyphs should remain 0 for the provided sample strings
// - a font trace should exist if missing glyphs ever regress
```
"#;

pub(crate) const DOC_TEXT_MEASURE_OVERLAY: &str = r#"
## Text measurement overlay (diagnostic)

This page visualizes text measurement vs the allocated layout box:

- the **container bounds** (what layout assigned),
- and the **measured text box** (what `TextMetrics` reports for the same constraints).

It helps debug mismatches where text appears to overflow its chrome/background or where
layout sizing and paint sizing disagree.
"#;

pub(crate) const USAGE_TEXT_MEASURE_OVERLAY: &str = r#"
```rust
// The panel draws:
// - green border: container bounds
// - yellow border: measured bounds (TextMetrics.size)
// - cyan line: baseline
```
"#;

pub(crate) const DOC_TEXT_FEATURE_TOGGLES: &str = r#"
## Text OpenType feature toggles (diagnostic)

This page is a small harness for the `TextShapingStyle.features` surface:

- toggles `liga` / `calt` / one `ssXX` (`ss01`) via `TextFontFeatureSetting`,
- shapes the same sample string twice:
  - baseline (no explicit features),
  - explicit feature overrides.

Notes:

- This is a **best-effort** surface: if a resolved font face does not support a tag, it is ignored.
- Visible differences are font-dependent. Inter typically demonstrates `liga` (fi/fl/ffi/ffl).
"#;

pub(crate) const USAGE_TEXT_FEATURE_TOGGLES: &str = r#"
```rust
use fret_core::{TextShapingStyle, TextSpan};

let shaping = TextShapingStyle::default()
    .with_feature("liga", 0)
    .with_feature("calt", 0)
    .with_feature("ss01", 1);

let span = TextSpan {
    len: text.len(),
    shaping,
    ..Default::default()
};
```
"#;

pub(crate) const DOC_WEB_IME_HARNESS: &str = r#"
## Web IME (harness)

This page exists to validate the wasm IME bridge contract (ADR 0180):

- a hidden textarea is used as the browser-owned IME target,
- `composition*` drives `Event::Ime::{Preedit,Commit}`,
- committed insertions arrive as `Event::TextInput` (no control characters),
- and we avoid **double-insert** on `compositionend` + `input`.

Try:

- CJK IME composition (preedit updates, commit),
- emoji input,
- backspace/arrows while composing,
- and verify the committed buffer does not duplicate inserts.
"#;

pub(crate) const USAGE_WEB_IME_HARNESS: &str = r#"
```rust
// Click the region to focus it. On wasm, it should focus a hidden textarea via `Effect::ImeAllow`.
// Use an IME to ensure `Event::Ime` and `Event::TextInput` are routed correctly.
// Optional: add `?ime_debug=1` to the URL (or set `window.__FRET_IME_DEBUG=true`) to log bridge
// focus/cursor-area updates to the browser console.
```
"#;

pub(crate) const DOC_CHART_TORTURE: &str = r#"
## Chart (torture harness)

This page is a stress surface for **canvas-driven charts** with pan/zoom interactions.

It exists to support the GPUI parity workstream:

- validate “no stale scene” behavior under view-cache reuse,
- identify where charts/plots should adopt prepaint-windowed sampling (ADR 0175),
- provide a deterministic bundle capture target for perf investigations.
"#;

pub(crate) const USAGE_CHART_TORTURE: &str = r#"
```rust
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

let props = ChartCanvasPanelProps::new(spec);
let el = chart_canvas_panel(cx, props);
```
"#;

pub(crate) const DOC_CANVAS_CULL_TORTURE: &str = r#"
## Canvas Cull (torture harness)

This page is a stress surface for **pan/zoom canvas scenes** with viewport-driven culling.

It exists to support the GPUI parity workstream:

- validate “no stale scene” behavior under view-cache reuse,
- identify when large canvas/node-graph surfaces should become **prepaint-windowed** (ADR 0175),
- provide a deterministic bundle capture target for perf investigations.
"#;

pub(crate) const USAGE_CANVAS_CULL_TORTURE: &str = r#"
```rust
use fret_canvas::ui::{PanZoomCanvasSurfacePanelProps, pan_zoom_canvas_surface_panel};

let props = PanZoomCanvasSurfacePanelProps::default();
let el = pan_zoom_canvas_surface_panel(cx, props, |_painter, _cx| {});
```
"#;

pub(crate) const DOC_NODE_GRAPH_CULL_TORTURE: &str = r#"
## Node Graph Cull (torture harness)

This page hosts a large `fret-node` canvas surface (nodes + edges) intended to stress:

- viewport-driven culling,
- pan/zoom interaction routing,
- paint-cache reuse under view-cache + shell.

It exists to support the GPUI parity workstream:

- promote a real ecosystem surface into the prepaint-windowed migration pipeline (ADR 0175),
- validate “paint-only” interaction updates for small deltas,
- provide deterministic script targets for perf investigations.
"#;

pub(crate) const USAGE_NODE_GRAPH_CULL_TORTURE: &str = r#"
```rust
use fret_node::{Graph, GraphId};
use fret_node::io::NodeGraphViewState;
use fret_node::ui::NodeGraphCanvas;
use fret_ui::retained_bridge::{RetainedSubtreeProps, UiTreeRetainedExt};

let graph = models.insert(Graph::new(GraphId::from_u128(1)));
let view = models.insert(NodeGraphViewState::default());

let el = cx.retained_subtree(RetainedSubtreeProps::new(move |ui| {
    ui.create_node_retained(NodeGraphCanvas::new(graph.clone(), view.clone()))
}));
```
"#;

pub(crate) const DOC_CHROME_TORTURE: &str = r#"
## Chrome (torture harness)

This page is a stress surface for interaction-driven “chrome”:

- hover/pressed/focus rings,
- caret/selection visuals,
- overlay open/close loops.

It exists to support the GPUI parity workstream:

- validate that “hover-only” and “focus-only” ticks can be paint-only under view-cache reuse,
- catch stale-paint regressions where semantics/hit-testing updates but the scene does not.
"#;

pub(crate) const USAGE_CHROME_TORTURE: &str = r#"
This page is intentionally policy-heavy and should be driven via diagnostics scripts.
"#;

pub(crate) const DOC_WINDOWED_ROWS_SURFACE_TORTURE: &str = r#"
## Windowed Rows Surface (torture harness)

This page is a baseline for **scroll-driven windowing without per-row declarative subtrees**.

It uses a structurally stable element tree (a `Scroll` + a leaf `Canvas`) and paints only the
visible row window inside the canvas paint handler.

This is intended to validate:

- scroll stability under view-cache reuse (no stale paint),
- near-minimal CPU work for scroll-only deltas,
- a reusable pattern for huge “list-like” surfaces that do not need per-row semantics/focus.
"#;

pub(crate) const USAGE_WINDOWED_ROWS_SURFACE_TORTURE: &str = r#"
```rust
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfaceProps, windowed_rows_surface,
};

let el = windowed_rows_surface(cx, WindowedRowsSurfaceProps::default(), |_p, _i, _rect| {});
```
"#;

pub(crate) const DOC_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str = r#"
## Windowed Rows Surface (interactive harness)

This page demonstrates a “windowed surface” pattern (ADR 0175) with **paint-only hover chrome**
(ADR 0166) using a stable element tree:

- `Scroll` (retained scroll state + transform)
- `PointerRegion` (row hit-testing in event hooks)
- `Canvas` (paint only the visible row window)

The goal is to show that pointer-driven chrome (hover highlight) can update via paint invalidation
without forcing rerender or relayout.
"#;

pub(crate) const USAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str = r#"
```rust
use fret_ui::element::PointerRegionProps;
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
    windowed_rows_surface_with_pointer_region,
};

let props = WindowedRowsSurfaceProps::default();
let pointer = PointerRegionProps::default();
let handlers = WindowedRowsSurfacePointerHandlers::default();

let el = windowed_rows_surface_with_pointer_region(
    cx,
    props,
    pointer,
    handlers,
    None,
    |_p, _i, _rect| {},
);
```
"#;

pub(crate) const DOC_DATA_TABLE_TORTURE: &str = r#"
## DataTable (torture harness)

This page is a baseline for **virtualized business tables** built with:

- `fret-ui-headless` TanStack-aligned table engine,
- `fret-ui-kit` virtualization helpers,
- `fret-ui-shadcn` `DataTable` integration surface.

This harness exists to make performance regressions measurable and reproducible when refactoring
VirtualList windowing, row measurement modes, and cache-root placement.
"#;

pub(crate) const USAGE_DATA_TABLE_TORTURE: &str = r#"
```rust
use fret_ui_shadcn::DataTable;

let table = DataTable::new();
```
"#;

pub(crate) const DOC_TREE_TORTURE: &str = r#"
## Tree (torture harness)

This page is a baseline for **virtualized trees** built with `fret-ui-kit::declarative::tree_view`.

It exists to validate:

- scroll stability under view-cache reuse (no stale paint),
- row-window correctness (expand/collapse does not detach state on cache hits),
- future migrations toward prepaint-driven windowing (ADR 0175).
"#;

pub(crate) const USAGE_TREE_TORTURE: &str = r#"
```rust
use fret_ui_kit::declarative::tree::tree_view;
```
"#;

pub(crate) const DOC_TABLE_RETAINED_TORTURE: &str = r#"
## Table (retained torture harness)

This page is a baseline for the **UI Kit table surface** running on the virt-003 retained host path (ADR 0177).

It exists to validate:

- overscan window boundary updates reconcile attach/detach deltas (without notify-based dirty views),
- header sorting + row selection remain correct under cache-root reuse,
- scripted regressions stay stable as we migrate more of the full table surface into retained hosts (GPUI-MVP5-eco-002).
"#;

pub(crate) const USAGE_TABLE_RETAINED_TORTURE: &str = r#"
```rust
use fret_ui_kit::declarative::table::table_virtualized_retained_v0;
```
"#;

pub(crate) const DOC_AI_TRANSCRIPT_TORTURE: &str = r#"
## AI transcript (torture harness)

This page is a baseline for **long scrolling conversations** (chat transcripts).

It exists to validate:

- scroll stability under view-cache reuse (no stale paint),
- virtualization correctness under composable message rows (virt-003 retained hosts; ADR 0177),
- future migrations toward prepaint-windowed/ephemeral updates (ADR 0160/0178).
"#;

pub(crate) const USAGE_AI_TRANSCRIPT_TORTURE: &str = r#"
```rust
use fret_ui_ai::{ConversationMessage, ConversationTranscript, MessageRole};

let transcript = ConversationTranscript::new(vec![
    ConversationMessage::new(1, MessageRole::User, "Hello"),
    ConversationMessage::new(2, MessageRole::Assistant, "Hi!"),
]);
```
"#;

pub(crate) const DOC_AI_CHAT_DEMO: &str = r#"
## AI chat (demo)

This page is a small, interactive demo for the `fret-ui-ai` chat surfaces:

- `ConversationTranscript` for a short transcript,
- `PromptInput` for composing + sending messages,
- stable `test_id` anchors for automation.

It exists to validate:

- prompt input ergonomics (send/stop/disabled/loading),
- transcript append behavior + stick-to-bottom eligibility,
- a keyboard-first automation path via `fretboard diag`.
"#;

pub(crate) const USAGE_AI_CHAT_DEMO: &str = r#"
```rust
use fret_ui_ai::{ConversationTranscript, PromptInput};
```
"#;

pub(crate) const DOC_AI_CONVERSATION_DEMO: &str = r#"
## AI conversation (demo)

This page demonstrates AI Elements-aligned conversation transcript surfaces:

- `ConversationTranscript` (text-only harness),
- `AiConversationTranscript` (rich parts via `MessageParts`).

It exists to validate stick-to-bottom behavior, virtualization stability, and stable `test_id`
anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_CONVERSATION_DEMO: &str = r#"
```rust
use fret_ui_ai::{AiConversationTranscript, ConversationMessage, ConversationTranscript, MessageRole};
```
"#;

pub(crate) const DOC_AI_MESSAGE_DEMO: &str = r#"
## AI message (demo)

This page demonstrates AI Elements-aligned message building blocks:

- `Message` (role-aware alignment wrapper),
- `MessageContent` (assistant flow vs user bubble),
- `MessageActions` / `MessageAction` (icon actions row).
"#;

pub(crate) const USAGE_AI_MESSAGE_DEMO: &str = r#"
```rust
use fret_ui_ai::{Message, MessageAction, MessageActions, MessageContent, MessageRole};
```
"#;

pub(crate) const DOC_AI_CONTEXT_DEMO: &str = r#"
## AI context (demo)

This page demonstrates the AI Elements-aligned `Context` hovercard surface:

- percent trigger,
- progress + compact token counts in the content.

The data model is app-owned; this surface is presentation-only.
"#;

pub(crate) const USAGE_AI_CONTEXT_DEMO: &str = r#"
```rust
use fret_ui_ai::Context;
```
"#;

pub(crate) const DOC_AI_TERMINAL_DEMO: &str = r#"
## AI terminal (demo)

This page demonstrates the AI Elements-aligned `Terminal` viewer surface:

- output text content (monospace),
- copy/clear actions,
- auto-scroll to bottom on output changes (when enabled).

Note: v1 is **viewer-only** and does not embed a PTY/TTY terminal.
"#;

pub(crate) const USAGE_AI_TERMINAL_DEMO: &str = r#"
```rust
use fret_ui_ai::Terminal;
```
"#;

pub(crate) const DOC_AI_PACKAGE_INFO_DEMO: &str = r#"
## AI package info (demo)

This page demonstrates the AI Elements-aligned `PackageInfo` surface:

- name + change type badge,
- current/new version display,
- dependency list building blocks (`PackageInfoDependencies` / `PackageInfoDependency`).

The data model is app-owned; this surface is presentation-only.
"#;

pub(crate) const USAGE_AI_PACKAGE_INFO_DEMO: &str = r#"
```rust
use fret_ui_ai::{PackageInfo, PackageInfoChangeKind};
```
"#;

pub(crate) const DOC_AI_OPEN_IN_CHAT_DEMO: &str = r#"
## AI open in chat (demo)

This page demonstrates the AI Elements-aligned `OpenIn` menu surface:

- trigger button,
- provider entries that emit `Effect::OpenUrl` when selected.

Note: the demo gate opens the menu but does not click a provider entry (to avoid launching a browser).
"#;

pub(crate) const USAGE_AI_OPEN_IN_CHAT_DEMO: &str = r#"
```rust
use fret_ui_ai::OpenIn;
```
"#;

pub(crate) const DOC_AI_TASK_DEMO: &str = r#"
## AI task (demo)

This page demonstrates the AI Elements-aligned `Task` collapsible surface:

- a trigger row with title + chevron,
- an indented content region with a muted left border,
- lightweight `TaskItem` / `TaskItemFile` building blocks.
"#;

pub(crate) const USAGE_AI_TASK_DEMO: &str = r#"
```rust
use fret_ui_ai::{Task, TaskContent, TaskItem, TaskItemFile, TaskTrigger};
```
"#;

pub(crate) const DOC_AI_AUDIO_PLAYER_DEMO: &str = r#"
## AI audio player (demo)

This page demonstrates an AI Elements-aligned `AudioPlayer` chrome surface:

- outline icon controls (play/pause, seek, mute),
- time + duration labels,
- time and volume sliders.

Note: this is a UI-only port. Apps are expected to own actual audio playback and drive the exposed
models (or attach callbacks to mirror changes into a backend).
"#;

pub(crate) const USAGE_AI_AUDIO_PLAYER_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    AudioPlayer, AudioPlayerControlBar, AudioPlayerDurationDisplay, AudioPlayerMuteButton,
    AudioPlayerPlayButton, AudioPlayerSeekBackwardButton, AudioPlayerSeekForwardButton,
    AudioPlayerTimeDisplay, AudioPlayerTimeRange, AudioPlayerVolumeRange,
};
```
"#;

pub(crate) const DOC_AI_TRANSCRIPTION_DEMO: &str = r#"
## AI transcription (demo)

This page demonstrates an AI Elements-aligned `Transcription` surface:

- a flex-wrapping segment row,
- segment styling for past/active/future regions,
- an optional seek interaction seam (`on_seek`) for app-owned playback backends.
"#;

pub(crate) const USAGE_AI_TRANSCRIPTION_DEMO: &str = r#"
```rust
use fret_ui_ai::{Transcription, TranscriptionSegment, TranscriptionSegmentData};
```
"#;

pub(crate) const DOC_AI_SPEECH_INPUT_DEMO: &str = r#"
## AI speech input (demo)

This page demonstrates an AI Elements-aligned `SpeechInput` surface:

- UI-only record/stop chrome (`mic` ↔ `square`),
- a `processing` state (spinner),
- an intent seam (`on_listening_change`) so apps own capture + ASR backends.
"#;

pub(crate) const USAGE_AI_SPEECH_INPUT_DEMO: &str = r#"
```rust
use fret_ui_ai::SpeechInput;
```
"#;

pub(crate) const DOC_AI_MIC_SELECTOR_DEMO: &str = r#"
## AI mic selector (demo)

This page demonstrates an AI Elements-aligned `MicSelector` surface:

- a popover-based selector with search input,
- stable selection model (`value_model`) and popover open model (`open_model`),
- app-owned device enumeration (UI-only chrome).
"#;

pub(crate) const USAGE_AI_MIC_SELECTOR_DEMO: &str = r#"
```rust
use fret_ui_ai::{MicSelector, MicSelectorContent, MicSelectorInput, MicSelectorList, MicSelectorTrigger, MicSelectorValue};
```
"#;

pub(crate) const DOC_AI_VOICE_SELECTOR_DEMO: &str = r#"
## AI voice selector (demo)

This page demonstrates an AI Elements-aligned `VoiceSelector` surface:

- dialog-based selector with search input,
- stable selection model (`value_model`) and dialog open model (`open_model`),
- app-owned voice inventory + preview playback (UI-only chrome).
"#;

pub(crate) const USAGE_AI_VOICE_SELECTOR_DEMO: &str = r#"
```rust
use fret_ui_ai::{VoiceSelector, VoiceSelectorButton, VoiceSelectorContent, VoiceSelectorInput, VoiceSelectorList};
```
"#;

pub(crate) const DOC_AI_AGENT_DEMO: &str = r#"
## AI agent (demo)

This page demonstrates an AI Elements-aligned `Agent` surface:

- header (name + optional model badge),
- instructions card,
- tools accordion (schemas rendered as JSON),
- output schema code block.

Note: this is a UI-only port. Apps own tool execution and schema sources.
"#;

pub(crate) const USAGE_AI_AGENT_DEMO: &str = r#"
```rust
use fret_ui_ai::{Agent, AgentContent, AgentHeader, AgentInstructions, AgentOutput, AgentTools};
```
"#;

pub(crate) const DOC_AI_SANDBOX_DEMO: &str = r#"
## AI sandbox (demo)

This page demonstrates an AI Elements-aligned `Sandbox` surface:

- collapsible root with status badge,
- tabs for switching between panels.

Note: this is a UI-only port. Apps own sandbox execution and panel content.
"#;

pub(crate) const USAGE_AI_SANDBOX_DEMO: &str = r#"
```rust
use fret_ui_ai::{Sandbox, SandboxContent, SandboxHeader, SandboxTabs};
```
"#;

pub(crate) const DOC_AI_PERSONA_DEMO: &str = r#"
## AI persona (demo)

This page demonstrates an AI Elements-aligned `Persona` surface.

Upstream AI Elements uses a Rive (webgl2) animation loaded from a remote `.riv` asset. The Fret
port currently provides a UI-only placeholder that preserves the state/variant taxonomy and
automation anchors.
"#;

pub(crate) const USAGE_AI_PERSONA_DEMO: &str = r#"
```rust
use fret_ui_ai::{Persona, PersonaState, PersonaVariant};
```
"#;

pub(crate) const DOC_AI_WORKFLOW_CHROME_DEMO: &str = r#"
## AI workflow chrome (demo)

This page demonstrates AI Elements-aligned workflow chrome surfaces:

- `WorkflowPanel` (bordered, rounded panel container),
- `WorkflowToolbar` (small tool row container).

Note: upstream uses `@xyflow/react` (React Flow). The Fret port is UI-only chrome intended to wrap
existing ecosystem crates (node graph/canvas/docking) without introducing new engines.
"#;

pub(crate) const USAGE_AI_WORKFLOW_CHROME_DEMO: &str = r#"
```rust
use fret_ui_ai::{WorkflowPanel, WorkflowToolbar};
```
"#;

pub(crate) const DOC_AI_WORKFLOW_CANVAS_DEMO: &str = r#"
## AI workflow canvas (demo)

This page demonstrates the AI Elements-aligned `WorkflowCanvas` host surface (UI-only chrome).
"#;

pub(crate) const USAGE_AI_WORKFLOW_CANVAS_DEMO: &str = r#"
```rust
use fret_ui_ai::WorkflowCanvas;
```
"#;

pub(crate) const DOC_AI_WORKFLOW_NODE_DEMO: &str = r#"
## AI workflow node (demo)

This page demonstrates the AI Elements-aligned `WorkflowNode` chrome (UI-only):

- node header/content/footer composition,
- optional handle indicators.
"#;

pub(crate) const USAGE_AI_WORKFLOW_NODE_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    WorkflowNode, WorkflowNodeContent, WorkflowNodeFooter, WorkflowNodeHandles, WorkflowNodeHeader,
    WorkflowNodeTitle,
};
```
"#;

pub(crate) const DOC_AI_WORKFLOW_EDGE_DEMO: &str = r#"
## AI workflow edge (demo)

This page demonstrates AI Elements-aligned workflow edge renderers (UI-only):

- `WorkflowEdgeTemporary` (dashed),
- `WorkflowEdgeAnimated` (animated stroke + optional arrow marker).
"#;

pub(crate) const USAGE_AI_WORKFLOW_EDGE_DEMO: &str = r#"
```rust
use fret_ui_ai::{WorkflowEdgeAnimated, WorkflowEdgeMarkerEnd, WorkflowEdgeTemporary};
```
"#;

pub(crate) const DOC_AI_WORKFLOW_CONNECTION_DEMO: &str = r#"
## AI workflow connection (demo)

This page demonstrates the AI Elements-aligned `WorkflowConnection` line renderer (UI-only).
"#;

pub(crate) const USAGE_AI_WORKFLOW_CONNECTION_DEMO: &str = r#"
```rust
use fret_ui_ai::WorkflowConnection;
```
"#;

pub(crate) const DOC_AI_WORKFLOW_CONTROLS_DEMO: &str = r#"
## AI workflow controls (demo)

This page demonstrates the AI Elements-aligned `WorkflowControls` chrome (UI-only).
"#;

pub(crate) const USAGE_AI_WORKFLOW_CONTROLS_DEMO: &str = r#"
```rust
use fret_ui_ai::{WorkflowControls, WorkflowControlsButton};
```
"#;

pub(crate) const DOC_AI_WORKFLOW_PANEL_DEMO: &str = r#"
## AI workflow panel (demo)

This page demonstrates the AI Elements-aligned `WorkflowPanel` container chrome (UI-only).
"#;

pub(crate) const USAGE_AI_WORKFLOW_PANEL_DEMO: &str = r#"
```rust
use fret_ui_ai::WorkflowPanel;
```
"#;

pub(crate) const DOC_AI_WORKFLOW_TOOLBAR_DEMO: &str = r#"
## AI workflow toolbar (demo)

This page demonstrates the AI Elements-aligned `WorkflowToolbar` row chrome (UI-only).
"#;

pub(crate) const USAGE_AI_WORKFLOW_TOOLBAR_DEMO: &str = r#"
```rust
use fret_ui_ai::WorkflowToolbar;
```
"#;

pub(crate) const DOC_AI_WORKFLOW_NODE_GRAPH_DEMO: &str = r#"
## AI workflow node graph (demo)

This page demonstrates an engine-backed workflow editor surface:

- `fret-node` provides the graph model + interaction (pan/zoom, selection, connect ports),
- `fret-ui-ai` provides AI Elements-aligned chrome wrappers (`WorkflowPanel`, `WorkflowToolbar`, `WorkflowControls`).

The goal is to validate the recommended layering: keep `fret-ui-ai` policy-light and reuse existing
ecosystem engines for editor-grade interaction.
"#;

pub(crate) const USAGE_AI_WORKFLOW_NODE_GRAPH_DEMO: &str = r#"
```rust
use fret_node::io::NodeGraphViewState;
use fret_node::ui::{NodeGraphCanvas, NodeGraphViewQueue};
use fret_ui_ai::{WorkflowControls, WorkflowControlsButton, WorkflowPanel, WorkflowToolbar};
```
"#;

pub(crate) const DOC_AI_CANVAS_WORLD_LAYER_SPIKE: &str = r#"
## AI canvas world layer (spike)

This page is a **composition spike** for an XYFlow-like mental model:

- a pan/zoom canvas paint pass (background + edges),
- nodes as normal element subtrees positioned in **canvas space**,
- optional screen-space overlays above the world.

It is intentionally **not** a full workflow engine. For editor-grade graph editing, use `fret-node`.
"#;

pub(crate) const USAGE_AI_CANVAS_WORLD_LAYER_SPIKE: &str = r#"
```rust
use fret_canvas::ui::{CanvasWorldSurfacePanelProps, canvas_world_surface_panel};

let world = canvas_world_surface_panel(
    cx,
    CanvasWorldSurfacePanelProps::default(),
    |p, paint_cx| {
        // draw grid/edges using paint_cx.view.render_transform(p.bounds())
        let _ = (p, paint_cx);
    },
    |cx, world_cx| {
        // build nodes as element subtrees in canvas-space coordinates
        let _ = (cx, world_cx);
        Vec::new()
    },
    |cx, world_cx| {
        // build overlays (screen-space)
        let _ = (cx, world_cx);
        Vec::new()
    },
);
```
"#;

pub(crate) const DOC_AI_PROMPT_INPUT_PROVIDER_DEMO: &str = r#"
## AI prompt input provider (demo)

This page demonstrates `PromptInputProvider` + a parts-first `PromptInputRoot` composition:

- provider mode lifts `text` + `attachments` models outside the prompt input surface,
- an external button mutates the attachments model (simulating an app-owned file picker effect),
- `PromptInputRoot::into_element_with_slots` composes block-start and block-end content using parts:
  - `PromptInputHeader` + `PromptInputAttachmentsRow`
  - `PromptInputFooter` + `PromptInputTools` + `PromptInputActionAddAttachmentsButton` + `PromptInputSubmit`

It exists to validate:

- provider-mode integration for apps (models are app-owned),
- parts-first composition seams for prompt chrome,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_PROMPT_INPUT_PROVIDER_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    PromptInputProvider, PromptInputRoot, PromptInputSlots, PromptInputHeader, PromptInputFooter,
    PromptInputTools, PromptInputSubmit, PromptInputActionAddAttachmentsButton,
    PromptInputAttachmentsRow,
};
```
"#;

pub(crate) const DOC_AI_PROMPT_INPUT_ACTION_MENU_DEMO: &str = r#"
## AI prompt input action menu (demo)

This page demonstrates `PromptInputActionMenu` (shadcn `DropdownMenu`) composed into a
parts-first `PromptInputRoot`.

It exists to validate:

- action menu trigger + content wiring in the prompt footer,
- intent-driven menu items (via `OnActivate`, not direct side effects),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_PROMPT_INPUT_ACTION_MENU_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    PromptInputActionAddAttachments, PromptInputActionMenu, PromptInputActionMenuContent,
    PromptInputActionMenuTrigger,
};
```
"#;

pub(crate) const DOC_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO: &str = r#"
## AI prompt input referenced sources (demo)

This page demonstrates a local “referenced sources” model for `PromptInputRoot` (aligned with
AI Elements `ReferencedSourcesContext` in `prompt-input.tsx`):

- referenced sources are **local to the prompt input** (even when attachments are provider-owned),
- the surface is intent-driven (apps decide how sources are discovered/added),
- chips are rendered via `PromptInputReferencedSourcesRow`.

It exists to validate:

- local model ownership and stable keyed identity for chips,
- remove affordances (chip hover + remove button),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO: &str = r#"
```rust
use fret_ui_ai::{PromptInputReferencedSourcesRow, use_prompt_input_referenced_sources};
```
"#;

pub(crate) const DOC_AI_INLINE_CITATION_DEMO: &str = r#"
## AI inline citation (demo)

This page demonstrates the AI Elements-aligned `InlineCitation` surface:

- hover card content,
- pager controls for multiple sources,
- optional “select source” seam (model update) for cross-highlighting.
"#;

pub(crate) const USAGE_AI_INLINE_CITATION_DEMO: &str = r#"
```rust
use fret_ui_ai::{InlineCitation, SourceItem};
```
"#;

pub(crate) const DOC_AI_SOURCES_DEMO: &str = r#"
## AI sources (demo)

This page demonstrates the AI Elements-aligned `SourcesBlock` surface:

- collapsible root (hidden by default upstream),
- app-owned link activation seam (`on_open_url`),
- optional highlighted row seam for pairing with `InlineCitation`.
"#;

pub(crate) const USAGE_AI_SOURCES_DEMO: &str = r#"
```rust
use fret_ui_ai::{SourceItem, SourcesBlock};
```
"#;

pub(crate) const DOC_AI_ARTIFACT_DEMO: &str = r#"
## AI artifact (demo)

This page is a small demo for the AI Elements-aligned `Artifact` container surfaces in `fret-ui-ai`:

- `Artifact` root (rounded border + shadow + overflow hidden)
- `ArtifactHeader` + `ArtifactTitle` + `ArtifactDescription`
- `ArtifactActions` + `ArtifactAction` + `ArtifactClose` (icon buttons + optional tooltips)
- `ArtifactContent` (scrollable content region)

It exists to validate:

- header/content composition outcomes,
- action button + tooltip wrapping,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_ARTIFACT_DEMO: &str = r#"
```rust
use fret_ui_ai::{Artifact, ArtifactHeader, ArtifactContent, ArtifactAction, ArtifactClose};
```
"#;

pub(crate) const DOC_AI_SHIMMER_DEMO: &str = r#"
## AI shimmer (demo)

This page demonstrates the AI Elements-aligned `Shimmer` surface in `fret-ui-ai`.

Upstream uses a CSS background-clip + gradient animation. In Fret, we approximate the same outcome
by painting a base text run and an overlaid clipped `Canvas` “erase band”:

- the base text is `muted-foreground`,
- a moving “erase” band is colored with `background`.

It exists to validate:

- `duration` / `spread` knobs,
- continuous frame scheduling while the shimmer is visible,
- inline-sizing expectations (upstream renders as `inline-block`; avoid cross-axis stretch when demoing),
- stable `test_id` anchors for screenshot-backed automation checks.
"#;

pub(crate) const USAGE_AI_SHIMMER_DEMO: &str = r#"
```rust
use fret_ui_ai::Shimmer;
```
"#;

pub(crate) const DOC_AI_REASONING_DEMO: &str = r#"
## AI reasoning (demo)

This page demonstrates the AI Elements-aligned `Reasoning` surfaces in `fret-ui-ai`:

- `Reasoning` (collapsible root with streaming-driven open/close policy),
- `ReasoningTrigger` (brain icon + thinking message + chevron),
- `ReasoningContent` (markdown block mounted inside the collapsible).

Upstream behavior to match:

- `defaultOpen ?? isStreaming`
- auto-open when streaming starts (unless `defaultOpen === false`)
- auto-close once, 1000ms after streaming ends
- compute duration (seconds, ceil) between streaming start/end and expose it via the trigger message

It exists to validate:

- timer scheduling (`Effect::SetTimer`) + cancellation correctness,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_REASONING_DEMO: &str = r#"
```rust
use fret_ui_ai::{Reasoning, ReasoningContent, ReasoningTrigger};
```
"#;

pub(crate) const DOC_AI_QUEUE_DEMO: &str = r#"
## AI queue (demo)

This page demonstrates the AI Elements-aligned `Queue` surfaces in `fret-ui-ai`:

- `Queue` root container,
- `QueueSection` + `QueueSectionTrigger` + `QueueSectionLabel` + `QueueSectionContent`,
- `QueueList` (ScrollArea-based constrained list),
- `QueueItem` + `QueueItemIndicator` + `QueueItemContent` + `QueueItemDescription`,
- `QueueItemActions` + `QueueItemAction`,
- `QueueItemAttachment` + `QueueItemFile`.

It exists to validate:

- section open/close behavior (Collapsible),
- hover-driven item chrome + action reveal,
- scroll constraints for long lists (`max-h-40` parity),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_QUEUE_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    Queue, QueueItem, QueueItemAction, QueueItemActions, QueueItemContent, QueueItemIndicator,
    QueueList, QueueSection, QueueSectionLabel, QueueSectionTrigger,
};
```
"#;

pub(crate) const DOC_AI_ATTACHMENTS_DEMO: &str = r#"
## AI attachments (demo)

This page demonstrates the AI Elements-aligned attachments surfaces in `fret-ui-ai`:

- `Attachments` container with three variants (`Grid`, `Inline`, `List`),
- `Attachment` + `AttachmentPreview` + `AttachmentInfo` + `AttachmentRemove`,
- `AttachmentEmpty` empty state surface.

It exists to validate:

- hover-driven remove affordance visibility (best-effort `group-hover` outcome),
- remove intent wiring and keyed identity stability,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_ATTACHMENTS_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    Attachment, AttachmentData, AttachmentFileData, AttachmentVariant, Attachments,
};
```
"#;

pub(crate) const DOC_AI_SUGGESTIONS_DEMO: &str = r#"
## AI suggestions (demo)

This page demonstrates the AI Elements-aligned suggestions surfaces in `fret-ui-ai`:

- `Suggestions` (horizontally scrollable row),
- `Suggestion` (pill button that emits a suggestion string intent).

It exists to validate:

- horizontal scroll sizing behavior (no wrapping),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_SUGGESTIONS_DEMO: &str = r#"
```rust
use fret_ui_ai::{Suggestion, Suggestions};
```
"#;

pub(crate) const DOC_AI_MESSAGE_BRANCH_DEMO: &str = r#"
## AI message branch (demo)

This page demonstrates AI Elements-aligned message branching surfaces in `fret-ui-ai`:

- `MessageBranchContent` (swap between alternate branches),
- `MessageBranchSelector` (previous/next + page label),
- `MessageBranch` (convenience wrapper that composes content + selector).

It exists to validate:

- stable branch identity (hidden branches stay mounted),
- wrap-around navigation (prev from 1 → last, next from last → 1),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_MESSAGE_BRANCH_DEMO: &str = r#"
```rust
use fret_ui_ai::{MessageBranch, MessageBranchContent, MessageBranchSelector};
```
"#;

pub(crate) const DOC_AI_FILE_TREE_DEMO: &str = r#"
## AI file tree (demo)

This page is a small demo for the AI Elements-aligned `FileTree` surface in `fret-ui-ai`.

It exists to validate:

- nested expand/collapse behavior (folder nodes),
- selection intent emission (`on_select`),
- nested row actions do not trigger folder toggle/selection,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_FILE_TREE_DEMO: &str = r#"
```rust
use fret_ui_ai::{FileTree, FileTreeFile, FileTreeFolder};
```
"#;

pub(crate) const DOC_AI_CODE_BLOCK_DEMO: &str = r#"
## AI code block (demo)

This page is a small demo for AI Elements-aligned code artifact surfaces in `fret-ui-ai`:

- `CodeBlock` for fenced-code rendering backed by `ecosystem/fret-code-view`,
- `Snippet` for inline copyable commands/values.

It exists to validate:

- stable copy feedback state (`Copied` timeout),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_CODE_BLOCK_DEMO: &str = r#"
```rust
use fret_ui_ai::{CodeBlock, CodeBlockCopyButton, Snippet, SnippetCopyButton, SnippetInput, SnippetText};
```
"#;

pub(crate) const DOC_AI_SNIPPET_DEMO: &str = r#"
## AI snippet (demo)

This page demonstrates the AI Elements-aligned `Snippet` surface (inline copyable code).
"#;

pub(crate) const USAGE_AI_SNIPPET_DEMO: &str = r#"
```rust
use fret_ui_ai::{Snippet, SnippetCopyButton, SnippetInput, SnippetText};
```
"#;

pub(crate) const DOC_AI_COMMIT_DEMO: &str = r#"
## AI commit (demo)

This page is a small demo for the AI Elements-aligned `Commit` disclosure surface in `fret-ui-ai`.

It exists to validate:

- Collapsible header/content composition (commit header toggles open/closed),
- nested action buttons do not toggle the disclosure (stop-propagation outcome),
- stable copy feedback state (`Copied` timeout),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_COMMIT_DEMO: &str = r#"
```rust
use fret_ui_ai::{Commit, CommitHeader, CommitContent, CommitCopyButton};
```
"#;

pub(crate) const DOC_AI_COMMIT_LARGE_DEMO: &str = r#"
## AI commit large (demo)

This page is a stress-oriented demo for the AI Elements-aligned `Commit` surface in `fret-ui-ai`.

It exists to validate:

- long file lists remain scrollable and stable under view-cache reuse,
- per-row `test_id` selectors are stable for automation,
- file path click seams are app-owned (effects are outside `fret-ui-ai`).
"#;

pub(crate) const USAGE_AI_COMMIT_LARGE_DEMO: &str = r#"
```rust
use fret_ui_ai::{Commit, CommitFiles, CommitFile, CommitFilePath, OnCommitFilePathClick};
```
"#;

pub(crate) const DOC_AI_STACK_TRACE_DEMO: &str = r#"
## AI stack trace (demo)

This page is a small demo for the AI Elements-aligned `StackTrace` disclosure surface in `fret-ui-ai`.

It exists to validate:

- stack trace parsing (error type/message + frames),
- Collapsible header/content composition,
- stable copy feedback state (`Copied` timeout),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_STACK_TRACE_DEMO: &str = r#"
```rust
use fret_ui_ai::{StackTrace, StackTraceFrames, StackTraceCopyButton, parse_stack_trace};
```
"#;

pub(crate) const DOC_AI_STACK_TRACE_LARGE_DEMO: &str = r#"
## AI stack trace large (demo)

This page is a stress-oriented demo for the AI Elements-aligned `StackTrace` surface in `fret-ui-ai`.

It exists to validate:

- long frame lists remain scrollable and stable,
- per-frame `test_id` selectors are stable for automation,
- file-path click seams are app-owned (effects are outside `fret-ui-ai`).
"#;

pub(crate) const USAGE_AI_STACK_TRACE_LARGE_DEMO: &str = r#"
```rust
use fret_ui_ai::{StackTrace, OnStackTraceFilePathClick};
```
"#;

pub(crate) const DOC_AI_TEST_RESULTS_DEMO: &str = r#"
## AI test results (demo)

This page is a small demo for the AI Elements-aligned `TestResults` surfaces in `fret-ui-ai`.

It exists to validate:

- summary (passed/failed/skipped + duration),
- progress bar outcomes,
- suite Collapsible composition (`TestSuite`),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_TEST_RESULTS_DEMO: &str = r#"
```rust
use fret_ui_ai::{TestResults, TestSuite, Test, TestError, TestStatusKind, TestResultsSummaryData};
```
"#;

pub(crate) const DOC_AI_TEST_RESULTS_LARGE_DEMO: &str = r#"
## AI test results large (demo)

This page is a stress-oriented demo for the AI Elements-aligned `TestResults` surfaces in `fret-ui-ai`.

It exists to validate:

- long test lists remain scrollable and stable,
- per-row `test_id` selectors are stable for automation,
- row activate/click seams are app-owned (effects are outside `fret-ui-ai`).
"#;

pub(crate) const USAGE_AI_TEST_RESULTS_LARGE_DEMO: &str = r#"
```rust
use fret_ui_ai::{OnTestActivate, Test, TestResults, TestSuite, TestStatusKind};
```
"#;

pub(crate) const DOC_AI_CHECKPOINT_DEMO: &str = r#"
## AI checkpoint (demo)

This page is a small demo for the AI Elements-aligned `Checkpoint` surfaces in `fret-ui-ai`.

It exists to validate:

- `CheckpointTrigger` button behavior (app-owned activate seam),
- Tooltip hover outcomes with stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_CHECKPOINT_DEMO: &str = r#"
```rust
use fret_ui_ai::{Checkpoint, CheckpointIcon, CheckpointTrigger};
```
"#;

pub(crate) const DOC_AI_CONFIRMATION_DEMO: &str = r#"
## AI confirmation (demo)

This page is a small demo for the AI Elements-aligned `Confirmation` surfaces in `fret-ui-ai`.

It exists to validate:

- conditional rendering based on `ToolUiPartState`,
- approval request actions are app-owned (effects are outside `fret-ui-ai`),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_CONFIRMATION_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    Confirmation, ConfirmationAccepted, ConfirmationAction, ConfirmationActions, ConfirmationRejected,
    ConfirmationRequest, ConfirmationTitle, ToolUiPartApproval, ToolUiPartState,
};
```
"#;

pub(crate) const DOC_AI_ENVIRONMENT_VARIABLES_DEMO: &str = r#"
## AI environment variables (demo)

This page is a small demo for the AI Elements-aligned `EnvironmentVariables` surfaces in `fret-ui-ai`.

It exists to validate:

- controlled/uncontrolled `showValues` behavior (toggle),
- masked value rendering (bullet replacement),
- copy button timing feedback with stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_ENVIRONMENT_VARIABLES_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    EnvironmentVariable, EnvironmentVariableCopyButton, EnvironmentVariableCopyFormat,
    EnvironmentVariableGroup, EnvironmentVariableName, EnvironmentVariableRequired,
    EnvironmentVariableValue, EnvironmentVariables, EnvironmentVariablesContent,
    EnvironmentVariablesHeader, EnvironmentVariablesTitle, EnvironmentVariablesToggle,
};
```
"#;

pub(crate) const DOC_AI_PLAN_DEMO: &str = r#"
## AI plan (demo)

This page is a small demo for the AI Elements-aligned `Plan` surfaces in `fret-ui-ai`.

It exists to validate:

- collapsible open/close behavior (header trigger),
- streaming shimmer states for title/description,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_PLAN_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    Plan, PlanAction, PlanContent, PlanDescription, PlanFooter, PlanHeader, PlanTitle, PlanTrigger,
};
```
"#;

pub(crate) const DOC_AI_TOOL_DEMO: &str = r#"
## AI tool (demo)

This page is a small demo for the AI Elements-aligned `Tool` / `ToolCallBlock` surfaces in `fret-ui-ai`.

It exists to validate:

- status badge mapping for `ToolCallState` (Awaiting Approval / Responded / Running / Pending / Completed / Denied / Error),
- parameter + result rendering via `ToolInput` / `ToolOutput` (JSON pretty-print via `ToolCallPayload`),
- stable `test_id` anchors for `fretboard diag` gates (trigger + content marker).
"#;

pub(crate) const USAGE_AI_TOOL_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    Tool, ToolCall, ToolCallBlock, ToolCallPayload, ToolCallState, ToolContent, ToolHeader, ToolInput,
    ToolOutput, ToolStatus,
};
```
"#;

pub(crate) const DOC_AI_WEB_PREVIEW_DEMO: &str = r#"
## AI web preview (demo)

This page is a small demo for the AI Elements-aligned `WebPreview` surfaces in `fret-ui-ai`.

It exists to validate:

- URL draft input + Enter-to-commit behavior (`Input::on_submit`),
- basic navigation chrome composition (icon buttons + separators),
- console disclosure surface + stable `test_id` anchors for `fretboard diag` gates.

Note: this demo renders **chrome only** (no embedded webview yet). The actual webview backend is
expected to be host/platform-owned and gated behind optional features.
"#;

pub(crate) const USAGE_AI_WEB_PREVIEW_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    WebPreview, WebPreviewBody, WebPreviewConsole, WebPreviewConsoleLog, WebPreviewConsoleLogLevel,
    WebPreviewController, WebPreviewNavigation, WebPreviewNavigationButton, WebPreviewUrl,
};
```
"#;

pub(crate) const DOC_AI_MODEL_SELECTOR_DEMO: &str = r#"
## AI model selector (demo)

This page is a small demo for the AI Elements-aligned `ModelSelector` surfaces in `fret-ui-ai`.

It exists to validate:

- dialog open/close behavior driven by an app-owned model,
- cmdk-style filtering and keyboard navigation via `CommandPalette`,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_MODEL_SELECTOR_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    ModelSelector, ModelSelectorContent, ModelSelectorLogo, ModelSelectorLogoGroup, ModelSelectorName,
};
use fret_ui_shadcn::{CommandPalette, CommandItem};
```
"#;

pub(crate) const DOC_AI_CHAIN_OF_THOUGHT_DEMO: &str = r#"
## AI chain of thought (demo)

This page is a small demo for the AI Elements-aligned `ChainOfThought` surfaces in `fret-ui-ai`.

It exists to validate:

- controlled/uncontrolled open behavior (`defaultOpen` and `open`),
- collapsible content motion for step lists,
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_CHAIN_OF_THOUGHT_DEMO: &str = r#"
```rust
use fret_ui_ai::{
    ChainOfThought, ChainOfThoughtContent, ChainOfThoughtHeader, ChainOfThoughtSearchResult,
    ChainOfThoughtSearchResults, ChainOfThoughtStep, ChainOfThoughtStepStatus,
};
```
"#;

pub(crate) const DOC_AI_SCHEMA_DISPLAY_DEMO: &str = r#"
## AI schema display (demo)

This page is a small demo for the AI Elements-aligned `SchemaDisplay` surface in `fret-ui-ai`.

It exists to validate:

- section collapsibles (Parameters / Request Body / Response),
- recursive property disclosure defaults (`depth < 2` opens by default),
- stable `test_id` anchors for `fretboard diag` gates.
"#;

pub(crate) const USAGE_AI_SCHEMA_DISPLAY_DEMO: &str = r#"
```rust
use fret_ui_ai::{HttpMethod, SchemaDisplay, SchemaParameter, SchemaProperty};
```
"#;

pub(crate) const DOC_AI_IMAGE_DEMO: &str = r#"
## AI image (demo)

This page demonstrates the AI Elements-aligned `Image` surface (generated image presentation).

Note: decode/upload remain app-owned; the UI Gallery demo sources pixels via `fret-ui-assets`
`ImageSource` and renders the resulting `ImageId`.
"#;

pub(crate) const USAGE_AI_IMAGE_DEMO: &str = r#"
```rust
use fret_ui_ai::Image;
```
"#;

pub(crate) const DOC_INSPECTOR_TORTURE: &str = r#"
## Inspector (torture harness)

This page is a baseline for **inspector-style property lists**:

- long, scroll-heavy surfaces,
- small per-row interaction chrome (hover/selection),
- stable identity requirements (editing/focus) under view-cache reuse.

It exists to validate:

- retained-host window boundary updates (attach/detach deltas, no notify-driven dirtiness),
- stale-paint safety while scrolling under cache+shell,
- readiness for migrating future editor inspector panels (GPUI-MVP5-eco-009).
"#;

pub(crate) const USAGE_INSPECTOR_TORTURE: &str = r#"
```rust
// This harness uses `virtual_list_keyed_retained_with_layout` directly.
```
"#;

pub(crate) const DOC_FILE_TREE_TORTURE: &str = r#"
## File tree (torture harness)

This page is a baseline for **file-tree / outline surfaces**:

- large scrolling lists with indentation (tree depth),
- stable row identity under view-cache reuse,
- hover/selection chrome that should not force rerenders.

It exists to validate:

- retained-host window boundary updates (attach/detach deltas, no notify-driven dirtiness),
- stale-paint safety while scrolling under cache+shell,
- readiness for migrating editor file trees and outlines (GPUI-MVP5-eco-009).
"#;

pub(crate) const USAGE_FILE_TREE_TORTURE: &str = r#"
```rust
// This harness uses `virtual_list_keyed_retained_with_layout_fn` directly.
```
"#;

pub(crate) const DOC_BUTTON: &str = r#"
## Button

Validate `variant` / `size` behaviors and default styling consistency.

This layer is **visual recipes**. Interaction policies (hover intent, focus trap, etc.) should live in `fret-ui-kit` / ecosystem crates.

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/base/button.mdx.
"#;

pub(crate) const USAGE_BUTTON: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let btn = shadcn::Button::new("Save")
    .variant(shadcn::ButtonVariant::Default)
    .into_element(cx);
```
"#;
