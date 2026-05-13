# M2 Assist Request Contract

Status: Landed
Date: 2026-05-12

## Assumptions-First Resume

- Confident: completion, hover, and code-action features need a view-layer request/payload contract
  before the editor surface grows more `CodeEditorHandle` setters. Evidence: `DESIGN.md` names
  ad-hoc handle growth as the highest-risk gap, and `TARGET_INTERFACE_STATE.md` lists these
  features as extension inputs rather than widget-owned policies.
- Confident: the owner is `fret-code-editor-view`, not `crates/fret-ui` and not
  `fret-code-editor` paint/input internals. Evidence: ADR 0185 keeps buffer/view/surface separate,
  and the existing diagnostics/decorations/gutter/semantic-token contracts already live in the view
  crate.
- Confident: overlay placement, focus, dismissal, listbox navigation, hover intent, and command
  execution remain ecosystem/app policy. Evidence: `M2_OVERLAY_FEATURE_BOUNDARY_2026-05-12.md` and
  `docs/code-editor.md` explicitly keep those decisions out of `fret-code-editor`.
- Likely: request and payload validation should be unit-testable without launching a UI. Evidence:
  adjacent view-layer contracts validate buffer byte ranges, char boundaries, logical/display
  anchors, stable ids, and deterministic data facts without runtime dependencies.

## Decision

Add a first assist data contract in `fret-code-editor-view`:

- `EditorAssistRequest`: revision-aware request facts for completion, hover, and code actions.
- `CompletionList` / `CompletionCandidate`: candidate payloads, active candidate identity, replace
  range, commit intent, commit characters, optional command id.
- `HoverPayload`: hover contents plus related command ids.
- `CodeActionList` / `CodeAction`: code-action payloads, command ids, related diagnostic ids, and
  disabled/preferred facts.

These types are re-exported from `fret-code-editor` so app authors can use the root editor facade
for the public surface. The root facade also re-exports stable buffer/view types needed by the new
signatures (`TextBuffer`, `Revision`, `DisplayMap`, `DisplayPoint`, and related buffer edit
contracts).

## Must-Be-True Outcomes

- Assist requests validate buffer byte ranges, selection byte offsets, UTF-8 char boundaries, and
  display-row anchors through `DisplayMap`.
- Completion payloads can express an active candidate without owning listbox navigation or
  dismissal.
- Hover payloads can reference commands as ids without owning overlay focus, hover intent, or
  placement policy.
- Code actions can reference command ids and diagnostics without owning a menu/popover/lightbulb
  recipe.
- The root `fret-code-editor` public surface exposes all new public signature types used by the
  assist contract.

## Evidence Anchors

- View contract implementation: `ecosystem/fret-code-editor-view/src/assist.rs`
- View crate exports: `ecosystem/fret-code-editor-view/src/lib.rs`
- Surface root exports: `ecosystem/fret-code-editor/src/lib.rs`
- Root public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`
- Authoring guide: `docs/code-editor.md`

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view assist --lib --no-fail-fast
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
python tools/check_layering.py
```
