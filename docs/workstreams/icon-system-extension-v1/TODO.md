# Icon System Extension v1 — TODO

Status: Closed closeout tracker (contract shipped; follow-on only)
Last updated: 2026-04-09

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-09.md`

Closeout note on 2026-04-09:

- `CLOSEOUT_AUDIT_2026-04-09.md` closes this lane on the v1 icon contract / runtime surface /
  pack protocol goal
- read any checklist rows below as shipped scope accounting, not as an open execution queue

## Queue A — baseline and contract freeze

- [x] Audit ADR 0065 against the current implementation and record the concrete drift list.
  - Recorded in `BASELINE_AUDIT_2026-04-09.md`.
  - Verified drift:
    - glyph fallback is named in the ADR but not implemented in `IconSource`
    - multicolor icon rendering exists below the default declarative authoring path
    - third-party pack protocol is implied by docs/examples but not yet captured as a lane-owned contract
    - current evidence docs still cite a non-existent `ecosystem/fret-icons/tests/semantic_alias_conflicts.rs`
- [x] Freeze the correctness-first target interface state before implementation.
  - Recorded in `TARGET_INTERFACE_STATE.md`.
  - This lane now prefers the durable public boundary over the smallest contract delta.
- [x] Decide whether icon render mode should live:
  - inside `IconSource`,
  - in a parallel metadata type,
  - or behind a narrower helper-only contract.
  - Accepted in `BASELINE_DECISION_2026-04-09.md`: keep `IconSource` data-only and move
    render-intent into a first-class icon-definition / presentation layer.
- [x] Decide the declaration/runtime surface for multicolor icons.
  - Minimum question: do we want a distinct `SvgImage`-style element surface, or a single icon
    surface with explicit render mode?
  - Accepted in `BASELINE_DECISION_2026-04-09.md`: add a distinct `SvgImage`-style declarative
    surface instead of overloading `SvgIconProps`.
- [x] Decide the minimum approved protocol for third-party icon-pack/import crates.
  - Keep the contract explicit and grep-friendly.
  - Accepted in `BASELINE_DECISION_2026-04-09.md`: explicit pack install seams + build-time
    import/vendoring workflow, explicit metadata/provenance home, and no runtime Iconify client in
    core.

## Queue B — first implementation slice

- [x] Refactor `ecosystem/fret-icons` toward a first-class icon-definition / resolved-icon
  contract.
  - Alias resolution must preserve source + fallback + presentation semantics together.
  - Avoid helper-only shadow contracts.
- [x] Add the runtime/declarative surface needed for multicolor SVG icon rendering.
  - Landed via `SvgImageProps` / `ElementContext::svg_image(...)` and `SceneOp::SvgImage`
    lowering.
- [x] Add a distinct `SvgImage`-style declarative surface in `crates/fret-ui`.
  - Evidence: `crates/fret-ui/src/{element.rs,elements/cx.rs,declarative/frame.rs,declarative/mount.rs,declarative/host_widget/{measure.rs,layout.rs,paint.rs},declarative/tests/svg_image.rs}`.
- [x] Update `fret-ui-kit` icon helpers to preserve the existing monochrome golden path while
  adding an explicit registry-authored icon posture.
  - `icon(...)` remains the themed monochrome path.
  - `icon_authored(...)` / `icon_authored_with(...)` now honor registry `IconRenderMode`.
- [x] Update `fret-bootstrap` guidance or hooks only if the new contract actually changes bootstrap
  behavior.
  - Landed via `BootstrapBuilder::register_icon_pack_contract(...)`.
  - `with_lucide_icons()` / `with_radix_icons()` now use the contract-aware path.

## Queue C — proof and docs

- [x] Add or refresh regression tests in the owning crates.
  - Added `svg_image_props_paint_to_svg_image_scene_op` in `crates/fret-ui`.
  - Added authored/themed icon posture coverage in `ecosystem/fret-ui-kit/src/declarative/icon.rs`.
- [x] Add one smallest proof surface for multicolor icons.
  - Landed as focused runtime/helper tests instead of a larger gallery refactor.
- [x] Write down pack metadata / provenance expectations once the contract type is real.
  - Landed via `IconPackRegistration`, `InstalledIconPacks`, `PACK_METADATA`, and updated ADR/docs.
- [x] Update `docs/crate-usage-guide.md` once the pack protocol and authoring surface are real.
  - Landed in `docs/crate-usage-guide.md` and `docs/examples/todo-app-golden-path.md`.
- [x] Update ADR/alignment docs if the accepted contract needs to change rather than merely catch up
  to existing ADR language.
  - Updated `docs/adr/0065-icon-system-and-asset-packaging.md`.
  - Updated `docs/adr/IMPLEMENTATION_ALIGNMENT.md` to reflect the full landed v1 closure.

## Open boundaries to protect

- Do not move vendor-specific icon policy into `crates/fret-ui`.
- Do not turn the icon registry into a generic remote asset registry.
- Do not widen this lane into pack-specific parity work unless the contract cannot be settled
  without that evidence.
