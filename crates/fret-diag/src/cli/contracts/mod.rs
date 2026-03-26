use clap::{CommandFactory, Parser, Subcommand};

pub(crate) mod commands;
pub(crate) mod shared;

#[derive(Debug, Parser)]
#[command(
    name = "fretboard diag",
    about = "Diagnostics tooling for the Fret workspace.",
    after_help = "Examples:\n  fretboard diag poke\n  fretboard diag latest\n  fretboard diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --launch -- cargo run -p fret-ui-gallery --release\n  fretboard diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release\n  fretboard diag campaign list --lane smoke --tag ui-gallery",
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
pub(crate) struct DiagCliContract {
    #[command(subcommand)]
    pub command: DiagCommandContract,
}

#[derive(Debug, Subcommand)]
pub(crate) enum DiagCommandContract {
    Agent(commands::agent::AgentCommandArgs),
    AiPacket(commands::ai_packet::AiPacketCommandArgs),
    Artifact(commands::artifact::ArtifactCommandArgs),
    BundleV2(commands::bundle_v2::BundleV2CommandArgs),
    Campaign(commands::campaign::CampaignCommandArgs),
    Compare(commands::compare::CompareCommandArgs),
    Config(commands::config::ConfigCommandArgs),
    Dashboard(commands::dashboard::DashboardCommandArgs),
    DockGraph(commands::dock_graph::DockGraphCommandArgs),
    DockRouting(commands::dock_routing::DockRoutingCommandArgs),
    Doctor(commands::doctor::DoctorCommandArgs),
    Extensions(commands::extensions::ExtensionsCommandArgs),
    FramesIndex(commands::frames_index::FramesIndexCommandArgs),
    Hotspots(commands::hotspots::HotspotsCommandArgs),
    Index(commands::index::IndexCommandArgs),
    Inspect(commands::inspect::InspectCommandArgs),
    Latest(commands::latest::LatestCommandArgs),
    LayoutSidecar(commands::layout_sidecar::LayoutSidecarCommandArgs),
    LayoutPerfSummary(commands::layout_perf_summary::LayoutPerfSummaryCommandArgs),
    Lint(commands::lint::LintCommandArgs),
    List(commands::list::ListCommandArgs),
    MemorySummary(commands::memory_summary::MemorySummaryCommandArgs),
    Meta(commands::meta::MetaCommandArgs),
    Matrix(commands::matrix::MatrixCommandArgs),
    Pack(commands::pack::PackCommandArgs),
    Path(commands::path::PathCommandArgs),
    Perf(commands::perf::PerfCommandArgs),
    PerfBaselineFromBundles(
        commands::perf_baseline_from_bundles::PerfBaselineFromBundlesCommandArgs,
    ),
    Poke(commands::poke::PokeCommandArgs),
    Pick(commands::pick::PickCommandArgs),
    PickApply(commands::pick_apply::PickApplyCommandArgs),
    PickArm(commands::pick_arm::PickArmCommandArgs),
    PickScript(commands::pick_script::PickScriptCommandArgs),
    Query(commands::query::QueryCommandArgs),
    Registry(commands::registry::RegistryCommandArgs),
    Resolve(commands::resolve::ResolveCommandArgs),
    Repro(commands::repro::ReproCommandArgs),
    Run(commands::run::RunCommandArgs),
    Repeat(commands::repeat::RepeatCommandArgs),
    Screenshots(commands::screenshots::ScreenshotsCommandArgs),
    Script(commands::script::ScriptCommandArgs),
    Sessions(commands::sessions::SessionsCommandArgs),
    Slice(commands::slice::SliceCommandArgs),
    Stats(commands::stats::StatsCommandArgs),
    Summarize(commands::summarize::SummarizeCommandArgs),
    Suite(commands::suite::SuiteCommandArgs),
    TestIds(commands::test_ids::TestIdsCommandArgs),
    TestIdsIndex(commands::test_ids_index::TestIdsIndexCommandArgs),
    Trace(commands::trace::TraceCommandArgs),
    Triage(commands::triage::TriageCommandArgs),
    Windows(commands::windows::WindowsCommandArgs),
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn try_parse_contract<I, T>(args: I) -> Result<DiagCliContract, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    DiagCliContract::try_parse_from(args)
}

#[cfg_attr(not(test), allow(dead_code))]
fn render_command_help(command_name: &str) -> Result<String, String> {
    render_command_help_path(&[command_name])
}

fn render_command_help_path(path: &[&str]) -> Result<String, String> {
    let mut cmd = DiagCliContract::command();
    let mut current = &mut cmd;
    for segment in path {
        current = current
            .find_subcommand_mut(segment)
            .ok_or_else(|| format!("missing clap help for diag {}", path.join(" ")))?;
    }
    let mut renderable = current.clone().bin_name(full_diag_bin_name(path));
    let mut out = Vec::new();
    renderable
        .write_long_help(&mut out)
        .map_err(|err| err.to_string())?;
    String::from_utf8(out).map_err(|err| err.to_string())
}

fn full_diag_bin_name(path: &[&str]) -> String {
    if path.is_empty() {
        "fretboard diag".to_string()
    } else {
        format!("fretboard diag {}", path.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use clap::error::ErrorKind;

    use super::{
        DiagCommandContract,
        commands::{
            agent::AgentCommandArgs,
            artifact::ArtifactSubcommandArgs,
            campaign::CampaignSubcommandArgs,
            config::ConfigSubcommandArgs,
            doctor::DoctorSubcommandArgs,
            latest::LatestCommandArgs,
            list::ListSubcommandArgs,
            path::PathCommandArgs,
            poke::PokeCommandArgs,
            query::{QuerySubcommandArgs, resolve_query_test_id_inputs},
            registry::RegistrySubcommandArgs,
            resolve::ResolveSubcommandArgs,
            script::ScriptSubcommandArgs,
            sessions::SessionsSubcommandArgs,
        },
        render_command_help, render_command_help_path, try_parse_contract,
    };

    #[test]
    fn run_contract_requires_a_script_argument() {
        let err =
            try_parse_contract(["fretboard", "run"]).expect_err("run should require a script");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn agent_contract_captures_source_warmup_and_output_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "agent",
            "target/fret-diag/demo",
            "--warmup-frames",
            "7",
            "--json",
            "--out",
            "target/agent.plan.json",
        ])
        .expect("agent should parse");

        let DiagCommandContract::Agent(AgentCommandArgs {
            source,
            warmup,
            output,
        }) = cli.command
        else {
            panic!("expected agent command");
        };

        assert_eq!(source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(warmup.warmup_frames, 7);
        assert!(output.json);
        assert_eq!(output.out, Some(PathBuf::from("target/agent.plan.json")));
    }

    #[test]
    fn suite_contract_accepts_script_dir_without_a_positional_suite_name() {
        let cli = try_parse_contract([
            "fretboard",
            "suite",
            "--script-dir",
            "tools/diag-scripts/ui-gallery/data_table",
            "--timeout-ms",
            "1",
            "--poll-ms",
            "1",
        ])
        .expect("suite contract should accept script-dir only invocations");

        let DiagCommandContract::Suite(args) = cli.command else {
            panic!("expected suite command");
        };

        assert_eq!(args.suite.as_deref(), None);
        assert_eq!(
            args.script_dirs,
            vec![PathBuf::from("tools/diag-scripts/ui-gallery/data_table")]
        );
        assert_eq!(args.timing.timeout_ms, 1);
        assert_eq!(args.timing.poll_ms, 1);
    }

    #[test]
    fn suite_contract_captures_launch_command_after_double_dash() {
        let cli = try_parse_contract([
            "fretboard",
            "suite",
            "ui-gallery",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
            "--release",
        ])
        .expect("suite contract should parse launch commands");

        let DiagCommandContract::Suite(args) = cli.command else {
            panic!("expected suite command");
        };

        assert_eq!(args.suite.as_deref(), Some("ui-gallery"));
        assert_eq!(
            args.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
                "--release".to_string(),
            ])
        );
    }

    #[test]
    fn repeat_contract_requires_a_script_argument() {
        let err = try_parse_contract(["fretboard", "repeat"])
            .expect_err("repeat should require a script");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn repeat_contract_rejects_zero_repeat_count() {
        let err = try_parse_contract(["fretboard", "repeat", "demo.json", "--repeat", "0"])
            .expect_err("repeat=0 should be rejected");
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn run_contract_rejects_incomplete_devtools_transport_args() {
        let missing_token = try_parse_contract([
            "fretboard",
            "run",
            "demo.json",
            "--devtools-ws-url",
            "ws://127.0.0.1:7331/",
        ])
        .expect_err("devtools ws url should require a token");
        assert_eq!(missing_token.kind(), ErrorKind::MissingRequiredArgument);

        let missing_ws = try_parse_contract([
            "fretboard",
            "run",
            "demo.json",
            "--devtools-token",
            "secret",
        ])
        .expect_err("devtools token should require a ws url");
        assert_eq!(missing_ws.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn suite_contract_rejects_session_id_without_devtools_ws_url() {
        let err = try_parse_contract([
            "fretboard",
            "suite",
            "ui-gallery",
            "--devtools-session-id",
            "session-1",
        ])
        .expect_err("devtools session id should require a ws url");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn perf_contract_requires_at_least_one_target() {
        let err =
            try_parse_contract(["fretboard", "perf"]).expect_err("perf should require a target");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn perf_contract_captures_threshold_and_suite_args() {
        let cli = try_parse_contract([
            "fretboard",
            "perf",
            "ui-gallery",
            "--top",
            "7",
            "--sort",
            "time",
            "--repeat",
            "11",
            "--prewarm-script",
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json",
            "--prelude-script",
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json",
            "--prelude-each-run",
            "--perf-threshold-agg",
            "p95",
            "--max-frame-p95-total-us",
            "18000",
            "--check-perf-hints",
            "--check-perf-hints-min-severity",
            "error",
            "--check-perf-hints-deny",
            "gpu.over-budget,input.hitch",
            "--check-pixels-unchanged",
            "ui-gallery-root",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
        ])
        .expect("perf contract should parse the supported subset");

        let DiagCommandContract::Perf(args) = cli.command else {
            panic!("expected perf command");
        };

        assert_eq!(args.targets, vec!["ui-gallery".to_string()]);
        assert_eq!(args.top, 7);
        assert_eq!(args.sort, Some(crate::BundleStatsSort::Time));
        assert_eq!(args.repeat, 11);
        assert!(args.prelude_each_run);
        assert_eq!(
            args.prewarm_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
            )]
        );
        assert_eq!(
            args.prelude_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json"
            )]
        );
        assert_eq!(
            args.perf_threshold_agg,
            Some(crate::PerfThresholdAggregate::P95)
        );
        assert_eq!(args.max_frame_p95_total_us, Some(18_000));
        assert!(args.check_perf_hints);
        assert_eq!(args.check_perf_hints_min_severity.as_deref(), Some("error"));
        assert_eq!(
            args.check_perf_hints_deny,
            vec!["gpu.over-budget,input.hitch".to_string()]
        );
        assert_eq!(
            args.check_pixels_unchanged.as_deref(),
            Some("ui-gallery-root")
        );
        assert_eq!(
            args.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
    }

    #[test]
    fn perf_baseline_from_bundles_contract_requires_an_output_path() {
        let err = try_parse_contract([
            "fretboard",
            "perf-baseline-from-bundles",
            "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json",
            "target/fret-diag/demo",
        ])
        .expect_err("perf-baseline-from-bundles should require --perf-baseline-out");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn perf_baseline_from_bundles_contract_captures_script_bundle_and_threshold_args() {
        let cli = try_parse_contract([
            "fretboard",
            "perf-baseline-from-bundles",
            "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json",
            "target/fret-diag/demo-a",
            "target/fret-diag/demo-b",
            "--sort",
            "time",
            "--perf-baseline-out",
            "target/perf.baseline.json",
            "--perf-baseline-headroom-pct",
            "25",
            "--warmup-frames",
            "5",
            "--json",
        ])
        .expect("perf-baseline-from-bundles should parse");

        let DiagCommandContract::PerfBaselineFromBundles(args) = cli.command else {
            panic!("expected perf-baseline-from-bundles command");
        };

        assert_eq!(
            args.script,
            "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json"
        );
        assert_eq!(
            args.bundle_artifacts,
            vec![
                "target/fret-diag/demo-a".to_string(),
                "target/fret-diag/demo-b".to_string(),
            ]
        );
        assert_eq!(args.sort, Some(crate::BundleStatsSort::Time));
        assert_eq!(
            args.perf_baseline_out,
            PathBuf::from("target/perf.baseline.json")
        );
        assert_eq!(args.perf_baseline_headroom_pct, 25);
        assert_eq!(args.warmup.warmup_frames, 5);
        assert!(args.json);
    }

    #[test]
    fn matrix_contract_requires_launch_and_captures_supported_flags() {
        let err = try_parse_contract(["fretboard", "matrix", "ui-gallery"])
            .expect_err("matrix should require --launch");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);

        let cli = try_parse_contract([
            "fretboard",
            "matrix",
            "ui-gallery",
            "--dir",
            "target/fret-diag-matrix",
            "--timeout-ms",
            "9",
            "--poll-ms",
            "3",
            "--warmup-frames",
            "5",
            "--compare-ignore-bounds",
            "--check-view-cache-reuse-min",
            "1",
            "--check-overlay-synthesis-min",
            "2",
            "--check-viewport-input-min",
            "3",
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
            "--json",
            "--launch-high-priority",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
            "--release",
        ])
        .expect("matrix should parse the supported subset");

        let DiagCommandContract::Matrix(args) = cli.command else {
            panic!("expected matrix command");
        };

        assert_eq!(
            args.target,
            super::commands::matrix::MatrixTargetArg::UiGallery
        );
        assert_eq!(
            args.output.dir,
            Some(PathBuf::from("target/fret-diag-matrix"))
        );
        assert_eq!(args.timing.timeout_ms, 9);
        assert_eq!(args.timing.poll_ms, 3);
        assert_eq!(args.timing.warmup_frames, 5);
        assert!(args.compare.compare_ignore_bounds);
        assert_eq!(args.check_view_cache_reuse_min, Some(1));
        assert_eq!(args.check_overlay_synthesis_min, Some(2));
        assert_eq!(args.check_viewport_input_min, Some(3));
        assert_eq!(
            args.env,
            vec!["FRET_UI_GALLERY_VIEW_CACHE_SHELL=1".to_string()]
        );
        assert!(args.launch_high_priority);
        assert_eq!(
            args.normalized_launch_argv(),
            vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
                "--release".to_string(),
            ]
        );
        assert!(args.output.json);
    }

    #[test]
    fn campaign_run_contract_captures_filters_and_suite_args() {
        let cli = try_parse_contract([
            "fretboard",
            "campaign",
            "run",
            "ui-gallery-smoke",
            "--lane",
            "smoke",
            "--tag",
            "ui-gallery",
            "--platform",
            "native",
            "--script-dir",
            "tools/diag-scripts/ui-gallery/data_table",
            "--glob",
            "tools/diag-scripts/ui-gallery-select-*.json",
            "--prewarm-script",
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json",
            "--prelude-script",
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json",
            "--prelude-each-run",
            "--pack",
            "--include-screenshots",
            "--top",
            "9",
            "--json",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
        ])
        .expect("campaign run contract should parse the supported subset");

        let DiagCommandContract::Campaign(args) = cli.command else {
            panic!("expected campaign command");
        };
        let CampaignSubcommandArgs::Run(run) = args.command else {
            panic!("expected campaign run command");
        };

        assert_eq!(run.campaign_ids, vec!["ui-gallery-smoke".to_string()]);
        assert_eq!(
            run.filters.lane,
            Some(crate::regression_summary::RegressionLaneV1::Smoke)
        );
        assert_eq!(run.filters.tags, vec!["ui-gallery".to_string()]);
        assert_eq!(run.filters.platforms, vec!["native".to_string()]);
        assert_eq!(run.top, 9);
        assert!(run.output.json);
        assert!(run.prelude_each_run);
        assert!(run.pack);
        assert!(run.include_screenshots);
        assert_eq!(
            run.script_dirs,
            vec![PathBuf::from("tools/diag-scripts/ui-gallery/data_table")]
        );
        assert_eq!(
            run.globs,
            vec!["tools/diag-scripts/ui-gallery-select-*.json".to_string()]
        );
        assert_eq!(
            run.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
    }

    #[test]
    fn campaign_share_contract_requires_a_source_argument() {
        let err = try_parse_contract(["fretboard", "campaign", "share"])
            .expect_err("campaign share should require a source");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn campaign_validate_contract_accepts_zero_or_more_manifests() {
        let cli = try_parse_contract(["fretboard", "campaign", "validate", "--json"])
            .expect("campaign validate should accept zero manifests");

        let DiagCommandContract::Campaign(args) = cli.command else {
            panic!("expected campaign command");
        };
        let CampaignSubcommandArgs::Validate(validate) = args.command else {
            panic!("expected campaign validate command");
        };
        assert!(validate.manifests.is_empty());
        assert!(validate.json);
    }

    #[test]
    fn list_scripts_contract_captures_filters_and_top() {
        let cli = try_parse_contract([
            "fretboard",
            "list",
            "scripts",
            "--contains",
            "ui-gallery",
            "--case-sensitive",
            "--all",
            "--top",
            "9",
            "--json",
        ])
        .expect("list scripts should parse the supported subset");

        let DiagCommandContract::List(args) = cli.command else {
            panic!("expected list command");
        };
        let ListSubcommandArgs::Scripts(scripts) = args.command else {
            panic!("expected list scripts command");
        };

        assert_eq!(scripts.filters.contains.as_deref(), Some("ui-gallery"));
        assert!(scripts.filters.case_sensitive);
        assert!(scripts.filters.all);
        assert_eq!(scripts.top, Some(9));
        assert!(scripts.json);
    }

    #[test]
    fn list_sessions_contract_accepts_dir_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "list",
            "sessions",
            "--dir",
            "target/fret-diag-test",
            "--contains",
            "smoke",
            "--json",
        ])
        .expect("list sessions should parse a session-specific out dir");

        let DiagCommandContract::List(args) = cli.command else {
            panic!("expected list command");
        };
        let ListSubcommandArgs::Sessions(sessions) = args.command else {
            panic!("expected list sessions command");
        };

        assert_eq!(sessions.dir, Some(PathBuf::from("target/fret-diag-test")));
        assert_eq!(sessions.filters.contains.as_deref(), Some("smoke"));
        assert!(sessions.json);
    }

    #[test]
    fn sessions_clean_contract_requires_a_selection_criterion() {
        let err = try_parse_contract(["fretboard", "sessions", "clean"])
            .expect_err("sessions clean should require --keep and/or --older-than-days");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn sessions_clean_contract_captures_cleanup_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "sessions",
            "clean",
            "--dir",
            "target/fret-diag-clean",
            "--keep",
            "10",
            "--older-than-days",
            "14",
            "--top",
            "50",
            "--apply",
            "--json",
        ])
        .expect("sessions clean should parse");

        let DiagCommandContract::Sessions(args) = cli.command else {
            panic!("expected sessions command");
        };
        let SessionsSubcommandArgs::Clean(clean) = args.command;

        assert_eq!(clean.dir, Some(PathBuf::from("target/fret-diag-clean")));
        assert_eq!(clean.keep, Some(10));
        assert_eq!(clean.older_than_days, Some(14));
        assert_eq!(clean.top, Some(50));
        assert!(clean.apply);
        assert!(clean.json);
    }

    #[test]
    fn doctor_contract_captures_bundle_flags_and_aliases() {
        let cli = try_parse_contract([
            "fretboard",
            "doctor",
            "target/fret-diag/session-1",
            "--dir",
            "target/fret-diag-root",
            "--warmup-frames",
            "4",
            "--fix-plan",
            "--fix-schema2",
            "--fix-bundle-json",
            "--fix-sidecars",
            "--check-required",
            "--json",
        ])
        .expect("doctor bundle command should parse the supported subset");

        let DiagCommandContract::Doctor(args) = cli.command else {
            panic!("expected doctor command");
        };

        assert!(args.command.is_none());
        assert_eq!(args.source.as_deref(), Some("target/fret-diag/session-1"));
        assert_eq!(args.dir, Some(PathBuf::from("target/fret-diag-root")));
        assert_eq!(args.warmup_frames, 4);
        assert!(args.fix_dry_run);
        assert!(args.fix_schema2);
        assert!(args.fix_bundle_json);
        assert!(args.fix_sidecars);
        assert!(args.check_required);
        assert!(args.json);
    }

    #[test]
    fn doctor_scripts_contract_accepts_max_examples_alias() {
        let cli = try_parse_contract([
            "fretboard",
            "doctor",
            "scripts",
            "--top",
            "7",
            "--strict",
            "--json",
        ])
        .expect("doctor scripts should parse nested args and aliases");

        let DiagCommandContract::Doctor(args) = cli.command else {
            panic!("expected doctor command");
        };
        let Some(DoctorSubcommandArgs::Scripts(scripts)) = args.command else {
            panic!("expected doctor scripts command");
        };

        assert_eq!(scripts.max_examples, 7);
        assert!(scripts.strict);
        assert!(scripts.json);
    }

    #[test]
    fn doctor_campaigns_contract_accepts_strict_json() {
        let cli = try_parse_contract(["fretboard", "doctor", "campaigns", "--strict", "--json"])
            .expect("doctor campaigns should parse nested args");

        let DiagCommandContract::Doctor(args) = cli.command else {
            panic!("expected doctor command");
        };
        let Some(DoctorSubcommandArgs::Campaigns(campaigns)) = args.command else {
            panic!("expected doctor campaigns command");
        };

        assert!(campaigns.strict);
        assert!(campaigns.json);
    }

    #[test]
    fn config_doctor_contract_captures_launch_report_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "config",
            "doctor",
            "--dir",
            "target/fret-diag-config",
            "--env",
            "FRET_DIAG_MAX_SNAPSHOTS=50",
            "--mode",
            "manual",
            "--config-path",
            "tools/diag-configs/diag.config.example.json",
            "--show-env",
            "all",
            "--report-json",
            "--print-launch-policy",
        ])
        .expect("config doctor should parse");

        let DiagCommandContract::Config(args) = cli.command else {
            panic!("expected config command");
        };
        let ConfigSubcommandArgs::Doctor(doctor) = args.command;

        assert_eq!(doctor.dir, Some(PathBuf::from("target/fret-diag-config")));
        assert_eq!(doctor.env, vec!["FRET_DIAG_MAX_SNAPSHOTS=50".to_string()]);
        assert_eq!(
            doctor.mode,
            Some(super::commands::config::ConfigDoctorModeArg::Manual)
        );
        assert_eq!(
            doctor.config_path,
            Some(PathBuf::from("tools/diag-configs/diag.config.example.json"))
        );
        assert_eq!(
            doctor.show_env,
            Some(super::commands::config::ConfigShowEnvArg::All)
        );
        assert!(doctor.report_json);
        assert!(doctor.print_launch_policy);
    }

    #[test]
    fn registry_contract_captures_action_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "registry",
            "write",
            "--path",
            "target/diag.index.json",
            "--json",
        ])
        .expect("registry write should parse");

        let DiagCommandContract::Registry(args) = cli.command else {
            panic!("expected registry command");
        };
        let RegistrySubcommandArgs::Write(write) = args.command else {
            panic!("expected registry write command");
        };

        assert_eq!(write.path, Some(PathBuf::from("target/diag.index.json")));
        assert!(write.json);
    }

    #[test]
    fn path_contract_captures_dir_and_trigger_override() {
        let cli = try_parse_contract([
            "fretboard",
            "path",
            "--dir",
            "target/fret-diag-path",
            "--trigger-path",
            "target/fret-diag-path/custom.trigger",
        ])
        .expect("path should parse");

        let DiagCommandContract::Path(PathCommandArgs { dir, trigger_path }) = cli.command else {
            panic!("expected path command");
        };

        assert_eq!(dir, Some(PathBuf::from("target/fret-diag-path")));
        assert_eq!(
            trigger_path,
            Some(PathBuf::from("target/fret-diag-path/custom.trigger"))
        );
    }

    #[test]
    fn poke_contract_captures_dump_request_and_wait_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "poke",
            "--dir",
            "target/fret-diag-poke",
            "--trigger-path",
            "target/fret-diag-poke/custom.trigger",
            "--label",
            "manual-dump",
            "--max-snapshots",
            "8",
            "--request-id",
            "42",
            "--wait",
            "--record-run",
            "--run-id",
            "99",
            "--timeout-ms",
            "9",
            "--poll-ms",
            "3",
        ])
        .expect("poke should parse");

        let DiagCommandContract::Poke(PokeCommandArgs {
            dir,
            trigger_path,
            label,
            max_snapshots,
            request_id,
            wait,
            record_run,
            run_id,
            wait_args,
        }) = cli.command
        else {
            panic!("expected poke command");
        };

        assert_eq!(dir, Some(PathBuf::from("target/fret-diag-poke")));
        assert_eq!(
            trigger_path,
            Some(PathBuf::from("target/fret-diag-poke/custom.trigger"))
        );
        assert_eq!(label.as_deref(), Some("manual-dump"));
        assert_eq!(max_snapshots, Some(8));
        assert_eq!(request_id, Some(42));
        assert!(wait);
        assert!(record_run);
        assert_eq!(run_id, Some(99));
        assert_eq!(wait_args.timeout_ms, 9);
        assert_eq!(wait_args.poll_ms, 3);
    }

    #[test]
    fn latest_contract_captures_dir_override() {
        let cli = try_parse_contract(["fretboard", "latest", "--dir", "target/fret-diag-latest"])
            .expect("latest should parse");

        let DiagCommandContract::Latest(LatestCommandArgs { dir }) = cli.command else {
            panic!("expected latest command");
        };

        assert_eq!(dir, Some(PathBuf::from("target/fret-diag-latest")));
    }

    #[test]
    fn artifact_lint_contract_accepts_optional_target_and_output_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "artifact",
            "lint",
            "target/fret-diag/42",
            "--warmup-frames",
            "3",
            "--json",
            "--out",
            "target/artifact.lint.json",
        ])
        .expect("artifact lint should parse");

        let DiagCommandContract::Artifact(args) = cli.command else {
            panic!("expected artifact command");
        };
        let ArtifactSubcommandArgs::Lint(lint) = args.command;

        assert_eq!(lint.source.as_deref(), Some("target/fret-diag/42"));
        assert_eq!(lint.warmup.warmup_frames, 3);
        assert!(lint.output.json);
        assert_eq!(
            lint.output.out,
            Some(PathBuf::from("target/artifact.lint.json"))
        );
    }

    #[test]
    fn lint_contract_captures_bounds_flags_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "lint",
            "target/fret-diag/demo",
            "--warmup-frames",
            "3",
            "--all-test-ids",
            "--lint-eps-px",
            "1.25",
            "--json",
            "--out",
            "target/bundle.lint.json",
        ])
        .expect("lint should parse");

        let DiagCommandContract::Lint(args) = cli.command else {
            panic!("expected lint command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 3);
        assert!(args.all_test_ids);
        assert_eq!(args.lint_eps_px, 1.25);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/bundle.lint.json"))
        );
    }

    #[test]
    fn pack_contract_captures_canonical_pack_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "pack",
            "target/fret-diag/demo",
            "--dir",
            "target/fret-diag-root",
            "--warmup-frames",
            "4",
            "--pack-out",
            "target/demo.zip",
            "--ai-packet",
            "--include-all",
            "--pack-schema2-only",
        ])
        .expect("pack should parse");

        let DiagCommandContract::Pack(args) = cli.command else {
            panic!("expected pack command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.dir, Some(PathBuf::from("target/fret-diag-root")));
        assert_eq!(args.warmup.warmup_frames, 4);
        assert_eq!(args.pack_out, Some(PathBuf::from("target/demo.zip")));
        assert!(args.ai_packet);
        assert!(args.include_all);
        assert!(args.pack_schema2_only);
    }

    #[test]
    fn triage_contract_captures_lite_metric_sort_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "triage",
            "target/fret-diag/demo",
            "--warmup-frames",
            "5",
            "--top",
            "8",
            "--sort",
            "time",
            "--lite",
            "--metric",
            "layout",
            "--json",
            "--out",
            "target/triage.lite.json",
        ])
        .expect("triage should parse");

        let DiagCommandContract::Triage(args) = cli.command else {
            panic!("expected triage command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 5);
        assert_eq!(args.top, 8);
        assert_eq!(args.sort, Some(crate::BundleStatsSort::Time));
        assert!(args.lite);
        assert!(matches!(
            args.metric,
            Some(crate::frames_index::TriageLiteMetric::LayoutTimeUs)
        ));
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/triage.lite.json"))
        );
    }

    #[test]
    fn windows_contract_captures_warmup_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "windows",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--json",
        ])
        .expect("windows should parse the supported subset");

        let DiagCommandContract::Windows(args) = cli.command else {
            panic!("expected windows command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.json);
    }

    #[test]
    fn dock_routing_contract_captures_warmup_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "dock-routing",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--json",
        ])
        .expect("dock-routing should parse the supported subset");

        let DiagCommandContract::DockRouting(args) = cli.command else {
            panic!("expected dock-routing command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.json);
    }

    #[test]
    fn dock_graph_contract_captures_json() {
        let cli =
            try_parse_contract(["fretboard", "dock-graph", "target/fret-diag/demo", "--json"])
                .expect("dock-graph should parse the supported subset");

        let DiagCommandContract::DockGraph(args) = cli.command else {
            panic!("expected dock-graph command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert!(args.json);
    }

    #[test]
    fn screenshots_contract_captures_json() {
        let cli = try_parse_contract([
            "fretboard",
            "screenshots",
            "target/fret-diag/demo",
            "--json",
        ])
        .expect("screenshots should parse the supported subset");

        let DiagCommandContract::Screenshots(args) = cli.command else {
            panic!("expected screenshots command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert!(args.json);
    }

    #[test]
    fn hotspots_contract_captures_lite_metric_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "hotspots",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--hotspots-top",
            "9",
            "--max-depth",
            "5",
            "--min-bytes",
            "4096",
            "--force",
            "--lite",
            "--metric",
            "paint",
            "--json",
            "--out",
            "target/hotspots.json",
        ])
        .expect("hotspots should parse the supported subset");

        let DiagCommandContract::Hotspots(args) = cli.command else {
            panic!("expected hotspots command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.warmup.warmup_frames, 4);
        assert_eq!(args.hotspots_top, 9);
        assert_eq!(args.max_depth, 5);
        assert_eq!(args.min_bytes, 4096);
        assert!(args.force);
        assert!(args.lite);
        assert!(matches!(
            args.metric,
            Some(crate::frames_index::TriageLiteMetric::PaintTimeUs)
        ));
        assert!(args.output.json);
        assert_eq!(args.output.out, Some(PathBuf::from("target/hotspots.json")));
    }

    #[test]
    fn bundle_v2_contract_captures_mode_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "bundle-v2",
            "target/fret-diag/demo",
            "--mode",
            "changed",
            "--pretty",
            "--force",
            "--json",
            "--out",
            "target/bundle.schema2.json",
        ])
        .expect("bundle-v2 should parse the supported subset");

        let DiagCommandContract::BundleV2(args) = cli.command else {
            panic!("expected bundle-v2 command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.mode, "changed");
        assert!(args.pretty);
        assert!(args.force);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/bundle.schema2.json"))
        );
    }

    #[test]
    fn compare_contract_captures_footprint_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "compare",
            "target/fret-diag/demo-a",
            "target/fret-diag/demo-b",
            "--footprint",
            "--warmup-frames",
            "4",
            "--compare-eps-px",
            "0.25",
            "--compare-ignore-bounds",
            "--compare-ignore-scene-fingerprint",
            "--json",
        ])
        .expect("compare should parse the supported subset");

        let DiagCommandContract::Compare(args) = cli.command else {
            panic!("expected compare command");
        };

        assert_eq!(args.source_a, "target/fret-diag/demo-a");
        assert_eq!(args.source_b, "target/fret-diag/demo-b");
        assert!(args.footprint);
        assert_eq!(args.warmup.warmup_frames, 4);
        assert_eq!(args.compare.compare_eps_px, 0.25);
        assert!(args.compare.compare_ignore_bounds);
        assert!(args.compare.compare_ignore_scene_fingerprint);
        assert!(args.json);
    }

    #[test]
    fn dashboard_contract_accepts_optional_source_top_and_dir() {
        let cli = try_parse_contract([
            "fretboard",
            "dashboard",
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270",
            "--dir",
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270",
            "--top",
            "9",
            "--json",
        ])
        .expect("dashboard should parse the supported subset");

        let DiagCommandContract::Dashboard(args) = cli.command else {
            panic!("expected dashboard command");
        };

        assert_eq!(
            args.source,
            Some(PathBuf::from(
                "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270"
            ))
        );
        assert_eq!(
            args.output.dir,
            Some(PathBuf::from(
                "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270"
            ))
        );
        assert_eq!(args.top, 9);
        assert!(args.output.json);
    }

    #[test]
    fn summarize_contract_accepts_zero_or_more_inputs_and_dir() {
        let cli = try_parse_contract([
            "fretboard",
            "summarize",
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270",
            "target/fret-diag/regression.summary.json",
            "--dir",
            "target/fret-diag-clap-smoke/summarize",
            "--json",
        ])
        .expect("summarize should parse the supported subset");

        let DiagCommandContract::Summarize(args) = cli.command else {
            panic!("expected summarize command");
        };

        assert_eq!(
            args.inputs,
            vec![
                PathBuf::from("target/fret-diag/campaigns/ui-gallery-smoke/1774499171270"),
                PathBuf::from("target/fret-diag/regression.summary.json"),
            ]
        );
        assert_eq!(
            args.output.dir,
            Some(PathBuf::from("target/fret-diag-clap-smoke/summarize"))
        );
        assert!(args.output.json);
    }

    #[test]
    fn stats_contract_captures_supported_checks_and_diff_mode() {
        let cli = try_parse_contract([
            "fretboard",
            "stats",
            "--diff",
            "target/fret-diag/demo-a",
            "target/fret-diag/demo-b",
            "--top",
            "9",
            "--sort",
            "time",
            "--verbose",
            "--warmup-frames",
            "4",
            "--json",
            "--check-stale-paint",
            "ui-gallery-root",
            "--check-stale-paint-eps",
            "0.25",
            "--check-wheel-scroll",
            "ui-gallery-root",
            "--check-hover-layout-max",
            "3",
            "--check-notify-hotspot-file-max",
            "src/view.rs",
            "7",
            "--check-view-cache-reuse-min",
            "1",
            "--check-retained-vlist-attach-detach-max",
            "2",
        ])
        .expect("stats should parse the supported subset");

        let DiagCommandContract::Stats(args) = cli.command else {
            panic!("expected stats command");
        };

        assert_eq!(
            args.diff,
            Some(vec![
                PathBuf::from("target/fret-diag/demo-a"),
                PathBuf::from("target/fret-diag/demo-b"),
            ])
        );
        assert_eq!(args.top, 9);
        assert_eq!(args.sort, Some(crate::BundleStatsSort::Time));
        assert!(args.verbose);
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.json);
        assert_eq!(
            args.checks.common.check_stale_paint.as_deref(),
            Some("ui-gallery-root")
        );
        assert_eq!(args.checks.common.check_stale_paint_eps, 0.25);
        assert_eq!(
            args.checks.check_wheel_scroll.as_deref(),
            Some("ui-gallery-root")
        );
        assert_eq!(args.checks.check_hover_layout_max, Some(3));
        assert_eq!(
            args.checks.check_notify_hotspot_file_max,
            vec!["src/view.rs".to_string(), "7".to_string()]
        );
        assert_eq!(args.checks.check_view_cache_reuse_min, Some(1));
        assert_eq!(args.checks.check_retained_vlist_attach_detach_max, Some(2));
    }

    #[test]
    fn stats_contract_accepts_stats_lite_checks_json_without_a_source() {
        let cli = try_parse_contract(["fretboard", "stats", "--stats-lite-checks-json"])
            .expect("stats-lite checks json mode should parse");

        let DiagCommandContract::Stats(args) = cli.command else {
            panic!("expected stats command");
        };

        assert!(args.stats_lite_checks_json);
        assert!(args.source.is_none());
        assert!(args.diff.is_none());
    }

    #[test]
    fn stats_contract_rejects_conflicting_source_and_diff_modes() {
        let err = try_parse_contract([
            "fretboard",
            "stats",
            "target/fret-diag/demo",
            "--diff",
            "target/fret-diag/demo-a",
            "target/fret-diag/demo-b",
        ])
        .expect_err("stats should reject conflicting source and diff modes");
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn trace_contract_requires_a_source_argument() {
        let err =
            try_parse_contract(["fretboard", "trace"]).expect_err("trace should require a source");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn trace_contract_captures_trace_out() {
        let cli = try_parse_contract([
            "fretboard",
            "trace",
            "target/fret-diag/demo",
            "--trace-out",
            "target/trace.chrome.json",
        ])
        .expect("trace contract should parse the supported subset");

        let DiagCommandContract::Trace(args) = cli.command else {
            panic!("expected trace command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(
            args.trace_out,
            Some(PathBuf::from("target/trace.chrome.json"))
        );
    }

    #[test]
    fn resolve_contract_requires_a_target() {
        let err = try_parse_contract(["fretboard", "resolve"])
            .expect_err("resolve should require a target");
        assert_eq!(
            err.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn resolve_latest_contract_captures_dir_session_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "resolve",
            "latest",
            "--dir",
            "target/fret-diag",
            "--within-session",
            "latest",
            "--json",
        ])
        .expect("resolve latest should parse the supported subset");

        let DiagCommandContract::Resolve(args) = cli.command else {
            panic!("expected resolve command");
        };
        let latest = match args.command {
            ResolveSubcommandArgs::Latest(latest) => latest,
        };

        assert_eq!(latest.dir, Some(PathBuf::from("target/fret-diag")));
        assert_eq!(latest.within_session.as_deref(), Some("latest"));
        assert!(latest.json);
    }

    #[test]
    fn test_ids_index_contract_captures_warmup_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "test-ids-index",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--json",
        ])
        .expect("test-ids-index should parse the supported subset");

        let DiagCommandContract::TestIdsIndex(args) = cli.command else {
            panic!("expected test-ids-index command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.json);
    }

    #[test]
    fn frames_index_contract_captures_warmup_and_json() {
        let cli = try_parse_contract([
            "fretboard",
            "frames-index",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--json",
        ])
        .expect("frames-index should parse the supported subset");

        let DiagCommandContract::FramesIndex(args) = cli.command else {
            panic!("expected frames-index command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.json);
    }

    #[test]
    fn ai_packet_contract_captures_explicit_test_id_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "ai-packet",
            "target/fret-diag/demo",
            "--warmup-frames",
            "6",
            "--packet-out",
            "target/ai.packet",
            "--test-id",
            "ui-gallery-root",
            "--sidecars-only",
            "--include-triage",
        ])
        .expect("ai-packet should parse");

        let DiagCommandContract::AiPacket(args) = cli.command else {
            panic!("expected ai-packet command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.warmup.warmup_frames, 6);
        assert_eq!(args.packet_out, Some(PathBuf::from("target/ai.packet")));
        assert_eq!(args.test_id.as_deref(), Some("ui-gallery-root"));
        assert!(args.sidecars_only);
        assert!(args.include_triage);
    }

    #[test]
    fn meta_contract_captures_report_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "meta",
            "target/fret-diag/demo",
            "--warmup-frames",
            "4",
            "--json",
            "--out",
            "target/bundle.meta.json",
            "--meta-report",
        ])
        .expect("meta should parse");

        let DiagCommandContract::Meta(args) = cli.command else {
            panic!("expected meta command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/bundle.meta.json"))
        );
        assert!(args.meta_report);
    }

    #[test]
    fn index_contract_captures_required_source_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "index",
            "target/fret-diag/demo",
            "--warmup-frames",
            "2",
            "--json",
            "--out",
            "target/bundle.index.json",
        ])
        .expect("index should parse");

        let DiagCommandContract::Index(args) = cli.command else {
            panic!("expected index command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 2);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/bundle.index.json"))
        );
    }

    #[test]
    fn test_ids_contract_captures_max_test_ids() {
        let cli = try_parse_contract([
            "fretboard",
            "test-ids",
            "target/fret-diag/demo",
            "--warmup-frames",
            "5",
            "--max-test-ids",
            "17",
            "--json",
            "--out",
            "target/test_ids.index.json",
        ])
        .expect("test-ids should parse");

        let DiagCommandContract::TestIds(args) = cli.command else {
            panic!("expected test-ids command");
        };

        assert_eq!(args.source, "target/fret-diag/demo");
        assert_eq!(args.warmup.warmup_frames, 5);
        assert_eq!(args.max_test_ids, 17);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/test_ids.index.json"))
        );
    }

    #[test]
    fn layout_sidecar_contract_accepts_optional_source_and_print() {
        let cli = try_parse_contract([
            "fretboard",
            "layout-sidecar",
            "target/fret-diag/demo",
            "--print",
            "--json",
            "--out",
            "target/layout.taffy.v1.json",
        ])
        .expect("layout-sidecar should parse");

        let DiagCommandContract::LayoutSidecar(args) = cli.command else {
            panic!("expected layout-sidecar command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert!(args.print);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/layout.taffy.v1.json"))
        );
    }

    #[test]
    fn layout_perf_summary_contract_accepts_source_and_output_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "layout-perf-summary",
            "target/fret-diag/demo",
            "--top",
            "7",
            "--warmup-frames",
            "4",
            "--json",
            "--out",
            "target/layout_perf_summary.json",
        ])
        .expect("layout-perf-summary should parse");

        let DiagCommandContract::LayoutPerfSummary(args) = cli.command else {
            panic!("expected layout-perf-summary command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.top, 7);
        assert_eq!(args.warmup.warmup_frames, 4);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/layout_perf_summary.json"))
        );
    }

    #[test]
    fn memory_summary_contract_accepts_aggregation_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "memory-summary",
            "target/fret-diag/demo",
            "--within-session",
            "latest",
            "--top-sessions",
            "3",
            "--sort-key",
            "renderer_gpu_images_bytes_estimate",
            "--fit-linear",
            "renderer_gpu_images_bytes_estimate:macos_physical_footprint_peak_bytes",
            "--top",
            "9",
            "--vmmap-regions-sorted-top",
            "--vmmap-regions-sorted-agg",
            "--vmmap-regions-sorted-agg-top",
            "11",
            "--vmmap-regions-sorted-detail-agg",
            "--vmmap-regions-sorted-detail-agg-top",
            "13",
            "--footprint-categories-agg",
            "--footprint-categories-agg-top",
            "15",
            "--no-recursive",
            "--max-depth",
            "4",
            "--max-samples",
            "120",
            "--json",
            "--out",
            "target/memory_summary.json",
        ])
        .expect("memory-summary should parse");

        let DiagCommandContract::MemorySummary(args) = cli.command else {
            panic!("expected memory-summary command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.within_session.as_deref(), Some("latest"));
        assert_eq!(args.top_sessions, Some(3));
        assert_eq!(args.sort_key, "renderer_gpu_images_bytes_estimate");
        assert_eq!(
            args.fit_linear,
            vec!["renderer_gpu_images_bytes_estimate:macos_physical_footprint_peak_bytes"]
        );
        assert_eq!(args.top, 9);
        assert!(args.vmmap_regions_sorted_top);
        assert!(args.vmmap_regions_sorted_agg);
        assert_eq!(args.vmmap_regions_sorted_agg_top, 11);
        assert!(args.vmmap_regions_sorted_detail_agg);
        assert_eq!(args.vmmap_regions_sorted_detail_agg_top, 13);
        assert!(args.footprint_categories_agg);
        assert_eq!(args.footprint_categories_agg_top, 15);
        assert!(args.no_recursive);
        assert_eq!(args.max_depth, 4);
        assert_eq!(args.max_samples, 120);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/memory_summary.json"))
        );
    }

    #[test]
    fn inspect_contract_accepts_action_and_consume_clicks() {
        let cli = try_parse_contract([
            "fretboard",
            "inspect",
            "toggle",
            "--consume-clicks",
            "false",
        ])
        .expect("inspect should parse");

        let DiagCommandContract::Inspect(args) = cli.command else {
            panic!("expected inspect command");
        };

        assert_eq!(args.action.as_str(), "toggle");
        assert_eq!(args.consume_clicks, Some(false));
    }

    #[test]
    fn pick_contract_accepts_wait_flags() {
        let cli = try_parse_contract(["fretboard", "pick", "--timeout-ms", "9", "--poll-ms", "3"])
            .expect("pick should parse");

        let DiagCommandContract::Pick(args) = cli.command else {
            panic!("expected pick command");
        };

        assert_eq!(args.wait.timeout_ms, 9);
        assert_eq!(args.wait.poll_ms, 3);
    }

    #[test]
    fn pick_script_contract_accepts_wait_flags_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "pick-script",
            "--timeout-ms",
            "9",
            "--poll-ms",
            "3",
            "--pick-script-out",
            "target/picked.script.json",
        ])
        .expect("pick-script should parse");

        let DiagCommandContract::PickScript(args) = cli.command else {
            panic!("expected pick-script command");
        };

        assert_eq!(args.wait.timeout_ms, 9);
        assert_eq!(args.wait.poll_ms, 3);
        assert_eq!(
            args.pick_script_out,
            Some(PathBuf::from("target/picked.script.json"))
        );
    }

    #[test]
    fn pick_apply_contract_requires_a_pointer() {
        let err = try_parse_contract(["fretboard", "pick-apply", "./script.json"])
            .expect_err("pick-apply should require --ptr");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn pick_apply_contract_accepts_script_pointer_output_and_wait() {
        let cli = try_parse_contract([
            "fretboard",
            "pick-apply",
            "./script.json",
            "--ptr",
            "/steps/0/target",
            "--out",
            "target/picked.json",
            "--timeout-ms",
            "9",
            "--poll-ms",
            "3",
        ])
        .expect("pick-apply should parse");

        let DiagCommandContract::PickApply(args) = cli.command else {
            panic!("expected pick-apply command");
        };

        assert_eq!(args.script, "./script.json");
        assert_eq!(args.ptr, "/steps/0/target");
        assert_eq!(args.out, Some(PathBuf::from("target/picked.json")));
        assert_eq!(args.wait.timeout_ms, 9);
        assert_eq!(args.wait.poll_ms, 3);
    }

    #[test]
    fn extensions_contract_accepts_key_print_and_output() {
        let cli = try_parse_contract([
            "fretboard",
            "extensions",
            "target/fret-diag/demo",
            "--key",
            "a.v1",
            "--print",
            "--warmup-frames",
            "6",
            "--json",
            "--out",
            "target/extensions.json",
        ])
        .expect("extensions should parse");

        let DiagCommandContract::Extensions(args) = cli.command else {
            panic!("expected extensions command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.key.as_deref(), Some("a.v1"));
        assert!(args.print);
        assert_eq!(args.warmup.warmup_frames, 6);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/extensions.json"))
        );
    }

    #[test]
    fn query_test_id_contract_captures_pattern_and_filters() {
        let cli = try_parse_contract([
            "fretboard",
            "query",
            "test-id",
            "target/fret-diag/demo",
            "ui-gallery",
            "--warmup-frames",
            "7",
            "--mode",
            "prefix",
            "--top",
            "12",
            "--case-sensitive",
            "--json",
            "--out",
            "target/query.test-id.json",
        ])
        .expect("query test-id should parse");

        let DiagCommandContract::Query(args) = cli.command else {
            panic!("expected query command");
        };
        let QuerySubcommandArgs::TestId(test_id) = args.command else {
            panic!("expected query test-id subcommand");
        };

        let resolved = resolve_query_test_id_inputs(&test_id.inputs, Path::new("."))
            .expect("query test-id inputs should resolve");

        assert_eq!(resolved.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(resolved.pattern, "ui-gallery");
        assert_eq!(test_id.warmup.warmup_frames, 7);
        assert_eq!(test_id.mode, "prefix");
        assert_eq!(test_id.top, 12);
        assert!(test_id.case_sensitive);
        assert!(test_id.output.json);
        assert_eq!(
            test_id.output.out,
            Some(PathBuf::from("target/query.test-id.json"))
        );
    }

    #[test]
    fn query_snapshots_contract_captures_index_first_filters() {
        let cli = try_parse_contract([
            "fretboard",
            "query",
            "snapshots",
            "target/fret-diag/demo",
            "--warmup-frames",
            "3",
            "--top",
            "9",
            "--window",
            "2",
            "--include-warmup",
            "--include-missing-semantics",
            "--semantics-source",
            "table",
            "--test-id",
            "ui-gallery-root",
            "--step-index",
            "11",
            "--json",
            "--out",
            "target/query.snapshots.json",
        ])
        .expect("query snapshots should parse");

        let DiagCommandContract::Query(args) = cli.command else {
            panic!("expected query command");
        };
        let QuerySubcommandArgs::Snapshots(snapshots) = args.command else {
            panic!("expected query snapshots subcommand");
        };

        assert_eq!(snapshots.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(snapshots.warmup.warmup_frames, 3);
        assert_eq!(snapshots.top, 9);
        assert_eq!(snapshots.window, Some(2));
        assert!(snapshots.include_warmup);
        assert!(snapshots.include_missing_semantics);
        assert_eq!(snapshots.semantics_source.as_deref(), Some("table"));
        assert_eq!(snapshots.test_id.as_deref(), Some("ui-gallery-root"));
        assert_eq!(snapshots.step_index, Some(11));
        assert!(snapshots.output.json);
        assert_eq!(
            snapshots.output.out,
            Some(PathBuf::from("target/query.snapshots.json"))
        );
    }

    #[test]
    fn query_overlay_placement_trace_contract_uses_canonical_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "query",
            "overlay-placement-trace",
            "target/fret-diag/demo",
            "--top",
            "8",
            "--kind",
            "anchored_panel",
            "--overlay-root-name",
            "overlay-root",
            "--anchor-test-id",
            "anchor",
            "--content-test-id",
            "content",
            "--preferred-side",
            "bottom",
            "--chosen-side",
            "top",
            "--flipped",
            "true",
            "--align",
            "center",
            "--sticky",
            "always",
            "--json",
            "--out",
            "target/query.overlay.json",
        ])
        .expect("query overlay-placement-trace should parse");

        let DiagCommandContract::Query(args) = cli.command else {
            panic!("expected query command");
        };
        let QuerySubcommandArgs::OverlayPlacementTrace(trace) = args.command else {
            panic!("expected query overlay-placement-trace subcommand");
        };

        assert_eq!(trace.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(trace.top, 8);
        assert_eq!(trace.kind.as_deref(), Some("anchored_panel"));
        assert_eq!(trace.overlay_root_name.as_deref(), Some("overlay-root"));
        assert_eq!(trace.anchor_test_id.as_deref(), Some("anchor"));
        assert_eq!(trace.content_test_id.as_deref(), Some("content"));
        assert_eq!(trace.preferred_side.as_deref(), Some("bottom"));
        assert_eq!(trace.chosen_side.as_deref(), Some("top"));
        assert_eq!(trace.flipped, Some(true));
        assert_eq!(trace.align.as_deref(), Some("center"));
        assert_eq!(trace.sticky.as_deref(), Some("always"));
        assert!(trace.output.json);
        assert_eq!(
            trace.output.out,
            Some(PathBuf::from("target/query.overlay.json"))
        );
    }

    #[test]
    fn query_scroll_extents_observation_contract_captures_flags() {
        let cli = try_parse_contract([
            "fretboard",
            "query",
            "scroll-extents-observation",
            "target/fret-diag/demo",
            "--warmup-frames",
            "5",
            "--window",
            "4",
            "--top",
            "16",
            "--all",
            "--deep-scan",
            "--timeline",
            "--json",
            "--out",
            "target/query.scroll.json",
        ])
        .expect("query scroll-extents-observation should parse");

        let DiagCommandContract::Query(args) = cli.command else {
            panic!("expected query command");
        };
        let QuerySubcommandArgs::ScrollExtentsObservation(scroll) = args.command else {
            panic!("expected query scroll-extents-observation subcommand");
        };

        assert_eq!(scroll.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(scroll.warmup.warmup_frames, 5);
        assert_eq!(scroll.window, Some(4));
        assert_eq!(scroll.top, 16);
        assert!(scroll.all);
        assert!(scroll.deep_scan);
        assert!(scroll.timeline);
        assert!(scroll.output.json);
        assert_eq!(
            scroll.output.out,
            Some(PathBuf::from("target/query.scroll.json"))
        );
    }

    #[test]
    fn slice_contract_captures_snapshot_selectors_and_limits() {
        let cli = try_parse_contract([
            "fretboard",
            "slice",
            "target/fret-diag/demo",
            "--test-id",
            "ui-gallery-root",
            "--warmup-frames",
            "6",
            "--window",
            "3",
            "--snapshot-seq",
            "41",
            "--max-matches",
            "5",
            "--max-ancestors",
            "7",
            "--json",
            "--out",
            "target/slice.test-id.json",
        ])
        .expect("slice should parse");

        let DiagCommandContract::Slice(args) = cli.command else {
            panic!("expected slice command");
        };

        assert_eq!(args.source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(args.test_id, "ui-gallery-root");
        assert_eq!(args.warmup.warmup_frames, 6);
        assert_eq!(args.window, Some(3));
        assert_eq!(args.snapshot_seq, Some(41));
        assert_eq!(args.max_matches, 5);
        assert_eq!(args.max_ancestors, 7);
        assert!(args.output.json);
        assert_eq!(
            args.output.out,
            Some(PathBuf::from("target/slice.test-id.json"))
        );
    }

    #[test]
    fn script_direct_contract_accepts_dir_and_path_overrides() {
        let cli = try_parse_contract([
            "fretboard",
            "script",
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json",
            "--dir",
            "target/fret-diag-script",
            "--script-path",
            "target/custom-script.json",
            "--script-trigger-path",
            "target/custom-script.touch",
        ])
        .expect("script direct mode should parse external direct args");

        let DiagCommandContract::Script(args) = cli.command else {
            panic!("expected script command");
        };
        let ScriptSubcommandArgs::Direct(raw) = args.command else {
            panic!("expected direct script command");
        };

        let parsed = super::commands::script::try_parse_direct_script_args(
            std::iter::once("script".to_string()).chain(raw),
        )
        .expect("direct script args should parse");

        assert_eq!(
            parsed.script,
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
        );
        assert_eq!(parsed.dir, Some(PathBuf::from("target/fret-diag-script")));
        assert_eq!(
            parsed.script_path,
            Some(PathBuf::from("target/custom-script.json"))
        );
        assert_eq!(
            parsed.script_trigger_path,
            Some(PathBuf::from("target/custom-script.touch"))
        );
    }

    #[test]
    fn script_validate_contract_accepts_multiple_inputs_and_check_out() {
        let cli = try_parse_contract([
            "fretboard",
            "script",
            "validate",
            "tools/diag-scripts",
            "tools/diag-scripts/ui-gallery-select-*.json",
            "--dir",
            "target/fret-diag-script-validate",
            "--check-out",
            "target/check.script_schema.json",
            "--json",
        ])
        .expect("script validate should parse multiple inputs");

        let DiagCommandContract::Script(args) = cli.command else {
            panic!("expected script command");
        };
        let ScriptSubcommandArgs::Validate(validate) = args.command else {
            panic!("expected script validate command");
        };

        assert_eq!(validate.inputs.len(), 2);
        assert_eq!(
            validate.output.dir,
            Some(PathBuf::from("target/fret-diag-script-validate"))
        );
        assert_eq!(
            validate.check_out,
            Some(PathBuf::from("target/check.script_schema.json"))
        );
        assert!(validate.output.json);
    }

    #[test]
    fn script_shrink_contract_captures_launch_and_session_args() {
        let cli = try_parse_contract([
            "fretboard",
            "script",
            "shrink",
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json",
            "--dir",
            "target/fret-diag-script-shrink",
            "--timeout-ms",
            "9",
            "--poll-ms",
            "3",
            "--session-auto",
            "--shrink-out",
            "target/shrink.min.json",
            "--shrink-any-fail",
            "--shrink-match-reason-code",
            "timeout",
            "--shrink-min-steps",
            "2",
            "--shrink-max-iters",
            "11",
            "--json",
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE=1",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
        ])
        .expect("script shrink should parse the supported subset");

        let DiagCommandContract::Script(args) = cli.command else {
            panic!("expected script command");
        };
        let ScriptSubcommandArgs::Shrink(shrink) = args.command else {
            panic!("expected script shrink command");
        };

        assert_eq!(
            shrink.script,
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
        );
        assert_eq!(
            shrink.output.dir,
            Some(PathBuf::from("target/fret-diag-script-shrink"))
        );
        assert_eq!(shrink.timing.timeout_ms, 9);
        assert_eq!(shrink.timing.poll_ms, 3);
        assert!(shrink.session.session_auto);
        assert_eq!(
            shrink.shrink_out,
            Some(PathBuf::from("target/shrink.min.json"))
        );
        assert!(shrink.shrink_any_fail);
        assert_eq!(shrink.shrink_match_reason_code.as_deref(), Some("timeout"));
        assert_eq!(shrink.shrink_min_steps, 2);
        assert_eq!(shrink.shrink_max_iters, 11);
        assert!(shrink.output.json);
        assert_eq!(
            shrink.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
    }

    #[test]
    fn repeat_contract_captures_compare_and_launch_args() {
        let cli = try_parse_contract([
            "fretboard",
            "repeat",
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json",
            "--repeat",
            "7",
            "--no-compare",
            "--check-memory-p90-max",
            "rss:1024",
            "--compare-ignore-bounds",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
        ])
        .expect("repeat contract should parse repeat-specific args");

        let DiagCommandContract::Repeat(args) = cli.command else {
            panic!("expected repeat command");
        };

        assert_eq!(args.repeat, 7);
        assert!(args.no_compare);
        assert!(args.compare.compare_ignore_bounds);
        assert_eq!(args.check_memory_p90_max, vec!["rss:1024".to_string()]);
        assert_eq!(
            args.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
    }

    #[test]
    fn repro_contract_requires_at_least_one_target() {
        let err =
            try_parse_contract(["fretboard", "repro"]).expect_err("repro should require a target");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn repro_contract_captures_pack_and_launch_args() {
        let cli = try_parse_contract([
            "fretboard",
            "repro",
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json",
            "--ai-only",
            "--include-screenshots",
            "--trace-chrome",
            "--launch",
            "--",
            "cargo",
            "run",
            "-p",
            "fret-ui-gallery",
        ])
        .expect("repro contract should parse the supported subset");

        let DiagCommandContract::Repro(args) = cli.command else {
            panic!("expected repro command");
        };

        assert_eq!(
            args.targets,
            vec!["tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string()]
        );
        assert!(args.pack.ai_only);
        assert!(args.pack.include_screenshots);
        assert!(args.trace_chrome);
        assert_eq!(
            args.launch.normalized_launch_argv(),
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
    }

    #[test]
    fn suite_contract_rejects_conflicting_session_scope_flags() {
        let err = try_parse_contract([
            "fretboard",
            "suite",
            "--session-auto",
            "--session",
            "smoke",
            "--script-dir",
            "tools/diag-scripts/ui-gallery/data_table",
        ])
        .expect_err("session-auto and session must conflict");

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn contract_help_mentions_the_migrated_command_surfaces() {
        let diag_help = render_command_help_path(&[]).expect("diag root help");
        let agent_help = render_command_help("agent").expect("agent help");
        let ai_packet_help = render_command_help("ai-packet").expect("ai-packet help");
        let artifact_help = render_command_help("artifact").expect("artifact help");
        let artifact_lint_help =
            render_command_help_path(&["artifact", "lint"]).expect("artifact lint help");
        let bundle_v2_help = render_command_help("bundle-v2").expect("bundle-v2 help");
        let campaign_help = render_command_help("campaign").expect("campaign help");
        let campaign_run_help =
            render_command_help_path(&["campaign", "run"]).expect("campaign run help");
        let compare_help = render_command_help("compare").expect("compare help");
        let config_help = render_command_help("config").expect("config help");
        let config_doctor_help =
            render_command_help_path(&["config", "doctor"]).expect("config doctor help");
        let dashboard_help = render_command_help("dashboard").expect("dashboard help");
        let dock_graph_help = render_command_help("dock-graph").expect("dock-graph help");
        let dock_routing_help = render_command_help("dock-routing").expect("dock-routing help");
        let doctor_help = render_command_help("doctor").expect("doctor help");
        let doctor_scripts_help =
            render_command_help_path(&["doctor", "scripts"]).expect("doctor scripts help");
        let extensions_help = render_command_help("extensions").expect("extensions help");
        let frames_index_help = render_command_help("frames-index").expect("frames-index help");
        let hotspots_help = render_command_help("hotspots").expect("hotspots help");
        let index_help = render_command_help("index").expect("index help");
        let inspect_help = render_command_help("inspect").expect("inspect help");
        let layout_sidecar_help =
            render_command_help("layout-sidecar").expect("layout-sidecar help");
        let layout_perf_summary_help =
            render_command_help("layout-perf-summary").expect("layout-perf-summary help");
        let lint_help = render_command_help("lint").expect("lint help");
        let list_help = render_command_help("list").expect("list help");
        let list_sessions_help =
            render_command_help_path(&["list", "sessions"]).expect("list sessions help");
        let latest_help = render_command_help("latest").expect("latest help");
        let memory_summary_help =
            render_command_help("memory-summary").expect("memory-summary help");
        let meta_help = render_command_help("meta").expect("meta help");
        let matrix_help = render_command_help("matrix").expect("matrix help");
        let pack_help = render_command_help("pack").expect("pack help");
        let path_help = render_command_help("path").expect("path help");
        let perf_help = render_command_help("perf").expect("perf help");
        let perf_baseline_help = render_command_help("perf-baseline-from-bundles")
            .expect("perf-baseline-from-bundles help");
        let pick_help = render_command_help("pick").expect("pick help");
        let pick_apply_help = render_command_help("pick-apply").expect("pick-apply help");
        let pick_arm_help = render_command_help("pick-arm").expect("pick-arm help");
        let pick_script_help = render_command_help("pick-script").expect("pick-script help");
        let poke_help = render_command_help("poke").expect("poke help");
        let query_help = render_command_help("query").expect("query help");
        let query_snapshots_help =
            render_command_help_path(&["query", "snapshots"]).expect("query snapshots help");
        let resolve_help = render_command_help("resolve").expect("resolve help");
        let resolve_latest_help =
            render_command_help_path(&["resolve", "latest"]).expect("resolve latest help");
        let help = render_command_help("run").expect("run help");
        let repeat_help = render_command_help("repeat").expect("repeat help");
        let repro_help = render_command_help("repro").expect("repro help");
        let registry_help = render_command_help("registry").expect("registry help");
        let screenshots_help = render_command_help("screenshots").expect("screenshots help");
        let script_help = render_command_help("script").expect("script help");
        let script_shrink_help =
            render_command_help_path(&["script", "shrink"]).expect("script shrink help");
        let sessions_help = render_command_help("sessions").expect("sessions help");
        let sessions_clean_help =
            render_command_help_path(&["sessions", "clean"]).expect("sessions clean help");
        let slice_help = render_command_help("slice").expect("slice help");
        let stats_help = render_command_help("stats").expect("stats help");
        let summarize_help = render_command_help("summarize").expect("summarize help");
        let suite_help = render_command_help("suite").expect("suite help");
        let test_ids_help = render_command_help("test-ids").expect("test-ids help");
        let test_ids_index_help =
            render_command_help("test-ids-index").expect("test-ids-index help");
        let trace_help = render_command_help("trace").expect("trace help");
        let triage_help = render_command_help("triage").expect("triage help");
        let windows_help = render_command_help("windows").expect("windows help");
        assert!(agent_help.contains("--warmup-frames"));
        assert!(agent_help.contains("--out"));
        assert!(agent_help.contains("--json"));
        assert!(ai_packet_help.contains("--test-id"));
        assert!(ai_packet_help.contains("--packet-out"));
        assert!(artifact_help.contains("lint"));
        assert!(artifact_lint_help.contains("--warmup-frames"));
        assert!(bundle_v2_help.contains("--mode"));
        assert!(bundle_v2_help.contains("--pretty"));
        assert!(campaign_help.contains("run"));
        assert!(campaign_run_help.contains("--lane"));
        assert!(campaign_run_help.contains("--launch"));
        assert!(compare_help.contains("--footprint"));
        assert!(!compare_help.contains("--compare-footprint"));
        assert!(config_help.contains("doctor"));
        assert!(config_doctor_help.contains("--mode"));
        assert!(config_doctor_help.contains("--print-launch-policy"));
        assert!(config_doctor_help.contains("--env"));
        assert!(dashboard_help.contains("--top"));
        assert!(dock_graph_help.contains("--json"));
        assert!(dock_routing_help.contains("--warmup-frames"));
        assert!(doctor_help.contains("--fix-schema2"));
        assert!(doctor_scripts_help.contains("--max-examples"));
        assert!(extensions_help.contains("--key"));
        assert!(frames_index_help.contains("--warmup-frames"));
        assert!(hotspots_help.contains("--lite"));
        assert!(hotspots_help.contains("--metric"));
        assert!(index_help.contains("--warmup-frames"));
        assert!(inspect_help.contains("toggle"));
        assert!(inspect_help.contains("--consume-clicks"));
        assert!(diag_help.contains("Usage: fretboard diag"));
        assert!(diag_help.contains("agent"));
        assert!(diag_help.contains("path"));
        assert!(diag_help.contains("poke"));
        assert!(diag_help.contains("latest"));
        assert!(latest_help.contains("--dir"));
        assert!(layout_sidecar_help.contains("--print"));
        assert!(layout_perf_summary_help.contains("--warmup-frames"));
        assert!(layout_perf_summary_help.contains("--top"));
        assert!(lint_help.contains("--all-test-ids"));
        assert!(lint_help.contains("--lint-eps-px"));
        assert!(list_help.contains("scripts"));
        assert!(list_sessions_help.contains("--dir"));
        assert!(memory_summary_help.contains("--sort-key"));
        assert!(memory_summary_help.contains("--fit-linear"));
        assert!(memory_summary_help.contains("--top-sessions"));
        assert!(meta_help.contains("--meta-report"));
        assert!(matrix_help.contains("ui-gallery"));
        assert!(matrix_help.contains("--launch"));
        assert!(matrix_help.contains("--check-view-cache-reuse-min"));
        assert!(!matrix_help.contains("--launch-write-bundle-json"));
        assert!(!matrix_help.contains("--check-dock-drag-min"));
        assert!(pack_help.contains("--pack-schema2-only"));
        assert!(!pack_help.contains("--schema2-only"));
        assert!(path_help.contains("--trigger-path"));
        assert!(perf_help.contains("--prewarm-script"));
        assert!(perf_help.contains("--perf-threshold-agg"));
        assert!(perf_baseline_help.contains("--perf-baseline-out"));
        assert!(perf_baseline_help.contains("--perf-baseline-headroom-pct"));
        assert!(!perf_baseline_help.contains("--pack"));
        assert!(pick_help.contains("--timeout-ms"));
        assert!(pick_apply_help.contains("--ptr"));
        assert!(pick_apply_help.contains("--timeout-ms"));
        assert!(pick_arm_help.contains("Usage: fretboard diag pick-arm"));
        assert!(pick_script_help.contains("--pick-script-out"));
        assert!(pick_script_help.contains("--poll-ms"));
        assert!(poke_help.contains("--request-id"));
        assert!(poke_help.contains("--record-run"));
        assert!(query_help.contains("test-id"));
        assert!(query_help.contains("snapshots"));
        assert!(query_snapshots_help.contains("--step-index"));
        assert!(query_snapshots_help.contains("--semantics-source"));
        assert!(resolve_help.contains("latest"));
        assert!(resolve_latest_help.contains("--within-session"));
        assert!(resolve_latest_help.contains("--dir"));
        assert!(resolve_latest_help.contains("Usage: fretboard diag resolve latest"));
        assert!(help.contains("run"));
        assert!(repeat_help.contains("--no-compare"));
        assert!(repro_help.contains("--ai-only"));
        assert!(registry_help.contains("check"));
        assert!(registry_help.contains("write"));
        assert!(registry_help.contains("print"));
        assert!(screenshots_help.contains("--json"));
        assert!(script_help.contains("normalize"));
        assert!(script_help.contains("upgrade"));
        assert!(script_help.contains("Direct execution"));
        assert!(script_shrink_help.contains("--shrink-out"));
        assert!(script_shrink_help.contains("--session-auto"));
        assert!(!script_shrink_help.contains("--reuse-launch"));
        assert!(sessions_help.contains("clean"));
        assert!(sessions_clean_help.contains("--older-than-days"));
        assert!(sessions_clean_help.contains("--apply"));
        assert!(slice_help.contains("--step-index"));
        assert!(slice_help.contains("--warmup-frames"));
        assert!(stats_help.contains("--stats-lite-checks-json"));
        assert!(stats_help.contains("--check-hover-layout"));
        assert!(stats_help.contains("--check-retained-vlist-keep-alive-reuse-min"));
        assert!(!stats_help.contains("--stats-lite-matrix-json"));
        assert!(!stats_help.contains("--check-prepaint-actions-min"));
        assert!(!stats_help.contains("--check-chart-sampling-window-shifts-min"));
        assert!(summarize_help.contains("[INPUT]..."));
        assert!(suite_help.contains("--script-dir"));
        assert!(suite_help.contains("--launch"));
        assert!(test_ids_help.contains("--max-test-ids"));
        assert!(test_ids_index_help.contains("--warmup-frames"));
        assert!(!test_ids_index_help.contains("--out"));
        assert!(trace_help.contains("--trace-out"));
        assert!(triage_help.contains("--lite"));
        assert!(triage_help.contains("--metric"));
        assert!(windows_help.contains("--warmup-frames"));
        assert!(windows_help.contains("Usage: fretboard diag windows"));
    }
}
