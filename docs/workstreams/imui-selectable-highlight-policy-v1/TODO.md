# ImUi Selectable Highlight Policy TODO

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): all tasks below landed in the closeout slice. Start a new narrow
follow-on for broader selectable/list behavior.

- [x] Resolve this as a new narrow follow-on from the active Dear ImGui gap lane.
- [x] Add `SelectableOptions::highlighted` with a default of `false`.
- [x] Route highlighted rows through hover-style palette without changing selected semantics.
- [x] Keep disabled highlighted rows visually muted.
- [x] Change the input-text picker active candidate from `selected: checked || active` to
  `selected: checked, highlighted: active`.
- [x] Add focused smoke/unit coverage and record gates.
