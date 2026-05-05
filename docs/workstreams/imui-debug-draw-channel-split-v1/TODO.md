# ImUi Debug Draw Channel Split v1 TODO

Status: Closed.

## Completed

- [x] Add `channels_split`, `channels_set_current`, and `channels_merge` to `ImUiDebugDrawList`.
- [x] Store current channel commands without changing every draw helper call site.
- [x] Auto-merge a still-open split at the end of `debug_draw(...)`.
- [x] Make `command_count` and `is_empty` account for pending channels.
- [x] Add focused tests for channel order and invalid channel switches.
- [x] Add compile smoke coverage for the public API.

## Future Follow-Ons

- [ ] Callback/user draw commands.
- [ ] Raw mesh/primitive writer API if a real editor overlay proves it is needed.
- [ ] Per-command metadata or draw command introspection.
