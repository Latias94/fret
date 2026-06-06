use std::sync::Arc;

use fret_app::{App, CommandId};
use fret_core::Px;
use fret_diag::regression_summary::RegressionBundleFollowupCommandV1;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::{
    CMD_REGRESSION_RUN_FOLLOWUP_COMMAND, followup, shell_quote_for_display, text_blob_sized,
};

pub(super) fn followup_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[followup::FollowupResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(
            cx,
            "follow-up history entries: <none for selected bundle>".to_string(),
            Px(84.0),
        );
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_followup_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_result_path_model, |value| {
                    *value = Some(Arc::<str>::from(result_path.clone()))
                });
            host.request_redraw(action_cx.window);
        });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}

#[cfg(test)]
pub(super) fn runnable_followup_command_action_lines(
    commands: &[RegressionBundleFollowupCommandV1],
) -> Vec<String> {
    commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .map(|command| format!("{} ({})", command.label, command.id))
        .collect()
}

pub(super) fn selected_followup_readiness_lines(
    selected_bundle_count: usize,
    commands: &[RegressionBundleFollowupCommandV1],
    baseline_bundle_or_dir: &str,
    baseline_session: &str,
) -> Vec<String> {
    let runnable = commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .collect::<Vec<_>>();
    let manual = commands
        .iter()
        .filter(|command| command.requires_baseline)
        .count();
    let has_visual_compare = commands.iter().any(|command| command.id == "visual-compare");
    let has_footprint_compare = commands
        .iter()
        .any(|command| command.id == "footprint-compare");
    let mut lines = vec![
        format!("selected_bundle_dirs: {selected_bundle_count}"),
        format!("runnable_followups: {}", runnable.len()),
        format!("manual_compare_followups: {manual}"),
        format!(
            "visual_compare_ready: {}",
            if has_visual_compare && !baseline_bundle_or_dir.trim().is_empty() {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "footprint_compare_ready: {}",
            if has_footprint_compare && !baseline_session.trim().is_empty() {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if let Some(first) = runnable.first() {
        lines.push(format!("first_runnable: {} ({})", first.label, first.id));
        lines.push(format!("first_command: {}", first.command_line));
    } else if selected_bundle_count == 0 {
        lines.push("state: no selected bundle evidence yet".to_string());
    } else {
        lines.push("state: selected bundle has no bundle-local follow-up command".to_string());
    }
    lines
}

pub(super) fn materialize_baseline_compare_followup_command(
    command: &RegressionBundleFollowupCommandV1,
    baseline: &str,
) -> Result<RegressionBundleFollowupCommandV1, String> {
    let baseline = baseline.trim();
    if baseline.is_empty() {
        return Err(format!("missing baseline input for {}", command.label));
    }
    let target = command
        .target_bundle_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing target bundle dir for {}", command.label))?;
    if command.id.starts_with("visual-compare") {
        let baseline_arg = shell_quote_for_display(baseline);
        let target_arg = shell_quote_for_display(target);
        let mut out = command.clone();
        out.requires_baseline = false;
        out.command_line = format!(
            "cargo run -p fretboard-dev -- diag compare {baseline_arg} {target_arg} --json"
        );
        out.diag_args = vec![
            "compare".to_string(),
            baseline.to_string(),
            target.to_string(),
            "--json".to_string(),
        ];
        return Ok(out);
    }
    if command.id.starts_with("footprint-compare") {
        let baseline_arg = shell_quote_for_display(baseline);
        let target_arg = shell_quote_for_display(target);
        let mut out = command.clone();
        out.requires_baseline = false;
        out.command_line = format!(
            "cargo run -p fretboard-dev -- diag compare {baseline_arg} {target_arg} --footprint --json"
        );
        out.diag_args = vec![
            "compare".to_string(),
            baseline.to_string(),
            target.to_string(),
            "--footprint".to_string(),
            "--json".to_string(),
        ];
        return Ok(out);
    }
    Err(format!("unsupported baseline compare command {}", command.id))
}

pub(super) fn runnable_followup_command_actions(
    cx: &mut ElementContext<'_, App>,
    pending_command_id_model: &Model<Option<Arc<str>>>,
    commands: &[RegressionBundleFollowupCommandV1],
    in_flight: bool,
) -> AnyElement {
    let runnable = commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .collect::<Vec<_>>();
    if runnable.is_empty() {
        return cx.text("No runnable follow-up commands for this selection.");
    }

    let rows = runnable
        .into_iter()
        .map(|command| {
            let command_id = command.id.clone();
            let command_label = command.label.clone();
            let command_line = command.command_line.clone();
            let pending_command_id_model = pending_command_id_model.clone();
            let action = CommandId::from(CMD_REGRESSION_RUN_FOLLOWUP_COMMAND);
            let on_run: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, reason| {
                let _ = host.models_mut().update(&pending_command_id_model, |value| {
                    *value = Some(Arc::<str>::from(command_id.clone()))
                });
                host.record_pending_command_dispatch_source(action_cx, &action, reason);
                host.dispatch_command(Some(action_cx.window), action.clone());
            });
            let label = shadcn::Badge::new(command_label)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx);
            let run = shadcn::Button::new("Run")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(in_flight)
                .on_activate(on_run)
                .into_element(cx);
            let command = text_blob_sized(cx, command_line, Px(42.0));
            ui::h_row(|_cx| [label, run, command])
                .gap(fret_ui_kit::Space::N2)
                .items_center()
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .max_h(Px(160.0)),
    )
    .into_element(cx)
}

fn short_followup_result_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let marker = ".fret/diag/followups/";
    if let Some((_, suffix)) = path.split_once(marker) {
        return format!("{marker}{suffix}");
    }
    path.rsplit('/').take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/")
}
