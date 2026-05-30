# Material 3 Exposed Dropdown Diagnostics Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

ExposedDropdown is closed for the current Material3 sweep evidence standard.

What changed:

- Promoted the existing Material3 ExposedDropdown filtering script into a diagnostics suite.
- Updated the component alignment matrix for `exposed_dropdown`.
- Added a dedicated closeout packet tying recipe/foundation/diagnostics ownership together.
- No Material3 ExposedDropdown component code changed.

Resume guidance:

- Use the filtering diagnostics script before changing ExposedDropdown popup/filtering behavior or
  Autocomplete selector inheritance.
- Use the blur synchronization Rust gate before changing committed-selection/query ownership.
- Use the trailing icon Rust gate before changing dropdown icon event routing.
- Keep state ownership in the Material recipe unless another design system proves the same policy
  should become shared kit infrastructure.
