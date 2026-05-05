# ImUi Debug Draw Response Surface v1 Milestones

Status: Closed.

## M0 - API Shape

Exit criteria:

- `debug_draw` and `debug_draw_with_options` return a response object.
- Summary data is available after command merging.
- Default options keep the helper paint-only.

Result: Complete.

## M1 - Opt-In Interaction

Exit criteria:

- Interaction options wrap the canvas in a pressable response surface.
- The pressable response uses existing IMUI response/lifecycle helpers.
- No core/runtime or renderer contract is widened.

Result: Complete.

## M2 - Teaching and Evidence

Exit criteria:

- Cookbook proof uses the returned response and summary accessors.
- Existing diagnostics smoke waits for response metadata.
- Focused tests and workstream gates pass.

Result: Complete.
