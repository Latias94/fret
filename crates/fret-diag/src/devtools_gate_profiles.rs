use crate::util::shell_quote_arg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolsGateProfileV1 {
    pub id: &'static str,
    pub label: &'static str,
    pub command_line: &'static str,
    pub evidence_files: &'static [&'static str],
    pub notes: &'static [&'static str],
}

pub const DEVTOOLS_GATE_STALE_COMMAND: &str = "cargo run -p fretboard-dev -- diag run <script.json> --check-stale-paint <test-id> --check-stale-scene <test-id> --json";
pub const DEVTOOLS_GATE_PIXELS_CHANGED_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag run <script.json> --check-pixels-changed <test-id> --json";
pub const DEVTOOLS_GATE_PERF_THRESHOLDS_COMMAND: &str = "cargo run -p fretboard-dev -- diag perf <script-or-suite> --repeat 7 --warmup-frames 5 --perf-threshold-agg p95 --max-top-total-us <us> --max-renderer-encode-scene-us <us> --json";
pub const DEVTOOLS_GATE_RESOURCE_FOOTPRINT_THRESHOLDS_COMMAND: &str = "cargo run -p fretboard-dev -- diag repro <script-or-suite> --max-working-set-bytes <bytes> --max-peak-working-set-bytes <bytes> --max-cpu-avg-percent-total-cores <percent> --json --launch -- <app-command>";
pub const DEVTOOLS_GATE_RESOURCE_FOOTPRINT_COMPARE_COMMAND: &str = "cargo run -p fretboard-dev -- diag compare <baseline-session> <candidate-session> --footprint --json";
pub const DEVTOOLS_GATE_SCRIPT_TARGET_PROFILE_IDS_V1: &[&str] =
    &["stale-paint-scene", "pixels-changed"];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DevtoolsGateScriptTargetCommandInputV1<'a> {
    pub script_json: &'a str,
    pub test_id: &'a str,
}

impl<'a> DevtoolsGateScriptTargetCommandInputV1<'a> {
    pub fn new(script_json: &'a str, test_id: &'a str) -> Self {
        Self {
            script_json,
            test_id,
        }
    }
}

pub const DEVTOOLS_GATE_PROFILES_V1: &[DevtoolsGateProfileV1] = &[
    DevtoolsGateProfileV1 {
        id: "stale-paint-scene",
        label: "stale paint/scene",
        command_line: DEVTOOLS_GATE_STALE_COMMAND,
        evidence_files: &["bundle.json", "script.result.json"],
        notes: &[
            "checks semantics movement/value changes against scene fingerprints",
            "use when pixels appear stale after layout, search, or scroll updates",
        ],
    },
    DevtoolsGateProfileV1 {
        id: "pixels-changed",
        label: "pixels changed",
        command_line: DEVTOOLS_GATE_PIXELS_CHANGED_COMMAND,
        evidence_files: &[
            "check.pixels_changed.json",
            "screenshots.result.json",
            "bundle.json",
        ],
        notes: &[
            "requires screenshot capture evidence for the target region",
            "use when a specific semantics region must repaint visibly",
        ],
    },
    DevtoolsGateProfileV1 {
        id: "perf-thresholds",
        label: "perf thresholds",
        command_line: DEVTOOLS_GATE_PERF_THRESHOLDS_COMMAND,
        evidence_files: &[
            "layout.perf.summary.v1.json",
            "check.perf_thresholds.json",
            "regression.summary.json",
        ],
        notes: &[
            "records p95/tail thresholds for editor-grade smoothness",
            "use with repeat counts and warmup frames instead of one-shot timing",
        ],
    },
    DevtoolsGateProfileV1 {
        id: "resource-footprint-thresholds",
        label: "resource footprint thresholds",
        command_line: DEVTOOLS_GATE_RESOURCE_FOOTPRINT_THRESHOLDS_COMMAND,
        evidence_files: &[
            "resource.footprint.json",
            "check.resource_footprint.json",
            "repro.summary.json",
        ],
        notes: &[
            "captures process footprint and enforces explicit CPU/memory ceilings",
            "prefer a built app binary for stable process attribution",
        ],
    },
    DevtoolsGateProfileV1 {
        id: "resource-footprint-compare",
        label: "resource footprint compare",
        command_line: DEVTOOLS_GATE_RESOURCE_FOOTPRINT_COMPARE_COMMAND,
        evidence_files: &["resource.footprint.json", "compare.json"],
        notes: &[
            "compares baseline and candidate session footprint summaries",
            "use after capturing two sessions with comparable launch paths",
        ],
    },
];

pub fn devtools_gate_profiles_v1() -> &'static [DevtoolsGateProfileV1] {
    DEVTOOLS_GATE_PROFILES_V1
}

pub fn devtools_gate_script_target_profile_ids_v1() -> &'static [&'static str] {
    DEVTOOLS_GATE_SCRIPT_TARGET_PROFILE_IDS_V1
}

pub fn devtools_gate_script_target_command_line(
    profile_id: &str,
    input: DevtoolsGateScriptTargetCommandInputV1<'_>,
) -> Option<String> {
    let profile_id = profile_id.trim();
    if !DEVTOOLS_GATE_SCRIPT_TARGET_PROFILE_IDS_V1.contains(&profile_id) {
        return None;
    }
    let profile = DEVTOOLS_GATE_PROFILES_V1
        .iter()
        .find(|profile| profile.id == profile_id)?;
    let script_json = shell_quote_arg_or_placeholder(input.script_json, "<script.json>");
    let test_id = shell_quote_arg_or_placeholder(input.test_id, "<test-id>");
    Some(
        profile
            .command_line
            .replace("<script.json>", &script_json)
            .replace("<test-id>", &test_id),
    )
}

pub fn devtools_gate_profile_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    let mut lines = vec![
        "gate route: first-class-gates".to_string(),
        format!("artifacts root: {artifacts_root}"),
    ];
    for profile in DEVTOOLS_GATE_PROFILES_V1 {
        lines.push(format!("gate profile: {}", profile.id));
        lines.push(format!("{}: {}", profile.label, profile.command_line));
        lines.push(format!(
            "{} evidence: {}",
            profile.label,
            profile.evidence_files.join(", ")
        ));
        lines.push(format!(
            "{} notes: {}",
            profile.label,
            profile.notes.join("; ")
        ));
    }
    lines
}

fn shell_quote_arg_or_placeholder(value: &str, placeholder: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        placeholder.to_string()
    } else {
        shell_quote_arg(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devtools_gate_profiles_include_first_class_gate_taxonomy() {
        let ids = devtools_gate_profiles_v1()
            .iter()
            .map(|profile| profile.id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "stale-paint-scene",
                "pixels-changed",
                "perf-thresholds",
                "resource-footprint-thresholds",
                "resource-footprint-compare",
            ]
        );
        assert!(
            devtools_gate_profiles_v1()
                .iter()
                .all(|profile| !profile.command_line.trim().is_empty())
        );
    }

    #[test]
    fn devtools_gate_profile_lines_surface_artifacts_and_threshold_commands() {
        let text = devtools_gate_profile_lines("target/fret-diag").join("\n");

        assert!(text.contains("gate route: first-class-gates"));
        assert!(text.contains("artifacts root: target/fret-diag"));
        assert!(text.contains("gate profile: stale-paint-scene"));
        assert!(text.contains(DEVTOOLS_GATE_STALE_COMMAND));
        assert!(text.contains("gate profile: pixels-changed"));
        assert!(text.contains("check.pixels_changed.json"));
        assert!(text.contains("gate profile: perf-thresholds"));
        assert!(text.contains("check.perf_thresholds.json"));
        assert!(text.contains("gate profile: resource-footprint-thresholds"));
        assert!(text.contains("--max-working-set-bytes <bytes>"));
        assert!(text.contains("check.resource_footprint.json"));
        assert!(text.contains("gate profile: resource-footprint-compare"));
        assert!(text.contains("--footprint --json"));
    }

    #[test]
    fn devtools_gate_script_target_profiles_are_parameterized() {
        assert_eq!(
            devtools_gate_script_target_profile_ids_v1(),
            &["stale-paint-scene", "pixels-changed"]
        );

        let input = DevtoolsGateScriptTargetCommandInputV1::new(
            "tools/diag-scripts/ui-editor/imui/smoke.json",
            "imui.editor.name",
        );
        let stale = devtools_gate_script_target_command_line("stale-paint-scene", input).unwrap();
        assert_eq!(
            stale,
            "cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/smoke.json --check-stale-paint imui.editor.name --check-stale-scene imui.editor.name --json"
        );
        let pixels = devtools_gate_script_target_command_line("pixels-changed", input).unwrap();
        assert_eq!(
            pixels,
            "cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/smoke.json --check-pixels-changed imui.editor.name --json"
        );
        assert!(devtools_gate_script_target_command_line("perf-thresholds", input).is_none());
    }

    #[test]
    fn devtools_gate_script_target_command_preserves_placeholders_until_filled() {
        let input = DevtoolsGateScriptTargetCommandInputV1::default();
        let command = devtools_gate_script_target_command_line("stale-paint-scene", input).unwrap();

        assert!(command.contains("<script.json>"));
        assert!(command.contains("<test-id>"));

        let quoted = devtools_gate_script_target_command_line(
            "pixels-changed",
            DevtoolsGateScriptTargetCommandInputV1::new("target/my script.json", "button ok"),
        )
        .unwrap();
        assert!(quoted.contains("'target/my script.json'"));
        assert!(quoted.contains("'button ok'"));
    }
}
