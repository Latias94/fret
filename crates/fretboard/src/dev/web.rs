use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use super::contracts::DevWebCommandArgs;
use super::project::{SelectedWebTarget, WebTargetKind, resolve_web_target};
use super::resolve_bool_override;

fn configure_trunk_web_command(
    cmd: &mut Command,
    package_root: &Path,
    port: Option<u16>,
    target_html: &Path,
) {
    cmd.current_dir(package_root)
        .args(["serve", "--no-color", "--open", "false"]);
    cmd.env_remove("NO_COLOR");

    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }

    cmd.arg(target_html);
}

pub(crate) fn run_web_contract(args: DevWebCommandArgs) -> Result<(), String> {
    let selected = resolve_web_target(
        args.manifest_path.as_deref(),
        args.package.as_deref(),
        args.bin.as_deref(),
    )?;
    let port = args.port;
    let open = resolve_bool_override(args.open, args.no_open).unwrap_or(true);
    let devtools_ws_url = args.devtools_ws_url;
    let devtools_token = args.devtools_token;

    if !selected.index_html_path.is_file() {
        return Err(format!(
            "selected package `{}` does not contain `index.html` at `{}`",
            selected.package_name,
            selected.index_html_path.display()
        ));
    }

    let effective_port = port.unwrap_or(8080);
    let mut url = format!("http://127.0.0.1:{effective_port}");

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

    let temp_target = match selected.kind {
        WebTargetKind::Lib => None,
        WebTargetKind::Bin => Some(write_temp_trunk_target(&selected)?),
    };
    let target_html = temp_target
        .as_deref()
        .unwrap_or(selected.index_html_path.as_path());

    eprintln!(
        "Starting Trunk dev server for package `{}` in `{}`",
        selected.package_name,
        selected.package_root.display()
    );

    let mut cmd = Command::new("trunk");
    configure_trunk_web_command(&mut cmd, &selected.package_root, port, target_html);

    let mut child = cmd.spawn().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            err.to_string()
        }
    })?;

    std::thread::spawn({
        let url = url.clone();
        let package_root = selected.package_root.clone();
        move || {
            use std::net::{SocketAddr, TcpStream, ToSocketAddrs as _};

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
                    let assets_ready = wait_for_trunk_web_assets_ready(&package_root, remaining);
                    if assets_ready {
                        eprintln!("\nFret web target ready: {url}\n");
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

            eprintln!("\nFret web target (may still be building): {url}\n");
            if open && let Err(err) = open_url(&url) {
                eprintln!("warning: failed to open browser: {err}");
            }
        }
    });

    let status = child.wait().map_err(|err| err.to_string())?;
    if let Some(path) = temp_target.as_deref() {
        let _ = std::fs::remove_file(path);
    }
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
}

fn write_temp_trunk_target(selected: &SelectedWebTarget) -> Result<PathBuf, String> {
    let bin_name = selected
        .target_name
        .as_deref()
        .ok_or_else(|| "internal error: web binary target missing".to_string())?;
    let original = std::fs::read_to_string(&selected.index_html_path).map_err(|err| {
        format!(
            "failed to read `{}`: {err}",
            selected.index_html_path.display()
        )
    })?;
    let patched = rewrite_trunk_rust_link(&original, "Cargo.toml", bin_name)?;
    let temp_name = format!(".fretboard-dev-{}.html", sanitize_temp_component(bin_name));
    let temp_path = selected.package_root.join(temp_name);
    std::fs::write(&temp_path, patched)
        .map_err(|err| format!("failed to write `{}`: {err}", temp_path.display()))?;
    Ok(temp_path)
}

fn sanitize_temp_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    out
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

fn wait_for_trunk_web_assets_ready(package_root: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    let mut last_sig: Option<TrunkWebAssetSignature> = None;
    let mut stable_since: Option<Instant> = None;

    while start.elapsed() < timeout {
        if let Some(sig) = trunk_web_asset_signature(package_root) {
            match last_sig {
                Some(previous) if previous == sig => {
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

fn trunk_web_asset_signature(package_root: &Path) -> Option<TrunkWebAssetSignature> {
    let dist_dir = package_root.join("dist");
    let index = dist_dir.join("index.html");
    let js = latest_dist_asset(&dist_dir, |name| name.ends_with(".js"))?;
    let wasm = latest_dist_asset(&dist_dir, |name| name.ends_with(".wasm"))?;

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

fn latest_dist_asset(dist_dir: &Path, mut predicate: impl FnMut(&str) -> bool) -> Option<PathBuf> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
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
            .map_err(|err| err.to_string())?;
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
            .map_err(|err| err.to_string())?;
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
            .map_err(|err| err.to_string())?;
        if !status.success() {
            return Err(format!("xdg-open exited with status: {status}"));
        }
        Ok(())
    }
}

fn rewrite_trunk_rust_link(
    html: &str,
    manifest_href: &str,
    bin_name: &str,
) -> Result<String, String> {
    let mut search_from = 0usize;
    while let Some(offset) = html[search_from..].find("<link") {
        let start = search_from + offset;
        let end = html[start..]
            .find('>')
            .map(|relative| start + relative)
            .ok_or_else(|| "malformed `<link>` tag in index.html".to_string())?;
        let tag = &html[start..=end];
        if let Some(rewritten) = rewrite_matching_link_tag(tag, manifest_href, bin_name)? {
            let mut out = String::with_capacity(html.len() + 64);
            out.push_str(&html[..start]);
            out.push_str(&rewritten);
            out.push_str(&html[end + 1..]);
            return Ok(out);
        }
        search_from = end + 1;
    }

    Err("failed to find `<link data-trunk rel=\"rust\">` in index.html".to_string())
}

fn rewrite_matching_link_tag(
    tag: &str,
    manifest_href: &str,
    bin_name: &str,
) -> Result<Option<String>, String> {
    let parsed = parse_link_tag(tag)?;
    if !parsed.is_trunk_rust_link() {
        return Ok(None);
    }

    let mut attrs = Vec::new();
    for attr in parsed.attrs {
        if attr.name.eq_ignore_ascii_case("href") || attr.name.eq_ignore_ascii_case("data-bin") {
            continue;
        }
        attrs.push(attr);
    }
    attrs.push(HtmlAttr {
        name: "href".to_string(),
        value: Some(manifest_href.to_string()),
    });
    attrs.push(HtmlAttr {
        name: "data-bin".to_string(),
        value: Some(bin_name.to_string()),
    });

    let mut rendered = String::from("<link");
    for attr in attrs {
        rendered.push(' ');
        rendered.push_str(&attr.name);
        if let Some(value) = attr.value {
            rendered.push('=');
            rendered.push('"');
            rendered.push_str(&value);
            rendered.push('"');
        }
    }
    if parsed.self_closing {
        rendered.push_str(" />");
    } else {
        rendered.push('>');
    }
    Ok(Some(rendered))
}

#[derive(Debug, Clone)]
struct ParsedLinkTag {
    attrs: Vec<HtmlAttr>,
    self_closing: bool,
}

impl ParsedLinkTag {
    fn is_trunk_rust_link(&self) -> bool {
        let has_data_trunk = self
            .attrs
            .iter()
            .any(|attr| attr.name.eq_ignore_ascii_case("data-trunk"));
        let is_rust = self.attrs.iter().any(|attr| {
            attr.name.eq_ignore_ascii_case("rel")
                && attr
                    .value
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case("rust"))
        });
        has_data_trunk && is_rust
    }
}

#[derive(Debug, Clone)]
struct HtmlAttr {
    name: String,
    value: Option<String>,
}

fn parse_link_tag(tag: &str) -> Result<ParsedLinkTag, String> {
    if !tag.starts_with("<link") || !tag.ends_with('>') {
        return Err("expected a `<link ...>` tag".to_string());
    }

    let bytes = tag.as_bytes();
    let mut idx = "<link".len();
    let mut attrs = Vec::new();
    let mut self_closing = false;

    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }

        if idx >= bytes.len() - 1 {
            break;
        }

        match bytes[idx] {
            b'/' => {
                self_closing = true;
                idx += 1;
            }
            b'>' => break,
            _ => {
                let name_start = idx;
                while idx < bytes.len()
                    && !bytes[idx].is_ascii_whitespace()
                    && !matches!(bytes[idx], b'=' | b'/' | b'>')
                {
                    idx += 1;
                }
                let name = tag[name_start..idx].to_string();

                while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                    idx += 1;
                }

                let value = if idx < bytes.len() && bytes[idx] == b'=' {
                    idx += 1;
                    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                        idx += 1;
                    }

                    if idx >= bytes.len() {
                        return Err("malformed HTML attribute".to_string());
                    }

                    let quote = bytes[idx];
                    if matches!(quote, b'"' | b'\'') {
                        idx += 1;
                        let value_start = idx;
                        while idx < bytes.len() && bytes[idx] != quote {
                            idx += 1;
                        }
                        if idx >= bytes.len() {
                            return Err("unterminated HTML attribute value".to_string());
                        }
                        let value = tag[value_start..idx].to_string();
                        idx += 1;
                        Some(value)
                    } else {
                        let value_start = idx;
                        while idx < bytes.len()
                            && !bytes[idx].is_ascii_whitespace()
                            && !matches!(bytes[idx], b'/' | b'>')
                        {
                            idx += 1;
                        }
                        Some(tag[value_start..idx].to_string())
                    }
                } else {
                    None
                };

                attrs.push(HtmlAttr { name, value });
            }
        }
    }

    Ok(ParsedLinkTag {
        attrs,
        self_closing,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_link_tag, rewrite_trunk_rust_link};

    #[test]
    fn rewrite_trunk_rust_link_injects_href_and_data_bin() {
        let html = r#"
<!doctype html>
<html>
  <head>
    <link data-trunk rel="rust" data-wasm-opt="2" />
  </head>
</html>
"#;
        let rewritten = rewrite_trunk_rust_link(html, "Cargo.toml", "preview")
            .expect("rust link should rewrite");
        assert!(rewritten.contains(r#"href="Cargo.toml""#));
        assert!(rewritten.contains(r#"data-bin="preview""#));
        assert!(rewritten.contains(r#"data-wasm-opt="2""#));
    }

    #[test]
    fn parse_link_tag_understands_single_quotes() {
        let parsed = parse_link_tag("<link data-trunk rel='rust' data-wasm-opt='2'>")
            .expect("tag should parse");
        assert!(parsed.is_trunk_rust_link());
    }
}
