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
pub const DEVTOOLS_GATE_PERF_THRESHOLD_PROFILE_ID_V1: &str = "perf-thresholds";

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DevtoolsGatePerfThresholdCommandInputV1<'a> {
    pub target: &'a str,
    pub repeat: &'a str,
    pub warmup_frames: &'a str,
    pub perf_threshold_agg: &'a str,
    pub max_top_total_us: &'a str,
    pub max_renderer_encode_scene_us: &'a str,
}

impl<'a> DevtoolsGatePerfThresholdCommandInputV1<'a> {
    pub fn new(
        target: &'a str,
        repeat: &'a str,
        warmup_frames: &'a str,
        perf_threshold_agg: &'a str,
        max_top_total_us: &'a str,
        max_renderer_encode_scene_us: &'a str,
    ) -> Self {
        Self {
            target,
            repeat,
            warmup_frames,
            perf_threshold_agg,
            max_top_total_us,
            max_renderer_encode_scene_us,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolsGateCommandV1 {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub diag_args: Vec<String>,
    pub missing_inputs: Vec<&'static str>,
}

pub type DevtoolsGateScriptTargetCommandV1 = DevtoolsGateCommandV1;

impl DevtoolsGateCommandV1 {
    pub fn is_runnable(&self) -> bool {
        self.missing_inputs.is_empty() && !self.diag_args.is_empty()
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
    devtools_gate_script_target_command(profile_id, input).map(|command| command.command_line)
}

pub fn devtools_gate_script_target_command(
    profile_id: &str,
    input: DevtoolsGateScriptTargetCommandInputV1<'_>,
) -> Option<DevtoolsGateCommandV1> {
    let profile_id = profile_id.trim();
    if !DEVTOOLS_GATE_SCRIPT_TARGET_PROFILE_IDS_V1.contains(&profile_id) {
        return None;
    }
    let profile = DEVTOOLS_GATE_PROFILES_V1
        .iter()
        .find(|profile| profile.id == profile_id)?;
    let script_json = shell_quote_arg_or_placeholder(input.script_json, "<script.json>");
    let test_id = shell_quote_arg_or_placeholder(input.test_id, "<test-id>");
    let command_line = profile
        .command_line
        .replace("<script.json>", &script_json)
        .replace("<test-id>", &test_id);
    let missing_inputs = script_target_missing_inputs(input);
    let diag_args = if missing_inputs.is_empty() {
        script_target_diag_args(profile.id, input.script_json.trim(), input.test_id.trim())
    } else {
        Vec::new()
    };
    Some(DevtoolsGateCommandV1 {
        id: profile.id.to_string(),
        label: profile.label.to_string(),
        command_line,
        diag_args,
        missing_inputs,
    })
}

pub fn devtools_gate_perf_threshold_command_line(
    input: DevtoolsGatePerfThresholdCommandInputV1<'_>,
) -> String {
    devtools_gate_perf_threshold_command(input).command_line
}

pub fn devtools_gate_perf_threshold_command(
    input: DevtoolsGatePerfThresholdCommandInputV1<'_>,
) -> DevtoolsGateCommandV1 {
    let profile = DEVTOOLS_GATE_PROFILES_V1
        .iter()
        .find(|profile| profile.id == DEVTOOLS_GATE_PERF_THRESHOLD_PROFILE_ID_V1)
        .expect("perf threshold profile must exist");
    let target = shell_quote_arg_or_placeholder(input.target, "<script-or-suite>");
    let repeat = shell_quote_arg_or_placeholder(input.repeat, "<repeat>");
    let warmup_frames = shell_quote_arg_or_placeholder(input.warmup_frames, "<warmup-frames>");
    let perf_threshold_agg = shell_quote_arg_or_placeholder(input.perf_threshold_agg, "<agg>");
    let max_top_total_us = shell_quote_arg_or_placeholder(input.max_top_total_us, "<us>");
    let max_renderer_encode_scene_us =
        shell_quote_arg_or_placeholder(input.max_renderer_encode_scene_us, "<us>");
    let command_line = format!(
        "cargo run -p fretboard-dev -- diag perf {target} --repeat {repeat} --warmup-frames {warmup_frames} --perf-threshold-agg {perf_threshold_agg} --max-top-total-us {max_top_total_us} --max-renderer-encode-scene-us {max_renderer_encode_scene_us} --json"
    );
    let missing_inputs = perf_threshold_missing_inputs(input);
    let diag_args = if missing_inputs.is_empty() {
        perf_threshold_diag_args(input)
    } else {
        Vec::new()
    };
    DevtoolsGateCommandV1 {
        id: profile.id.to_string(),
        label: profile.label.to_string(),
        command_line,
        diag_args,
        missing_inputs,
    }
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

fn script_target_missing_inputs(
    input: DevtoolsGateScriptTargetCommandInputV1<'_>,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if input.script_json.trim().is_empty() {
        missing.push("script.json");
    }
    if input.test_id.trim().is_empty() {
        missing.push("test-id");
    }
    missing
}

fn script_target_diag_args(profile_id: &str, script_json: &str, test_id: &str) -> Vec<String> {
    let mut args = vec!["run".to_string(), script_json.to_string()];
    match profile_id {
        "stale-paint-scene" => {
            args.extend([
                "--check-stale-paint".to_string(),
                test_id.to_string(),
                "--check-stale-scene".to_string(),
                test_id.to_string(),
            ]);
        }
        "pixels-changed" => {
            args.extend(["--check-pixels-changed".to_string(), test_id.to_string()]);
        }
        _ => return Vec::new(),
    }
    args.push("--json".to_string());
    args
}

fn perf_threshold_missing_inputs(
    input: DevtoolsGatePerfThresholdCommandInputV1<'_>,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if input.target.trim().is_empty() {
        missing.push("script-or-suite");
    }
    if parse_nonzero_u64(input.repeat).is_none() {
        missing.push("repeat");
    }
    if parse_u64(input.warmup_frames).is_none() {
        missing.push("warmup-frames");
    }
    if !matches!(
        input.perf_threshold_agg.trim(),
        "max" | "p95" | "p90" | "p50"
    ) {
        missing.push("perf-threshold-agg");
    }
    if parse_nonzero_u64(input.max_top_total_us).is_none() {
        missing.push("max-top-total-us");
    }
    if parse_nonzero_u64(input.max_renderer_encode_scene_us).is_none() {
        missing.push("max-renderer-encode-scene-us");
    }
    missing
}

fn perf_threshold_diag_args(input: DevtoolsGatePerfThresholdCommandInputV1<'_>) -> Vec<String> {
    vec![
        "perf".to_string(),
        input.target.trim().to_string(),
        "--repeat".to_string(),
        input.repeat.trim().to_string(),
        "--warmup-frames".to_string(),
        input.warmup_frames.trim().to_string(),
        "--perf-threshold-agg".to_string(),
        input.perf_threshold_agg.trim().to_string(),
        "--max-top-total-us".to_string(),
        input.max_top_total_us.trim().to_string(),
        "--max-renderer-encode-scene-us".to_string(),
        input.max_renderer_encode_scene_us.trim().to_string(),
        "--json".to_string(),
    ]
}

fn parse_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

fn parse_nonzero_u64(value: &str) -> Option<u64> {
    parse_u64(value).filter(|value| *value > 0)
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
    fn devtools_gate_script_target_commands_include_runnable_diag_args() {
        let input = DevtoolsGateScriptTargetCommandInputV1::new(
            "tools/diag-scripts/smoke.json",
            "button.ok",
        );
        let stale = devtools_gate_script_target_command("stale-paint-scene", input).unwrap();
        assert!(stale.is_runnable());
        assert!(stale.missing_inputs.is_empty());
        assert_eq!(
            stale.diag_args,
            vec![
                "run",
                "tools/diag-scripts/smoke.json",
                "--check-stale-paint",
                "button.ok",
                "--check-stale-scene",
                "button.ok",
                "--json"
            ]
        );

        let pixels = devtools_gate_script_target_command("pixels-changed", input).unwrap();
        assert!(pixels.is_runnable());
        assert_eq!(
            pixels.diag_args,
            vec![
                "run",
                "tools/diag-scripts/smoke.json",
                "--check-pixels-changed",
                "button.ok",
                "--json"
            ]
        );
    }

    #[test]
    fn devtools_gate_script_target_command_preserves_placeholders_until_filled() {
        let input = DevtoolsGateScriptTargetCommandInputV1::default();
        let command = devtools_gate_script_target_command("stale-paint-scene", input).unwrap();

        assert!(!command.is_runnable());
        assert_eq!(command.missing_inputs, vec!["script.json", "test-id"]);
        assert!(command.diag_args.is_empty());
        assert!(command.command_line.contains("<script.json>"));
        assert!(command.command_line.contains("<test-id>"));

        let quoted = devtools_gate_script_target_command(
            "pixels-changed",
            DevtoolsGateScriptTargetCommandInputV1::new("target/my script.json", "button ok"),
        )
        .unwrap();
        assert!(quoted.command_line.contains("'target/my script.json'"));
        assert!(quoted.command_line.contains("'button ok'"));
        assert_eq!(
            quoted.diag_args,
            vec![
                "run",
                "target/my script.json",
                "--check-pixels-changed",
                "button ok",
                "--json"
            ]
        );
    }

    #[test]
    fn devtools_gate_perf_threshold_command_preserves_placeholders_until_filled() {
        let command = devtools_gate_perf_threshold_command(
            DevtoolsGatePerfThresholdCommandInputV1::default(),
        );

        assert_eq!(command.id, "perf-thresholds");
        assert_eq!(command.label, "perf thresholds");
        assert!(!command.is_runnable());
        assert_eq!(
            command.missing_inputs,
            vec![
                "script-or-suite",
                "repeat",
                "warmup-frames",
                "perf-threshold-agg",
                "max-top-total-us",
                "max-renderer-encode-scene-us",
            ]
        );
        assert!(command.diag_args.is_empty());
        assert!(command.command_line.contains("<script-or-suite>"));
        assert!(command.command_line.contains("--repeat <repeat>"));
        assert!(
            command
                .command_line
                .contains("--warmup-frames <warmup-frames>")
        );
        assert!(command.command_line.contains("--perf-threshold-agg <agg>"));
    }

    #[test]
    fn devtools_gate_perf_threshold_command_includes_runnable_diag_args() {
        let input = DevtoolsGatePerfThresholdCommandInputV1::new(
            "perf-docking-arbitration-steady",
            "7",
            "5",
            "p95",
            "35000",
            "12000",
        );
        let command = devtools_gate_perf_threshold_command(input);

        assert!(command.is_runnable());
        assert!(command.missing_inputs.is_empty());
        assert_eq!(
            command.command_line,
            "cargo run -p fretboard-dev -- diag perf perf-docking-arbitration-steady --repeat 7 --warmup-frames 5 --perf-threshold-agg p95 --max-top-total-us 35000 --max-renderer-encode-scene-us 12000 --json"
        );
        assert_eq!(
            command.diag_args,
            vec![
                "perf",
                "perf-docking-arbitration-steady",
                "--repeat",
                "7",
                "--warmup-frames",
                "5",
                "--perf-threshold-agg",
                "p95",
                "--max-top-total-us",
                "35000",
                "--max-renderer-encode-scene-us",
                "12000",
                "--json",
            ]
        );
    }

    #[test]
    fn devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers() {
        let command =
            devtools_gate_perf_threshold_command(DevtoolsGatePerfThresholdCommandInputV1::new(
                "target/my suite",
                "0",
                "abc",
                "avg",
                "-1",
                "",
            ));

        assert!(!command.is_runnable());
        assert!(command.command_line.contains("'target/my suite'"));
        assert_eq!(
            command.missing_inputs,
            vec![
                "repeat",
                "warmup-frames",
                "perf-threshold-agg",
                "max-top-total-us",
                "max-renderer-encode-scene-us",
            ]
        );
    }
}
