# M1 Next Gap Audit - 2026-04-24

Status: complete

## Assumptions First

1. The collection helper/readiness verdict remains closed.
   Confidence: Confident.
   Evidence: `imui-collection-helper-readiness-v1` closes without shared helper widening.
   Consequence if wrong: a future lane must name one exact helper shape and prove both collection
   surfaces need it.

2. The editor-notes proof is now the strongest locally testable non-multi-window surface.
   Confidence: Confident.
   Evidence: `imui-editor-notes-inspector-command-v1` and
   `imui-editor-notes-dirty-status-v1` both landed app-owned inspector depth there.
   Consequence if wrong: the next lane should switch to a different proof surface with an equally
   cheap source-policy and surface-test gate.

3. Multi-window/tear-off remains important but is not the right next local slice while macOS and
   multi-window acceptance cannot be exercised here.
   Confidence: Likely.
   Evidence: the active docking/multi-window parity lane remains runner/backend-owned and separate.
   Consequence if wrong: start a runner-owned lane with backend evidence instead of an IMUI helper
   lane.

4. Public IMUI helper growth still needs stronger two-surface proof.
   Confidence: Confident.
   Evidence: recent helper-readiness and dirty-status closeouts both kept public API widening out of
   scope.
   Consequence if wrong: open a helper-readiness follow-on that names one public helper candidate
   and proves it across two first-party surfaces.

## Candidate Ranking

1. Recommended: `imui-editor-notes-draft-actions-v1`.
   Shape: add app-owned `Commit draft` / `Discard draft` or equivalent draft action affordances to
   `editor_notes_demo.rs`, using existing local models only.
   Why: it deepens editor-grade document feel, stays locally testable, and extends the strongest
   proof surface without requiring persistence or dirty-close runtime contracts.
   Minimum gates: editor-notes rail surface test plus a source-policy test.

2. Candidate: `imui-editor-notes-local-validation-v1`.
   Shape: add app-owned notes validation/status copy such as empty/long-note warnings.
   Why: also local and testable, but less Dear ImGui-class than explicit draft actions.

3. Candidate: `imui-collection-second-surface-command-depth-v1`.
   Shape: add one collection command to the shell-mounted `Scene collection` proof.
   Why: useful for two-surface evidence, but risks drifting back into helper-readiness without a
   specific shared helper candidate.

4. Parked: multi-window/tear-off hand-feel.
   Shape: runner/backend acceptance and docking parity.
   Why parked: important but not locally verifiable in the current macOS/multi-window constraint.

5. Parked: generic IMUI/public helper widening.
   Shape: `fret-ui-kit::imui` or `fret-imui` surface expansion.
   Why parked: no fresh evidence yet proves a specific shared helper must exist.

## Decision

The next implementation lane should be `imui-editor-notes-draft-actions-v1`: a narrow app-owned
editor-notes action slice that adds explicit local draft actions without persistence, dirty-close,
clipboard, command-bus, or public IMUI API changes.
