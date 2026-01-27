use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "init" => crate::scaffold::init_cmd(args.collect()),
        "new" => crate::scaffold::new_cmd(args.collect()),
        "config" => crate::config::config_cmd(args.collect()),
        "hotpatch" => crate::hotpatch::hotpatch_cmd(args.collect()),
        "diag" => crate::diag::diag_cmd(args.collect()),
        "list" => match args.next().as_deref() {
            Some("native-demos") => crate::demos::list_native_demos(),
            Some("web-demos") => crate::demos::list_web_demos(),
            Some(other) => Err(format!("unknown list target: {other}")),
            None => Err("missing list target (try: list native-demos)".to_string()),
        },
        "dev" => match args.next().as_deref() {
            Some("native") => crate::dev::dev_native(args.collect()),
            Some("web") => crate::dev::dev_web(args.collect()),
            Some(other) => Err(format!("unknown dev target: {other}")),
            None => Err("missing dev target (try: dev native)".to_string()),
        },
        other => Err(format!("unknown command: {other}")),
    }
}

pub(crate) fn help() -> Result<(), String> {
    println!(
        r#"fretboard dev tooling for the Fret workspace

Usage:
  fretboard help
  fretboard new [template] [--path <path>] [--name <name>] [--ui-assets] [--icons <lucide|radix|none>] [--command-palette]
  fretboard new             # interactive wizard
  fretboard new todo        # non-interactive (template shortcut)
  fretboard new todo-mvu    # non-interactive (MVU template shortcut)
  fretboard new hello       # non-interactive (template shortcut)
  fretboard new empty       # minimal Cargo-like project
  fretboard init <template> [...]    # alias for `new` (compat)
  fretboard config menubar [--path <path>] [--force]
  fretboard hotpatch poke [--path <path>]
  fretboard hotpatch path [--path <path>]
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]
  fretboard diag path [--trigger-path <path>] [--dir <dir>]
  fretboard diag poke [--trigger-path <path>] [--dir <dir>]
  fretboard diag latest [--dir <dir>]
  fretboard diag pack [<bundle_dir|bundle.json>] [--dir <dir>] [--pack-out <path>] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots]
  fretboard diag triage <bundle_dir|bundle.json> [--top <n>] [--sort <invalidation|time>] [--warmup-frames <n>] [--json] [--out <path>]
  fretboard diag script <script.json> [--dir <dir>] [--script-path <path>] [--script-trigger-path <path>]
  fretboard diag run <script.json> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--pack] [--pack-out <path>] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-wheel-scroll <test_id>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag suite <ui-gallery|ui-gallery-virt-retained|docking-arbitration|script.json...> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-wheel-scroll <test_id>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag stats <bundle_dir|bundle.json> [--top <n>] [--sort <invalidation|time>] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-wheel-scroll <test_id>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>]
  fretboard diag matrix ui-gallery [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--warmup-frames <n>] [--compare-eps-px <px>] [--compare-ignore-bounds] [--compare-ignore-scene-fingerprint] [--check-view-cache-reuse-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>] [--json]
  fretboard diag compare <bundle_a|dir> <bundle_b|dir> [--warmup-frames <n>] [--compare-eps-px <px>] [--compare-ignore-bounds] [--compare-ignore-scene-fingerprint] [--json]
  fretboard diag perf <ui-gallery|script.json...> [--top <n>] [--sort <invalidation|time>] [--repeat <n>] [--timeout-ms <ms>] [--poll-ms <ms>] [--dir <dir>] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-wheel-scroll <test_id>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard list native-demos
  fretboard list web-demos
  fretboard dev native [--bin <name> | --choose] [--hotpatch] [--hotpatch-trigger-path <path>] [--hotpatch-poll-ms <ms>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose] --hotpatch-devserver <ws_endpoint> [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose] --hotpatch-dx [--hotpatch-dx-ws <ws_endpoint>] [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev web [--port <port>] [--demo <demo> | --choose]

Examples:
  fretboard new todo --name my-todo
  fretboard new todo-mvu --name my-todo-mvu
  fretboard new hello --name hello-world
  fretboard new hello --name hello-world --command-palette
  fretboard new todo --name my-todo --icons none
  fretboard new empty --name my-app
  fretboard config menubar --path .
  fretboard dev native --bin components_gallery
  fretboard dev native --bin todo_demo
  fretboard dev native --bin assets_demo
  fretboard dev native --bin hotpatch_smoke_demo
  fretboard dev native --choose
  fretboard dev native --bin image_upload_demo -- --help
  fretboard dev native --hotpatch --choose   # file-triggered runner reload (default: `.fret/hotpatch.touch`)
  fretboard hotpatch poke                   # updates `.fret/hotpatch.touch` (triggers a reload)
  fretboard hotpatch watch                  # polls workspace sources and auto-pokes on change
  fretboard diag poke                      # touches `target/fret-diag/trigger.touch` (dumps diagnostics when enabled)
  fretboard diag latest                    # prints the most recent diagnostics bundle path
  fretboard diag pack                      # zips a bundle directory for sharing (default: latest)
  fretboard diag triage                    # prints a machine-readable triage JSON (built from bundle stats)
  fretboard diag script ./script.json      # writes `target/fret-diag/script.json` and touches `target/fret-diag/script.touch`
  fretboard diag run ./script.json         # pushes script and waits for `script.result.json` (exit 0 on pass, 1 on fail/timeout)
  fretboard diag run tools/diag-scripts/todo-baseline.json --dir target/fret-diag-todo-auto --launch -- cargo run -p fret-demo --bin todo_demo
  fretboard diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --pack --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag suite ui-gallery          # runs `tools/diag-scripts/ui-gallery-*.json` sequentially (app must be running)
  fretboard diag stats ./target/fret-diag/1234  # summarizes invalidation + other frame stats from a `bundle.json`
  fretboard diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --repeat 7 --warmup-frames 5 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --check-overlay-synthesis-min 1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag compare ./target/fret-diag/uncached ./target/fret-diag/cached --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --json
  fretboard dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard dev web --demo plot_demo
"#
    );
    Ok(())
}

pub(crate) fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
