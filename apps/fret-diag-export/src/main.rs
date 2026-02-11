use anyhow::{Context as _, anyhow};
use fret_diag::transport::{
    ClientKindV1, DevtoolsWsClientConfig, ToolingDiagClient, WsDiagTransportConfig,
};
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionDescriptorV1, DevtoolsSessionListV1,
    DiagTransportMessageV1,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        return Err(anyhow!(
            "fret-diag-export is only available on native targets"
        ));
    }

    let args = Args::parse(std::env::args().skip(1))?;

    let ws_url = args
        .ws_url
        .or_else(|| std::env::var("FRET_DEVTOOLS_WS").ok())
        .unwrap_or_else(|| "ws://127.0.0.1:7331/".to_string());

    let token = args
        .token
        .or_else(|| std::env::var("FRET_DEVTOOLS_TOKEN").ok())
        .context("missing --token (or env FRET_DEVTOOLS_TOKEN)")?;

    let out_dir = args
        .out_dir
        .unwrap_or_else(|| PathBuf::from(".fret/diag/exports"));
    let timeout = Duration::from_millis(args.timeout_ms.unwrap_or(180_000));

    let client = connect_tooling_client(&ws_url, &token)?;

    let session_id = match args.session_id {
        Some(s) => s,
        None => wait_for_web_app_session(&client, Duration::from_secs(60))
            .context("timeout waiting for a web_app session (is the app connected?)")?,
    };

    let script_json = read_json(&args.script_path)
        .with_context(|| format!("failed reading script: {}", args.script_path.display()))?;

    client.send(DiagTransportMessageV1 {
        schema_version: 1,
        r#type: "script.run".to_string(),
        session_id: Some(session_id.clone()),
        request_id: None,
        payload: serde_json::json!({ "script": script_json }),
    });

    let (dir, bundle) = wait_for_bundle_dumped(&client, &session_id, timeout)?;

    let export_dir = out_dir.join(&dir);
    std::fs::create_dir_all(&export_dir)
        .with_context(|| format!("create_dir_all failed: {}", export_dir.display()))?;

    let bundle_path = export_dir.join("bundle.json");
    std::fs::write(&bundle_path, serde_json::to_string_pretty(&bundle)?)
        .with_context(|| format!("write failed: {}", bundle_path.display()))?;

    let latest_path = out_dir.join("latest.txt");
    let _ = std::fs::write(&latest_path, format!("{dir}\n"));

    println!("{}", export_dir.display());
    Ok(())
}

#[derive(Debug)]
struct Args {
    ws_url: Option<String>,
    token: Option<String>,
    session_id: Option<String>,
    script_path: PathBuf,
    out_dir: Option<PathBuf>,
    timeout_ms: Option<u64>,
}

impl Args {
    fn parse<I>(mut it: I) -> anyhow::Result<Self>
    where
        I: Iterator<Item = String>,
    {
        let mut ws_url: Option<String> = None;
        let mut token: Option<String> = None;
        let mut session_id: Option<String> = None;
        let mut script_path: Option<PathBuf> = None;
        let mut out_dir: Option<PathBuf> = None;
        let mut timeout_ms: Option<u64> = None;

        while let Some(arg) = it.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--ws-url" => ws_url = Some(next_string(&mut it, "--ws-url")?),
                "--token" => token = Some(next_string(&mut it, "--token")?),
                "--session-id" => session_id = Some(next_string(&mut it, "--session-id")?),
                "--script" => script_path = Some(PathBuf::from(next_string(&mut it, "--script")?)),
                "--out-dir" => out_dir = Some(PathBuf::from(next_string(&mut it, "--out-dir")?)),
                "--timeout-ms" => {
                    timeout_ms = Some(
                        next_string(&mut it, "--timeout-ms")?
                            .parse::<u64>()
                            .context("--timeout-ms must be an integer")?,
                    );
                }
                other if other.starts_with("--") => {
                    return Err(anyhow!("unknown flag: {other} (try --help)"));
                }
                other => {
                    if script_path.is_none() {
                        script_path = Some(PathBuf::from(other));
                    } else {
                        return Err(anyhow!("unexpected arg: {other} (try --help)"));
                    }
                }
            }
        }

        let script_path = script_path.context("missing --script <path> (try --help)")?;

        Ok(Self {
            ws_url,
            token,
            session_id,
            script_path,
            out_dir,
            timeout_ms,
        })
    }
}

fn print_help() {
    eprintln!(
        r#"fret-diag-export

Export a diagnostics bundle from a devtools-ws-connected app into a local directory, by running a
script that includes a `capture_bundle` step and waiting for `bundle.dumped`.

Usage:
  fret-diag-export --script <script.json> [--ws-url <ws://.../>] [--token <token>] [--session-id <id>] [--out-dir <path>] [--timeout-ms <ms>]

Defaults:
  --ws-url     env FRET_DEVTOOLS_WS or ws://127.0.0.1:7331/
  --token      env FRET_DEVTOOLS_TOKEN
  --out-dir    .fret/diag/exports
  --timeout-ms 180000

Examples:
  cargo run -p fret-devtools-ws
  cd apps/fret-ui-gallery-web; trunk serve --port 8080
  # open: http://127.0.0.1:8080/?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=...
  cargo run -p fret-diag-export -- --script tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json --token <token>
"#
    );
}

fn next_string<I>(it: &mut I, flag: &str) -> anyhow::Result<String>
where
    I: Iterator<Item = String>,
{
    it.next()
        .ok_or_else(|| anyhow!("missing value for {flag}"))
        .map(|s| s.to_string())
}

fn connect_tooling_client(ws_url: &str, token: &str) -> anyhow::Result<ToolingDiagClient> {
    let mut cfg = DevtoolsWsClientConfig::with_defaults(ws_url.to_string(), token.to_string());
    cfg.client_kind = ClientKindV1::Tooling;
    cfg.client_version = format!("fret-diag-export/{}", env!("CARGO_PKG_VERSION"));
    cfg.capabilities = vec![
        "sessions".to_string(),
        "inspect".to_string(),
        "pick".to_string(),
        "scripts".to_string(),
        "bundles".to_string(),
    ];

    ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(cfg)).map_err(|e| anyhow!("{e}"))
}

fn wait_for_web_app_session(client: &ToolingDiagClient, timeout: Duration) -> Option<String> {
    let deadline = Instant::now() + timeout;
    let mut known: Vec<DevtoolsSessionDescriptorV1> = Vec::new();

    while Instant::now() < deadline {
        while let Some(msg) = client.try_recv() {
            match msg.r#type.as_str() {
                "session.list" => {
                    if let Ok(list) = serde_json::from_value::<DevtoolsSessionListV1>(msg.payload) {
                        known = list.sessions;
                    }
                }
                "session.added" => {
                    if let Ok(added) = serde_json::from_value::<DevtoolsSessionAddedV1>(msg.payload)
                    {
                        known.retain(|s| s.session_id != added.session.session_id);
                        known.push(added.session);
                    }
                }
                _ => {}
            }
        }

        if let Some(s) = known.iter().find(|s| s.client_kind == "web_app") {
            return Some(s.session_id.clone());
        }

        std::thread::sleep(Duration::from_millis(20));
    }

    None
}

fn wait_for_bundle_dumped(
    client: &ToolingDiagClient,
    session_id: &str,
    timeout: Duration,
) -> anyhow::Result<(String, serde_json::Value)> {
    let deadline = Instant::now() + timeout;
    let mut last_script_stage: Option<String> = None;

    while Instant::now() < deadline {
        while let Some(msg) = client.try_recv() {
            if msg.session_id.as_deref() != Some(session_id) {
                continue;
            }

            match msg.r#type.as_str() {
                "script.result" => {
                    if let Some(stage) = msg.payload.get("stage").and_then(|v| v.as_str()) {
                        if last_script_stage.as_deref() != Some(stage) {
                            eprintln!("script.result stage={stage}");
                            last_script_stage = Some(stage.to_string());
                        }
                    }
                }
                "bundle.dumped" => {
                    let dir = msg
                        .payload
                        .get("dir")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s.to_string())
                        .context("bundle.dumped missing dir")?;

                    let bundle = msg
                        .payload
                        .get("bundle")
                        .cloned()
                        .context("bundle.dumped missing bundle payload")?;

                    return Ok((dir, bundle));
                }
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_millis(20));
    }

    Err(anyhow!("timeout waiting for bundle.dumped"))
}

fn read_json(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let text = std::fs::read_to_string(path)?;
    let json = serde_json::from_str::<serde_json::Value>(&text)?;
    Ok(json)
}
