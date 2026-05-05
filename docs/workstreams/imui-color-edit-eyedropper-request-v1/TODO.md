# ImUi Color Edit Eyedropper Request v1 TODO

Status: Closed
Last updated: 2026-05-05

- [x] Confirm no current runtime/platform screen-sampling effect exists.
- [x] Add an app-owned `ColorEditEyedropperRequest` and callback hook.
- [x] Show a popup `Eyedropper` command only when the app provides the hook.
- [x] Apply synchronous sampled colors through the existing model/draft/error update path.
- [x] Preserve target alpha when `show_alpha=false`.
- [x] Add focused tests and source-policy anchors that prevent accidental runtime effect coupling.
- [x] Update roadmap, tracker, gap audit, umbrella evidence, and workstream catalog.
- [x] Run focused gates and close the lane.

## Follow-On Candidates

- Native/web platform screen sampling as an explicit runtime/platform contract.
- Deeper side-preview polish.
- Higher-fidelity color picker visual parity.
