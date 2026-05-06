# P3 Public Surface Catalog - 2026-05-06

Status: public surface audit; no API refactor opened yet
Last updated: 2026-05-06

## Decision

Keep the current IMUI public surface shape. Do not move functionality between crates or widen
`fret-imui` from this source-audit lane.

The current owner split is defensible:

- `fret-imui` is the small policy-light immediate authoring frontend.
- `fret::imui` is the app-facing optional lane and the default teaching path.
- `fret::imui::kit` is the policy-heavy widget/response/options lane from `fret-ui-kit`.
- `fret::imui::editor` is the editor-grade controls/composites lane from `fret-ui-editor`.
- `fret::imui::docking` is the docking helper lane from `fret-docking`.
- direct `fret_imui`, `fret_ui_kit::imui`, and `fret_ui_editor::imui` imports remain acceptable in
  crate tests, internal smoke tests, and advanced/domain-local proofs, but should not be the
  first-open app teaching surface.

The main rule for future growth is stricter than the current Rust visibility:

- public in `fret-ui-kit::imui` is not automatically a default-path recommendation,
- app-facing docs/examples should prefer `fret::imui`,
- broad helper additions still need two proof surfaces plus a focused gate before they are treated
  as stable authoring vocabulary.

## Surface Catalog

| Surface | Owner | Purpose | Allowed growth | Do not add here |
| --- | --- | --- | --- | --- |
| `fret_imui::{imui, imui_raw, imui_build, ImUi}` | `ecosystem/fret-imui` | Minimal immediate-mode authoring frontend over `ElementContext` and `AnyElement` | policy-light control-flow helpers, identity/keying helpers, state/query selector bridges behind features | widget policy, editor controls, docking policy, renderer/platform dependencies |
| `fret::imui` | `ecosystem/fret` facade | App-facing optional IMUI lane | curated re-exports that keep app docs on one root path | default app prelude widening, component policy that belongs in `kit` / `editor` / `docking` |
| `fret::imui::prelude` | `ecosystem/fret` facade | Common explicit imports for app IMUI views | traits and entrypoints needed to write ordinary IMUI panels | everything from `kit::*`; keep heavy nouns behind named submodules |
| `fret::imui::kit` | `ecosystem/fret-ui-kit` | Policy-heavy immediate widgets, response signals, options, debug draw, floating, popup, table, text, selection helpers | helpers proven by at least two surfaces or needed by public cookbook/proof gates | app-owned collection behavior, editor-specific controls, raw Dear ImGui string/cursor sugar without proof |
| `fret::imui::editor` | `ecosystem/fret-ui-editor` | Editor-grade controls/composites plus thin immediate adapters | editor controls, property grids, inspector panels, adapter functions over declarative source-of-truth controls | generic IMUI runtime helpers, app collection state, docking/window policy |
| `fret::imui::docking` | `ecosystem/fret-docking` | Docking adoption helpers for immediate authoring | docking shell helpers tied to dock graph/runtime ops | generic layout helpers, general collection behavior, editor-control policy |

## Source-Backed Facts

- `ecosystem/fret-imui/src/lib.rs` exports only the minimal authoring frontend plus optional
  selector/query extension modules. Its direct dependencies are `fret-authoring` and `fret-ui`.
- `ecosystem/fret-ui-kit/src/imui.rs` is feature-gated behind `fret-ui-kit/imui` and owns the large
  policy-heavy surface: `ImUiFacade`, `UiWriterImUiFacadeExt`, response types, options, debug draw,
  child regions, tables, text, drag/drop, floating, and `ImUiMultiSelectState`.
- `ecosystem/fret-ui-editor/src/imui.rs` is a thin adapter layer. Every public function accepts a
  declarative editor control/composite and calls its `into_element(...)`; it must not become a
  parallel widget implementation.
- `ecosystem/fret/src/lib.rs` exposes `fret::imui` only behind the `imui` feature. The root module
  re-exports the minimal entrypoints and places policy-heavy lanes behind `kit`, `editor`, and
  `docking` submodules.
- `ecosystem/fret/src/lib.rs` already contains source tests that assert the explicit IMUI module,
  README/rustdoc teaching text, and that IMUI traits stay out of `fret::app::prelude::*`.
- `apps/fret-cookbook/src/lib.rs` already guards the first-party IMUI cookbook examples so they
  teach `fret::imui` rather than raw `fret_imui` / `fret_ui_kit::imui` imports.

## Hazards

1. **Wide `kit::*` export pressure**
   - `fret::imui::kit` intentionally re-exports `fret_ui_kit::imui::*`.
   - Consequence: any new public symbol in `fret-ui-kit::imui` becomes reachable through the app
     facade.
   - Guardrail: new helpers need proof/gates before docs or examples teach them as authoring
     vocabulary.
2. **Direct crate imports still exist in advanced/test surfaces**
   - `fret-imui` tests and `fret-ui-kit` smoke tests correctly use direct crate paths.
   - Some domain-local demos may still use direct `fret_imui` / `fret_ui_kit::imui` imports.
   - Guardrail: classify each direct import as test/internal/advanced before promoting that surface
     as public teaching.
3. **Adapter modules are visible but not first-open vocabulary**
   - `fret-ui-kit::imui::adapters` remains public as a contract seam for adapter tests.
   - Guardrail: keep it out of cookbook/default docs unless a real external adapter consumer needs
     it.
4. **Editor adapter drift**
   - `fret-ui-editor::imui` is only safe while it stays a thin adapter over declarative controls.
   - Guardrail: do not add editor-only immediate implementations there; add declarative controls
     first, then expose thin IMUI adapters.

## Public Growth Rules

Use this order before adding new public IMUI API:

1. Pick the owner layer:
   - control flow / identity only: `fret-imui`;
   - generic IMUI widget policy: `fret-ui-kit::imui`;
   - editor controls/composites: `fret-ui-editor` plus `fret-ui-editor::imui`;
   - docking: `fret-docking::imui`;
   - app-facing teaching: `fret::imui`.
2. Name at least two proof surfaces unless the API is a thin adapter over an already-public
   declarative control.
3. Add or name a focused gate before documenting the helper as public authoring vocabulary.
4. Keep `fret::app::prelude::*` free of IMUI imports; IMUI stays opt-in through
   `fret::imui::prelude::*`.
5. Prefer typed options and explicit ids over Dear ImGui string parsing or stack-like global state.

## Follow-On Threshold

Open a narrow public-surface follow-on only when one of these happens:

- a public helper lands in `fret-ui-kit::imui` and needs a facade/source gate before cookbook
  promotion,
- a domain demo is promoted to a first-open teaching surface and still uses direct raw IMUI crates,
- a second external adapter consumer needs the `adapters` seam documented,
- `fret-ui-editor::imui` starts to accumulate logic that belongs in declarative editor controls,
- root `fret::imui` needs a breaking re-export policy change.

Suggested follow-on names:

- `imui-facade-source-gate-v1` for source-policy gate hardening,
- `imui-editor-adapter-thinness-v1` for editor adapter drift,
- `imui-kit-public-growth-v1` for a concrete new helper promotion.

## Gates

Suggested audit/gate commands:

```powershell
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
rg -n "pub mod imui|pub use fret_imui|pub use fret_ui_kit::imui|pub mod kit|pub mod editor|pub mod docking|pub mod prelude" ecosystem/fret/src/lib.rs
cargo nextest run -p fret root_surface_exposes_explicit_imui_module readme_and_rustdoc_expose_imui_as_explicit_optional_surface --no-fail-fast
cargo check -p fret --no-default-features --features imui
```

## Gate Results

2026-05-06 local results:

- `python tools/audit_crate.py --crate fret-imui` passed and confirmed the minimal dependency
  posture.
- `python tools/audit_crate.py --crate fret-ui-kit` passed and identified
  `src/imui/facade_writer.rs` as a large policy-heavy surface.
- `python tools/audit_crate.py --crate fret-ui-editor` passed and identified editor controls plus
  `src/imui.rs` as the thin adapter surface.
- `python tools/audit_crate.py --crate fret` passed and confirmed the wide facade shape.
- `rg -n "pub mod imui|pub use fret_imui|pub use fret_ui_kit::imui|pub mod kit|pub mod editor|pub mod docking|pub mod prelude" ecosystem/fret/src/lib.rs`
  passed and found the explicit `fret::imui` module plus `kit`, `editor`, `docking`, and `prelude`
  submodules.
- `cargo nextest run -p fret root_surface_exposes_explicit_imui_module readme_and_rustdoc_expose_imui_as_explicit_optional_surface --no-fail-fast`
  passed: 2 tests run, 2 passed.
- `cargo check -p fret --no-default-features --features imui` passed.

Rerun the gates above when this note changes or when a public IMUI surface is promoted.
