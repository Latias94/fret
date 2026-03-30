use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use crate::cli::workspace_root;
use crate::demos::{display_path, prompt_choose_demo, validate_web_demo, web_demos_as_vec};

use super::{contracts::DevWebCommandArgs, resolve_bool_override};

fn configure_trunk_web_command(cmd: &mut Command, web_dir: &Path, port: Option<u16>) {
    cmd.current_dir(web_dir).args(["serve", "--no-color"]);
    cmd.env_remove("NO_COLOR");

    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }
}

pub(crate) fn run_web_contract(args: DevWebCommandArgs) -> Result<(), String> {
    let port = args.port;
    let demo = args.demo;
    let choose = args.choose;
    // Dev web is primarily an interactive workflow; default to opening the browser
    // once the server is reachable. Use `--no-open` for CI or when you explicitly
    // do not want the auto-open behavior.
    let open = resolve_bool_override(args.open, args.no_open).unwrap_or(true);
    let devtools_ws_url = args.devtools_ws_url;
    let devtools_token = args.devtools_token;

    let root = workspace_root()?;
    let web_dir = root.join("apps").join("fret-demo-web");

    let effective_port = port.unwrap_or(8080);
    let mut url = format!("http://127.0.0.1:{effective_port}");
    let demo = match (demo.as_deref(), choose) {
        (Some(name), _) => {
            validate_web_demo(name)?;
            Some(name.to_string())
        }
        (None, true) => {
            let demos = web_demos_as_vec();
            let default = demos.first().map(|d| d.as_str());
            Some(prompt_choose_demo(
                "Select a web demo",
                &demos,
                default,
                validate_web_demo,
            )?)
        }
        (None, false) => None,
    };
    if let Some(demo) = demo.as_deref() {
        url.push_str(&format!("/?demo={demo}"));
    }

    if let Some(ws_url) = devtools_ws_url.as_deref() {
        if ws_url.trim().is_empty() {
            return Err("--devtools-ws-url must not be empty".to_string());
        }
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("fret_devtools_ws=");
        url.push_str(ws_url.trim());
    }

    if let Some(token) = devtools_token.as_deref() {
        if token.trim().is_empty() {
            return Err("--devtools-token must not be empty".to_string());
        }
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("fret_devtools_token=");
        url.push_str(token.trim());
    }

    eprintln!("Starting Trunk dev server in `{}`", display_path(&web_dir));

    let mut cmd = Command::new("trunk");
    configure_trunk_web_command(&mut cmd, &web_dir, port);

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            e.to_string()
        }
    })?;

    std::thread::spawn({
        let url = url.clone();
        let web_dir = web_dir.clone();
        move || {
            use std::net::{SocketAddr, TcpStream, ToSocketAddrs as _};
            use std::time::{Duration, Instant};

            let start = Instant::now();
            let deadline = Duration::from_secs(90);

            let Ok(mut addrs) = format!("127.0.0.1:{effective_port}").to_socket_addrs() else {
                return;
            };
            let Some(addr) = addrs.find(SocketAddr::is_ipv4) else {
                return;
            };

            while start.elapsed() < deadline {
                if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
                    let remaining = deadline.saturating_sub(start.elapsed());
                    let assets_ready = wait_for_trunk_web_assets_ready(&web_dir, remaining);
                    if assets_ready {
                        eprintln!("\nFret web demo ready: {url}\n");
                    } else {
                        eprintln!(
                            "\nFret web dev server is reachable but assets may still be building: {url}\n"
                        );
                    }
                    if open && let Err(err) = open_url(&url) {
                        eprintln!("warning: failed to open browser: {err}");
                    }
                    return;
                }
                std::thread::sleep(Duration::from_millis(200));
            }

            eprintln!("\nFret web demo (may still be building): {url}\n");
            if open && let Err(err) = open_url(&url) {
                eprintln!("warning: failed to open browser: {err}");
            }
        }
    });

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrunkWebAssetSignature {
    index_len: u64,
    index_mtime_ms: u128,
    js_len: u64,
    js_mtime_ms: u128,
    wasm_len: u64,
    wasm_mtime_ms: u128,
}

fn wait_for_trunk_web_assets_ready(web_dir: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    let mut last_sig: Option<TrunkWebAssetSignature> = None;
    let mut stable_since: Option<Instant> = None;

    while start.elapsed() < timeout {
        if let Some(sig) = trunk_web_asset_signature(web_dir) {
            match last_sig {
                Some(prev) if prev == sig => {
                    let stable = stable_since.get_or_insert_with(Instant::now);
                    if stable.elapsed() >= Duration::from_millis(1200) {
                        return true;
                    }
                }
                _ => {
                    last_sig = Some(sig);
                    stable_since = Some(Instant::now());
                }
            }
        } else {
            last_sig = None;
            stable_since = None;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    false
}

fn trunk_web_asset_signature(web_dir: &Path) -> Option<TrunkWebAssetSignature> {
    let dist_dir = web_dir.join("dist");
    let index = dist_dir.join("index.html");
    let js = latest_dist_asset(&dist_dir, |name| {
        name.starts_with("fret-demo-web-") && name.ends_with(".js")
    })?;
    let wasm = latest_dist_asset(&dist_dir, |name| {
        name.starts_with("fret-demo-web-") && name.ends_with("_bg.wasm")
    })?;

    let (index_len, index_mtime_ms) = file_len_and_mtime_ms(&index)?;
    let (js_len, js_mtime_ms) = file_len_and_mtime_ms(&js)?;
    let (wasm_len, wasm_mtime_ms) = file_len_and_mtime_ms(&wasm)?;

    if index_len == 0 || js_len == 0 || wasm_len == 0 {
        return None;
    }

    Some(TrunkWebAssetSignature {
        index_len,
        index_mtime_ms,
        js_len,
        js_mtime_ms,
        wasm_len,
        wasm_mtime_ms,
    })
}

fn latest_dist_asset(
    dist_dir: &Path,
    mut predicate: impl FnMut(&str) -> bool,
) -> Option<std::path::PathBuf> {
    let mut newest: Option<(SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(dist_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !entry.file_type().ok()?.is_file() {
            continue;
        }
        let name = path.file_name()?.to_str()?;
        if !predicate(name) {
            continue;
        }
        let modified = entry
            .metadata()
            .ok()?
            .modified()
            .ok()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        match &newest {
            Some((best_modified, _)) if modified <= *best_modified => {}
            _ => newest = Some((modified, path)),
        }
    }
    newest.map(|(_, path)| path)
}

fn file_len_and_mtime_ms(path: &Path) -> Option<(u64, u128)> {
    let meta = std::fs::metadata(path).ok()?;
    let modified_ms = meta
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some((meta.len(), modified_ms))
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "rundll32 FileProtocolHandler exited with status: {status}"
            ));
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("open exited with status: {status}"));
        }
        Ok(())
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("xdg-open exited with status: {status}"));
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::configure_trunk_web_command;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn configure_trunk_web_command_removes_no_color_and_sets_flag() {
        let web_dir = Path::new("/tmp/fret-demo-web");
        let mut cmd = Command::new("trunk");
        cmd.env("NO_COLOR", "1");

        configure_trunk_web_command(&mut cmd, web_dir, Some(9001));

        let args: Vec<String> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(cmd.get_current_dir(), Some(web_dir));
        assert_eq!(args, vec!["serve", "--no-color", "--port", "9001"]);

        let no_color = cmd
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("NO_COLOR"))
            .expect("NO_COLOR should be explicitly removed for trunk");
        assert!(no_color.1.is_none());
    }
}
