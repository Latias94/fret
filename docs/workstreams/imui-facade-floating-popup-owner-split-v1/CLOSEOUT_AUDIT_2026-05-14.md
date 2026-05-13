# Closeout Audit - 2026-05-14

Status: closed

## Verdict

Close `imui-facade-floating-popup-owner-split-v1` as a private source-owner split.

The floating/popup trait default-body cluster is no longer owned directly by `facade_writer.rs`; it
now lives in `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`.

## Shipped Scope

- Moved only trait default implementation bodies for floating layer/area, popup, tooltip,
  drag/drop, context-popup, and in-window floating-window entry points.
- Kept all public method names, signatures, and behavior shape unchanged.
- Kept `fret-imui` thin and unchanged.
- Kept `crates/fret-ui` runtime contracts unchanged.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
- `tools/gate_imui_workstream_source.py` locks the new owner anchors.

## Follow-On Boundary

Do not reopen this folder for public trait-surface reshaping, additive popup/floating behavior,
docking/multi-window behavior, or public runtime APIs. Start separate narrow follow-ons for those
concerns.
