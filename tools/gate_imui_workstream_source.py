from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui workstream source"


@dataclass(frozen=True)
class SourceCheck:
    path: Path
    required: list[str]
    forbidden: list[str]


def read_source(path: Path) -> str:
    try:
        return (WORKSPACE_ROOT / path).read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def check_source(check: SourceCheck, failures: list[str]) -> None:
    source = read_source(check.path)
    for marker in check.required:
        if marker not in source:
            failures.append(f"{check.path.as_posix()}: missing {marker}")
    for marker in check.forbidden:
        if marker in source:
            failures.append(f"{check.path.as_posix()}: forbidden {marker}")


def main() -> None:
    checks = [
        SourceCheck(
            Path("docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md"),
            required=[
                "`fret-authoring::Response` must stay unchanged.",
                "Richer lifecycle status stays in `fret-ui-kit::imui::ResponseExt`.",
                "The initial quartet is:",
                "`activated`",
                "`deactivated`",
                "`edited`",
                "`deactivated_after_edit`",
                "Do not widen `crates/fret-ui` or invent a global key-owner model in this lane.",
                "`apps/fret-examples/src/imui_response_signals_demo.rs`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-response-status-lifecycle-v1"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                "python tools/gate_imui_workstream_source.py",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule",
                "`docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow",
                "`ResponseExt` lifecycle vocabulary slice",
                "`docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /",
                "item-local shortcut ownership follow-on",
                "the current helper-local",
                "first-party proof warrants a different narrow lane, and",
                "`docs/workstreams/imui-collection-pane-proof-v1/` now records the closed collection-first /",
                "pane-first proof pair with a no-helper-widening verdict",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/DESIGN.md"),
            required=[
                "focused `activate_shortcut`",
                "`SetNextItemShortcut()` / `SetItemKeyOwner()`",
                "`crates/fret-ui` must remain unchanged unless stronger ADR-backed evidence appears.",
                "`apps/fret-examples/src/imui_response_signals_demo.rs`",
                "`ecosystem/fret-imui/src/tests/interaction.rs`",
                "Global keymap / command routing semantics remain fixed input, not negotiable scope here.",
                "Do not reopen `ResponseExt` lifecycle vocabulary, collection/pane proof breadth, or richer",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"),
            required=[
                "Keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract surface",
                "do not promote a new dedicated key-owner proof demo yet.",
                "menu_item_command_uses_command_metadata_shortcut_and_gating",
                "combo_model_activate_shortcut_is_scoped_to_focused_trigger",
                "runtime keymap / IME arbitration",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md"),
            required=[
                "M2 closes on a no-new-surface verdict.",
                "There is still no stronger first-party consumer pressure for a broader key-owner surface.",
                "Do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade.",
                "Do not add a broader item-local shortcut registration seam.",
                "reopen this question only if stronger first-party proof exceeds the current demo/test",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md"),
            required=[
                "Status: closed closeout record",
                "The current helper-local shortcut seams already close the first-party key-owner demand for this cycle",
                "There is still no stronger first-party consumer pressure for a broader key-owner surface",
                "do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade here by",
                "start a different narrow lane with stronger first-party proof if future pressure still",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-key-owner-surface-v1"',
                '"status": "closed"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md"',
                '"default_action": "close"',
                "python tools/gate_imui_workstream_source.py",
                "imui_response_signals_demo",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-pane-proof-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"),
            required=[
                "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
                "`apps/fret-examples/src/workspace_shell_demo.rs`",
                "`apps/fret-examples/src/editor_notes_demo.rs`",
                "Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the current collection-first proof",
                "Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the current pane-first proof",
                "Keep `apps/fret-examples/src/editor_notes_demo.rs` as the supporting minimal pane rail proof.",
                "Do not introduce a dedicated asset-grid/file-browser proof demo yet.",
                "Do not introduce a narrower child-region-only proof demo yet.",
                "key ownership",
                "promoted shell helpers",
                "runner/backend multi-window parity",
                "broader menu/tab policy",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/collection.rs"),
            required=[
                "Collection-first asset browser proof",
                "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
                "imui-editor-proof.authoring.imui.collection.order-toggle",
                "imui-editor-proof.authoring.imui.collection.browser",
                "imui-editor-proof.authoring.imui.collection.grid",
                "imui_editor_proof_demo.model.authoring_parity.collection_selection",
                "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
                "imui_editor_proof_demo.model.authoring_parity.collection_reverse_order",
                "imui_editor_proof_demo.model.authoring_parity.collection_drop_status",
                "ui.id(asset.id.clone(), |ui| {",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md"),
            required=[
                "Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the collection-first M2 proof surface.",
                "Close M2 with an in-demo asset-browser/file-browser proof instead of a new dedicated demo.",
                "Marquee / box-select stays deferred for M2.",
                "`ecosystem/fret-imui/src/tests/interaction.rs` now proves selected collection drag payloads survive visible order flips.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/workspace_shell_demo.rs"),
            required=[
                "struct WorkspaceShellPaneProofState {",
                "fn workspace_shell_pane_proof<'a, Cx>(",
                "use fret::{imui::prelude::*, shadcn, shadcn::themes::ShadcnColorScheme};",
                "imui_build(cx, out, move |ui| {",
                "workspace-shell-pane-{}-proof.shell",
                "workspace-shell-pane-{}-proof.toolbar",
                "workspace-shell-pane-{}-proof.tabs",
                "workspace-shell-pane-{}-proof.inspector",
                "workspace-shell-pane-{}-proof.status",
                "Decision: keep the current `child_region` seam for M3.",
                "vec![workspace_shell_pane_proof(",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md"),
            required=[
                "Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the pane-first M3 proof surface.",
                "Close M3 with a shell-mounted pane proof inside the existing workspace shell demo.",
                "Keep `ecosystem/fret-ui-kit/src/imui/child_region.rs` unchanged for M3.",
                "No narrower pane-only diagnostics path is required at M3 because the existing workspace shell diag floor remains sufficient.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md"),
            required=[
                "M4 closes on a no-helper-widening verdict.",
                "do not add helper widening, a narrower pane-only demo, or a narrower pane-only diagnostics path",
                "Treat `imui-collection-pane-proof-v1` as:",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-collection-pane-proof-v1"',
                '"status": "closed"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                '"path": "docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md"',
                '"path": "docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md"',
                '"name": "collection-focused-interaction"',
                '"name": "pane-proof-source-policy"',
                '"name": "pane-proof-surface-floor"',
                "python tools/gate_imui_workstream_source.py",
                "python tools/gate_imui_facade_teaching_source.py",
                "collection_drag_payload_preserves_selected_keys_across_order_flip",
                "workspace_shell_demo",
                "editor_notes_demo",
                "workspace_shell_pane_proof_surface",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md"),
            required=[
                "keep the public `fret-ui-kit::imui` surface stable while restructuring internals",
                "`ecosystem/fret-ui-kit/src/imui.rs` still mixes the module hub",
                "`ecosystem/fret-ui-kit/src/imui/options.rs` and `ecosystem/fret-ui-kit/src/imui/response.rs`",
                "The first implementation slice should stay structural:",
                "Do not widen `crates/fret-ui`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/M0_BASELINE_AUDIT_2026-04-21.md"),
            required=[
                "`ecosystem/fret-ui-kit/src/imui.rs`: 2209 lines",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`: 1027 lines",
                "M0 chooses this first implementation slice:",
                "modularize `options.rs`",
                "modularize `response.rs`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md"),
            required=[
                "`options.rs` now re-exports smaller private owner files",
                "`response.rs` now re-exports smaller private owner files",
                "no public type names changed",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`",
                "`ecosystem/fret-ui-kit/src/imui.rs`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md"),
            required=[
                "`interaction_runtime.rs` now re-exports the same helper family over five private owner files",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs`",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/disabled.rs`",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs`",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`",
                "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs`",
                "hover/lifecycle/drag/disabled bookkeeping are reviewable as separate owners",
                "`ecosystem/fret-ui-kit/src/imui.rs`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md"),
            required=[
                "`ecosystem/fret-ui-kit/src/imui.rs` now re-imports smaller owner files for support helpers",
                "`ecosystem/fret-ui-kit/src/imui/facade_support.rs`",
                "`ecosystem/fret-ui-kit/src/imui/floating_options.rs`",
                "`UiWriterUiKitExt`",
                "`ImUiFacade` / `UiWriterImUiFacadeExt` writer glue",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md"),
            required=[
                "`ecosystem/fret-ui-kit/src/imui/facade_writer.rs`",
                "`ImUiFacade`",
                "`UiWriterImUiFacadeExt`",
                "`ecosystem/fret-ui-kit/src/imui.rs`: 125 lines",
                "one dedicated owner file",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md"),
            required=[
                "Status: closed",
                "This lane is closed.",
                "`options.rs` -> smaller private owner files",
                "`interaction_runtime.rs` -> owner files under `interaction_runtime/`",
                "`facade_writer.rs`",
                "Do not reopen this lane by default.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-facade-internal-modularization-v1"',
                '"status": "closed"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                '"path": "docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md"',
                "python tools/gate_imui_workstream_source.py",
                "imui_response_contract_smoke",
                "cargo nextest run -p fret-imui --no-fail-fast",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/README.md"),
            required=[
                "`docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/roadmap.md"),
            required=[
                "`docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/todo-tracker.md"),
            required=[
                "`docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`",
                "`docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-box-select-v1/DESIGN.md"),
            required=[
                "land one app-owned background marquee / box-select slice on the existing proof surface",
                "The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.",
                "The first correct target is:",
                "background-only marquee / box-select slice inside",
                "Do not begin by designing a shared helper surface.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md"),
            required=[
                "The closed collection/pane proof lane explicitly deferred marquee / box-select for M2.",
                "The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.",
                "The current proof surface already has the right ingredients for a narrow app-owned box-select",
                "Dear ImGui treats box-select as part of collection depth",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md"),
            required=[
                "Background drag now draws a marquee overlay and updates collection selection app-locally.",
                "Selection stays normalized to visible collection order",
                "Plain background click clears the selection;",
                "baseline set.",
                "No new public `fret-ui-kit::imui` box-select helper is admitted in this lane.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md"),
            required=[
                "Status: closed closeout record",
                "Treat `imui-collection-box-select-v1` as:",
                "a closeout record for the landed app-owned background marquee / box-select slice",
                "Start a different narrower follow-on only if stronger first-party proof shows either:",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-collection-box-select-v1"',
                '"status": "closed"',
                '"scope_kind": "closeout"',
                '"follow_on_of": "imui-collection-pane-proof-v1"',
                '"path": "docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md"',
                "python tools/gate_imui_workstream_source.py",
                "proof_collection_drag_rect_normalizes_drag_direction",
                "proof_collection_box_select_replace_uses_visible_collection_order",
                "proof_collection_box_select_append_preserves_baseline_and_adds_hits",
                "imui_editor_collection_box_select_surface",
                '"default_action": "start_follow_on"',
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-collection-box-select-v1/` now records the closed",
                "background-only box-select slice in `imui_editor_proof_demo`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/README.md"),
            required=[
                "`docs/workstreams/imui-collection-box-select-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/roadmap.md"),
            required=[
                "`docs/workstreams/imui-collection-box-select-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/todo-tracker.md"),
            required=[
                "`docs/workstreams/imui-collection-box-select-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md"),
            required=[
                "The generic key-owner lane stays closed; this lane is collection proof depth, not generic helper growth.",
                "The first landable target is therefore narrow:",
                "make the collection scope itself a focusable keyboard owner in the proof demo,",
                "`Arrow` / `Home` / `End` to move the active tile in visible order,",
                "Do not start by designing a shared helper or a new generic shortcut facade.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md"),
            required=[
                "The closed collection box-select lane explicitly deferred collection keyboard-owner depth.",
                "The generic key-owner lane already closed on a no-new-surface verdict and should stay closed.",
                "The current proof surface already has the right ingredients for a narrow app-owned keyboard",
                'The smallest credible slice is still narrower than "full parity"',
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md"),
            required=[
                "The collection scope now owns a focusable keyboard region locally in the proof demo.",
                "`Arrow` / `Home` / `End` now move the active tile",
                "`Shift+Arrow` / `Shift+Home` / `Shift+End` now extend the selected range",
                "`Escape` now clears the selected set while keeping the current keyboard location app-defined.",
                "No new generic `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale facade is admitted here.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md"),
            required=[
                "Status: closed closeout record",
                "Treat `imui-collection-keyboard-owner-v1` as:",
                "generic key-owner no-new-surface verdict remains closed",
                "No reopening of the generic key-owner lane.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-collection-keyboard-owner-v1"',
                '"status": "closed"',
                '"scope_kind": "closeout"',
                '"follow_on_of": "imui-collection-box-select-v1"',
                '"path": "docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md"',
                "python tools/gate_imui_workstream_source.py",
                "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
                "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
                "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
                "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
                "imui_editor_collection_keyboard_owner_surface",
                '"default_action": "start_follow_on"',
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-collection-keyboard-owner-v1/` now records the closed",
                "app-owned collection keyboard-owner slice in `imui_editor_proof_demo`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/README.md"),
            required=[
                "`docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/roadmap.md"),
            required=[
                "`docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/todo-tracker.md"),
            required=[
                "`docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-delete-action-v1/DESIGN.md"),
            required=[
                "The closed collection keyboard-owner lane already deferred collection action semantics.",
                "The first landable target is therefore narrow:",
                "make `Delete` / `Backspace` remove the current selected set in visible collection order,",
                "add one explicit button-owned affordance for the same action,",
                "Do not start by designing a shared collection command facade or helper.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md"),
            required=[
                "The closed collection keyboard-owner lane explicitly deferred collection action semantics.",
                "The proof-budget rule and runtime contract posture remain unchanged for this lane.",
                "The current proof surface already has the right ingredients for a narrow app-owned delete slice:",
                "Dear ImGui keeps delete requests at the collection proof surface rather than using them as a reason to widen unrelated runtime or shared-helper contracts.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md"),
            required=[
                "The collection proof now supports one app-owned delete-selected action slice.",
                "`Delete` / `Backspace` now remove the selected set from the stored asset model.",
                "The explicit action button reuses the same delete helper instead of forking policy.",
                "Remaining assets, selection, and keyboard active tile now reflow app-locally after deletion.",
                "No new public `fret-ui-kit::imui` collection action helper is admitted in this lane.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md"),
            required=[
                "Status: closed closeout record",
                "Treat `imui-collection-delete-action-v1` as:",
                "a closeout record for the landed app-owned collection delete-selected slice",
                "No reopening of the generic key-owner lane or the closed keyboard-owner folder.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-collection-delete-action-v1"',
                '"status": "closed"',
                '"scope_kind": "closeout"',
                '"follow_on_of": "imui-collection-keyboard-owner-v1"',
                '"path": "docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md"',
                '"path": "docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md"',
                "python tools/gate_imui_workstream_source.py",
                "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
                "proof_collection_delete_selection_picks_previous_visible_item_at_end",
                "imui_editor_collection_delete_action_surface",
                '"default_action": "start_follow_on"',
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-collection-delete-action-v1/` now records the closed",
                "app-owned collection delete-selected slice in `imui_editor_proof_demo`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/README.md"),
            required=[
                "`docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/roadmap.md"),
            required=[
                "`docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/todo-tracker.md"),
            required=[
                "`docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
                "`docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-context-menu-v1/DESIGN.md"),
            required=[
                "The closed collection delete-action lane already deferred context-menu action breadth.",
                "The first landable target is therefore narrow:",
                "reuse the current app-owned delete helper inside one shared collection popup scope,",
                "support right-click on both assets and collection background,",
                "Do not start by designing a shared collection context-menu helper or broader command surface.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md"),
            required=[
                "The closed collection delete-action lane explicitly deferred context-menu breadth.",
                "The current proof surface already has the right ingredients for a narrow app-owned collection context menu:",
                "The menu/popup helper floor already exists generically, so this lane is not a justification to widen shared helper ownership.",
                "Dear ImGui keeps the asset-browser context menu at the proof surface and routes delete through the same selection model instead of inventing a separate command contract.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md"),
            required=[
                "The collection proof now supports one shared popup scope for app-owned quick actions.",
                "Right-click on an unselected asset now replaces selection with that asset before opening the popup.",
                "Right-click on collection background now opens the same popup without widening helper surface.",
                "The popup reuses the existing delete helper instead of forking collection action policy.",
                "No new public `fret-ui-kit::imui` collection context-menu helper is admitted in this lane.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md"),
            required=[
                "Status: closed closeout record",
                "Treat `imui-collection-context-menu-v1` as:",
                "a closeout record for the landed app-owned collection context-menu slice",
                "No reopening of the closed delete-action lane or the generic menu/key-owner lanes.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-collection-context-menu-v1"',
                '"status": "closed"',
                '"scope_kind": "closeout"',
                '"follow_on_of": "imui-collection-delete-action-v1"',
                '"path": "docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md"',
                '"path": "docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md"',
                '"path": "docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md"',
                "python tools/gate_imui_workstream_source.py",
                "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
                "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
                "imui_editor_collection_context_menu_surface",
                '"default_action": "start_follow_on"',
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-collection-context-menu-v1/` now records the closed",
                "app-owned collection context-menu slice in `imui_editor_proof_demo`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/README.md"),
            required=[
                "`docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/roadmap.md"),
            required=[
                "`docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/todo-tracker.md"),
            required=[
                "`docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
                "`docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`",
            ],
            forbidden=[],
        ),
    ]

    failures: list[str] = []
    for check in checks:
        check_source(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
