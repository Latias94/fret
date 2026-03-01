# UI Diagnostics / Inspector TODO

Status: living (workstream roadmap)

This file tracks the next iteration of Fret's GPUI/Zed-style inspector workflow:

- **Bundles**: portable `bundle.json` snapshots for AI triage and repro sharing
- **Picking**: click-to-select semantics nodes and generate stable selectors
- **Scripted behaviors**: deterministic, step-per-frame interaction tests

Related docs:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

## What is already done (baseline)

- Bundle export (`bundle.json`) + `latest.txt` pointer
- Script harness (`diag run`, `diag suite ui-gallery`)
- `test_id` contract: authoring → semantics snapshot → bundle → selector
- Declarative identity: pick results include `GlobalElementId` and scripts can target it via `{"kind":"global_element_id","element":...}` (best for harness tests)
- Picking (one-shot): `diag pick`, `diag pick-script`, `diag pick-apply`
- Continuous inspect mode: `diag inspect on|off|toggle|status` (file-triggered, AI-friendly)
- In-app inspect shortcuts (diagnostics-only): `Esc` exit, `Ctrl+C` copy selector, `Ctrl+Shift+C` copy details, `L` lock/unlock, `Alt+Up/Down` parent-chain navigation
- In-app hover/pick prefer hit-test routing when available (fallback to bounds-based pick).
- Selector validation exists (uniqueness + optional `root_z_index` gating) and is used by in-app “copy details” and focus selection.
- Explainability panel (“why is input blocked?”) implemented and shown in help mode (cheap: bounded lists; no label printing).
- View cache frame stats exported in bundles (`debug.stats.view_cache_*`, `debug.stats.invalidation_*`)

## Milestone M1: “Inspect UX parity” (highest ROI)

1. **Hover/selection highlight overlay (non-invasive)**
   - Show hovered node bounds (and picked node bounds) with z-layer indication.
   - Render label: `role`, `label`, `test_id`, `node_id`, and root `z_index`.
   - Interaction: toggle on/off; lock selection; copy selector to clipboard.
   - Ownership: should live in `fret-ui-kit` / app overlay, not `fret-ui`.
   - Status:
     - MVP implemented in `fret-bootstrap` as a diagnostics-only overlay layer (border + label) while inspection is active (scripts/picking).
     - Continuous inspect toggle implemented via file-triggered `inspect.json` + `inspect.touch` (`fretboard diag inspect ...`).
     - In-app toggle/help implemented (diagnostics-only): `Ctrl/Cmd+Alt+I` toggles inspect, `Ctrl/Cmd+Alt+H` shows shortcut help.
     - In-app shortcuts implemented (diagnostics-only): `Esc` exit, `Ctrl+C` copy selector, `Ctrl+Shift+C` copy details, `L` lock/unlock selection.
     - Locked navigation implemented (diagnostics-only): `Alt+Up/Down` walks the semantics parent chain with a small “back to child” stack.
     - Local neighborhood view implemented in help mode: parent + siblings + children + type-to-filter.
     - Remaining gaps: no full semantics tree browser (expand/collapse, virtualization, global search).

2. **Pick modes**
   - One-shot pick (already): click once and write `pick.result.json`.
   - Continuous picking mode: hover shows candidate, click selects (optional click pass-through).
   - Keyboard “inspect focus”: select current focused node without pointer. (done: press `F` in inspect mode)

3. **Selector quality improvements**
   - Score/select best selector:
     - Prefer `test_id`.
     - Otherwise prefer `(role + label + ancestors)` with minimal ancestor chain that is still unique under barrier/root.
   - Add optional `root_z_index` gating so scripts pick the right overlay.
   - Avoid using redacted labels when `FRET_DIAG_REDACT_TEXT=1`.
   - Status:
     - In-app copy-details (`Ctrl+Shift+C`) now includes a scored selector-candidates list (match count + chosen node).
     - Runtime selector resolution honors `root_z_index` when present; validated selectors may add it when needed to become unique.

4. **Explainability: “why is input blocked?”**
   - Add a minimal in-app explanation panel (shown in help mode) that surfaces:
     - hit-test target under pointer (best-effort),
     - barrier roots (semantics ids),
     - visible roots summary: `(root_id, z_index, blocks_underlay_input, hit_testable)`.
    - Keep it cheap (no per-frame full tree dumps; bounded string work).
    - Respect redaction (`FRET_DIAG_REDACT_TEXT=1`).
    - Status:
      - Implemented in `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_explain.rs` and rendered in help mode.
      - Note: current output intentionally avoids labels/text, so it remains safe under redaction by construction.

## Milestone M2: “Script stability + coverage”

1. **More scripted actions**
   - Status:
     - Implemented (v2): pointer move/down/up/drag, wheel, scroll-into-view, long press, pointer sweeps.
     - Remaining gaps: double-click action, and a higher-level “paste” action (beyond `set_clipboard_text`).

2. **More predicates and assertions**
   - `focused_descendant_is` (done)
   - `active_descendant_is` (done: `ActiveItemIs` / `active_item_is`)
   - `exists_under` / `not_exists_under` (done)
   - `value_equals` for text fields (done; note locale sensitivity of `value` strings)

3. **Golden “first regression pack” expansion**
   - Status:
     - Dialog: escape close + focus restore, focus trap tab-cycle scripts landed.
     - Select: commit/dismiss/focus-restore scripts landed (scoped assertions).
     - Combobox: commit/dismiss/focus-restore scripts landed (scoped assertions + state rows).
     - Menus: DropdownMenu + ContextMenu scripts landed (open/close, keyboard nav/typeahead, last-action assertions).
     - Pending: Docking (tab drag, drop targets, split, close tab).

## Milestone M3: “AI debugging ergonomics”

1. **Bundle enrichment (debug only)**
   - Include keymap resolution snapshot and active command context.
   - Include focus stack/scope metadata (focus scopes, modal barrier reasons).
   - Include cache counters (paint cache hits/misses, view cache invalidations).

2. **Better result artifacts**
   - `diag pick` prints multiple selectors with a confidence score.
   - `diag pick-apply` optionally patches N pointers in one run (batch update).
   - Support “patch by match” (find first step where selector matches old value).

3. **Sharing**
   - Bundle compression option (zip) with stable naming. (done: `fretboard diag pack`, plus `--include-triage`)
   - Optional screenshot packaging for visual overlay debugging. (done: `FRET_DIAG_GPU_SCREENSHOTS=1` writes `target/fret-diag/screenshots/<bundle>/...` + `manifest.json`; `fretboard diag pack --include-screenshots` packs them; viewer can auto-select by manifest and render as an overlay background; scripts can also `capture_screenshot` and wait on `screenshots.result.json`)
   - Simple bug template: attach `bundle.json` + optional `script.json`.
   - Offline bundle viewer: `tools/fret-bundle-viewer` (supports semantics tree, perf panels, `triage.json` export, and `.zip`/paste import).

## CI / automation strategy (open questions)

- Do we want a “headless” runner for behavior tests, or a “robot” runner (real window + event injection)?
- On Windows CI, can we reliably run a UI window and inject events without flakiness?
- If we can’t: should we treat scripted tests as “developer regression pack” instead of CI-gating?

## Notes / constraints

- Keep `fret-ui` as a mechanism/contract layer; inspector UI/interaction policies belong to ecosystem/app overlays.
- `test_id` is debug/test-only and must not conflict with a11y `label`/`name` fields.
