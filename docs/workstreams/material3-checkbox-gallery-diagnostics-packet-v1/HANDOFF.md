# Material 3 Checkbox Gallery Diagnostics Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

Checkbox gallery diagnostics are closed for the current Material3 surface.

What changed:

- `ui-gallery-material3-checkbox-centered-chrome.json` now opens the dedicated Checkbox page instead
  of the aggregate Material3 gallery page.
- Centered chrome diagnostics pass and prove the stable root/chrome selectors.
- Tri-state screenshot diagnostics pass across standard and expressive states.
- The Material3 component matrix now records Checkbox as gallery-diagnostics aligned.

Resume guidance:

- Do not move Checkbox indication or minimum target behavior out of Material foundation without a
  second design-system consumer.
- If future Checkbox drift appears, start from the two promoted diagnostics scripts and the focused
  `radio_alignment` checkbox tests.
