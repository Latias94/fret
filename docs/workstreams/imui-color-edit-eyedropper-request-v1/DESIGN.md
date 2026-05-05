# ImUi Color Edit Eyedropper Request v1

Status: Closed narrow P1 affordance follow-on
Last updated: 2026-05-05

The editor `ColorEdit` needed an eyedropper entry point, but Fret does not currently expose a
portable platform screen-sampling effect or completed frame readback event. ADR 0120 and ADR 0125
describe future frame capture/readback semantics, but the runtime `Effect` surface does not yet
contain that contract. This lane therefore adds an app-owned request hook instead of pretending the
framework can sample arbitrary screen pixels today.

## Ownership

- `ColorEditOptions::on_eyedropper` owns the opt-in app callback.
- `ColorEditEyedropperRequest` owns alpha application rules for sampled colors.
- `popup/eyedropper.rs` owns the popup command row and synchronous sample application.
- `fret-runtime`, `fret-platform`, `fret-render`, and `fret-imui` are not widened.

## Must-Be-True Outcomes

- Apps can opt into an `Eyedropper` popup command without direct popup rewrites.
- Returning `Some(sampled_color)` updates the color model, draft text, and validation state.
- Returning `None` leaves asynchronous app/platform sampling in app ownership.
- RGB-only controls preserve the target alpha when applying a sampled color.
- The implementation does not add or depend on a new runtime/platform effect.

## Non-Goals

- No OS screen sampling implementation.
- No renderer readback or frame-capture effect.
- No global eyedropper mode.
- No default visible eyedropper button when an app has not provided a callback.
