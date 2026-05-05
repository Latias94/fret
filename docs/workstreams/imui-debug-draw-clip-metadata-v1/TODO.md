# ImUi Debug Draw Clip Metadata v1 TODO

Status: Closed.

## Completed

- [x] Add `clip_rect` and `clip_depth` to `DebugDrawCommandSummary`.
- [x] Add `max_clip_depth` and `final_clip_depth` to `DebugDrawListSummary`.
- [x] Simulate source-level clip state in command summary merge order.
- [x] Add unit coverage for nested clip state across draw commands.
- [x] Add public smoke coverage for aggregate clip-depth fields.
- [x] Update IMUI gap audit and workstream indexes.

## Future Follow-Ons

- [ ] Backend scissor/draw-call attribution if diagnostics need renderer-level clip evidence.
- [ ] Hit-test-aware debug draw interaction if editor overlays need selectable clipped geometry.
- [ ] Callback/user draw commands only if a contract-safe renderer extension point is designed.
