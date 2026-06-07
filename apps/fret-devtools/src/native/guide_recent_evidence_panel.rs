use fret_app::App;
use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::ElementContext;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::guide_recent_evidence_state::collect_guide_recent_evidence_state;
use super::ui_primitives::{diag_section, text_blob_sized};
use super::{
    CMD_COPY_RECENT_EVIDENCE_REPORT, CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR,
    CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND, CMD_COPY_RECENT_FAILED_EVIDENCE_JSON,
    CMD_COPY_RECENT_FAILED_EVIDENCE_PATH, CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON,
    CMD_RERUN_RECENT_FAILED_EVIDENCE, CMD_SELECT_RECENT_FAILED_EVIDENCE, State,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FirstOpenRecentEvidenceActionSpec {
    pub(super) label: &'static str,
    pub(super) command: &'static str,
    pub(super) disabled: bool,
}

pub(super) fn first_open_recent_evidence_action_specs(
    has_failed_evidence: bool,
    failed_evidence_rerunnable: bool,
) -> Vec<FirstOpenRecentEvidenceActionSpec> {
    vec![
        FirstOpenRecentEvidenceActionSpec {
            label: "Copy recent evidence report",
            command: CMD_COPY_RECENT_EVIDENCE_REPORT,
            disabled: false,
        },
        FirstOpenRecentEvidenceActionSpec {
            label: "Select failed evidence",
            command: CMD_SELECT_RECENT_FAILED_EVIDENCE,
            disabled: !has_failed_evidence,
        },
        FirstOpenRecentEvidenceActionSpec {
            label: "Rerun failed evidence",
            command: CMD_RERUN_RECENT_FAILED_EVIDENCE,
            disabled: !failed_evidence_rerunnable,
        },
    ]
}

pub(super) fn first_open_recent_evidence_action_row(
    cx: &mut ElementContext<'_, App>,
    specs: &[FirstOpenRecentEvidenceActionSpec],
) -> AnyElement {
    let actions = specs
        .iter()
        .map(|spec| {
            shadcn::Button::new(spec.label)
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(spec.disabled)
                .on_click(spec.command)
                .into_element(cx)
        })
        .collect::<Vec<_>>();
    ui::h_row(|_cx| actions)
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

pub(super) fn guide_recent_evidence_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let recent = collect_guide_recent_evidence_state(cx.app, st);
    let recent_evidence_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Copy recent evidence report")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_COPY_RECENT_EVIDENCE_REPORT)
                .into_element(cx),
            shadcn::Button::new("Select failed evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent.target.is_none())
                .on_click(CMD_SELECT_RECENT_FAILED_EVIDENCE)
                .into_element(cx),
            shadcn::Button::new("Rerun failed evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!recent.rerunnable)
                .on_click(CMD_RERUN_RECENT_FAILED_EVIDENCE)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence path")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent.target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_PATH)
                .into_element(cx),
            shadcn::Button::new("Copy failed bundle dir")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!recent.bundle_dir_available)
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence command")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent.target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND)
                .into_element(cx),
            shadcn::Button::new("Copy failed evidence JSON")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent.target.is_none())
                .on_click(CMD_COPY_RECENT_FAILED_EVIDENCE_JSON)
                .into_element(cx),
            shadcn::Button::new("Open failed evidence JSON")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(recent.target.is_none())
                .on_click(CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let recent_evidence_blob = text_blob_sized(cx, recent.report_text, Px(132.0));
    diag_section(
        cx,
        "Recent Evidence",
        "Latest GUI-launched gate, workflow, and follow-up artifacts restored from the shared diagnostics histories.",
        vec![recent_evidence_actions, recent_evidence_blob],
    )
}
