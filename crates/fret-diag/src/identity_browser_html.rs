use std::fmt::Write;
use std::path::Path;

use crate::identity_browser::{IdentityWarningBrowserFilters, IdentityWarningBrowserReport};

pub(crate) fn write_identity_browser_html(
    path: &Path,
    bundle: &str,
    report: &IdentityWarningBrowserReport,
    filters: &IdentityWarningBrowserFilters,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let html = render_identity_browser_html(bundle, report, filters);
    std::fs::write(path, html.as_bytes()).map_err(|e| e.to_string())
}

pub(crate) fn render_identity_browser_html(
    bundle: &str,
    report: &IdentityWarningBrowserReport,
    filters: &IdentityWarningBrowserFilters,
) -> String {
    let mut out = String::new();
    out.push_str(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Fret Identity Warnings</title>
<style>
:root {
  color-scheme: light;
  --bg: #f5f7f8;
  --panel: #ffffff;
  --ink: #172026;
  --muted: #64727d;
  --line: #d8e0e5;
  --accent: #1d6f8f;
  --accent-weak: #dff1f6;
  --warn: #9f4f00;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--ink);
  font: 13px/1.45 system-ui, -apple-system, Segoe UI, sans-serif;
}
.shell {
  display: grid;
  grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);
  min-height: 100vh;
}
.groups {
  border-right: 1px solid var(--line);
  background: #eef3f5;
  padding: 16px;
  overflow: auto;
}
.content {
  padding: 18px 20px 24px;
  overflow: auto;
}
h1, h2 {
  margin: 0;
  letter-spacing: 0;
}
h1 { font-size: 20px; }
h2 { font-size: 13px; margin-bottom: 10px; color: var(--muted); text-transform: uppercase; }
.bundle {
  margin-top: 4px;
  color: var(--muted);
  word-break: break-all;
}
.summary {
  display: grid;
  grid-template-columns: repeat(5, minmax(104px, 1fr));
  gap: 8px;
  margin: 16px 0;
}
.metric, .group {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 6px;
}
.metric { padding: 10px; }
.metric strong { display: block; font-size: 18px; }
.metric span { color: var(--muted); }
.toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
  margin: 12px 0;
}
.toolbar input {
  width: min(560px, 100%);
  min-height: 34px;
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 6px 10px;
  font: inherit;
}
.filter-note { color: var(--muted); }
.group {
  width: 100%;
  padding: 10px;
  margin-bottom: 8px;
}
.group-kind { font-weight: 650; }
.group-meta {
  display: grid;
  gap: 2px;
  margin-top: 6px;
  color: var(--muted);
  word-break: break-word;
}
table {
  width: 100%;
  border-collapse: collapse;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 6px;
  overflow: hidden;
}
th, td {
  border-bottom: 1px solid var(--line);
  padding: 8px;
  text-align: left;
  vertical-align: top;
}
th {
  position: sticky;
  top: 0;
  background: #edf4f6;
  color: #344651;
  z-index: 1;
}
tbody tr:hover { background: var(--accent-weak); }
.mono { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
.muted { color: var(--muted); }
.kind { color: var(--accent); font-weight: 650; }
.warn { color: var(--warn); font-weight: 650; }
details { margin-top: 4px; }
summary { cursor: pointer; color: var(--accent); }
pre {
  max-width: 760px;
  overflow: auto;
  margin: 8px 0 0;
  padding: 10px;
  border: 1px solid var(--line);
  border-radius: 6px;
  background: #f8fafb;
}
@media (max-width: 900px) {
  .shell { grid-template-columns: 1fr; }
  .groups { border-right: 0; border-bottom: 1px solid var(--line); }
  .summary { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  table { font-size: 12px; }
}
</style>
</head>
<body>
<div class="shell">
"#,
    );
    out.push_str("<aside class=\"groups\">\n<h2>Groups</h2>\n");
    if report.groups.is_empty() {
        out.push_str("<p class=\"muted\">No groups</p>\n");
    } else {
        for group in &report.groups {
            let key = &group.key;
            let search = format!(
                "{} {} {} {} {} {:?}",
                key.kind,
                key.source_file.as_deref().unwrap_or(""),
                key.element_path.as_deref().unwrap_or(""),
                key.list_id.map_or(String::new(), |v| v.to_string()),
                key.key_hash.map_or(String::new(), |v| v.to_string()),
                key.frame_id
            );
            let _ = writeln!(
                out,
                "<div class=\"group\" data-search=\"{}\"><div class=\"group-kind\">{}</div><div class=\"group-meta\"><span>rows: {}</span><span>window: {}</span><span>frame: {}</span><span>file: {}</span><span>list: {}</span><span>key: {}</span></div></div>",
                escape_html(&search),
                escape_html(&key.kind),
                group.rows,
                key.window,
                fmt_opt_u64(key.frame_id),
                escape_html(key.source_file.as_deref().unwrap_or("unknown")),
                fmt_opt_u64(key.list_id),
                fmt_opt_u64(key.key_hash)
            );
        }
    }
    out.push_str("</aside>\n<main class=\"content\">\n<header>\n");
    let _ = writeln!(
        out,
        "<h1>Fret Identity Warnings</h1>\n<div class=\"bundle mono\">{}</div>",
        escape_html(bundle)
    );
    out.push_str("</header>\n<section class=\"summary\" aria-label=\"Summary\">\n");
    metric(&mut out, "Total", report.total_observations);
    metric(&mut out, "Matching", report.matching_observations);
    metric(&mut out, "Deduped", report.deduped_observations);
    metric(&mut out, "Returned", report.rows.len());
    metric(&mut out, "Groups", report.groups.len());
    out.push_str("</section>\n");
    let _ = writeln!(
        out,
        "<div class=\"filter-note\">window={} kind={} element={} list={} file={} path={} timeline={} top={}</div>",
        fmt_opt_u64(filters.window),
        escape_html(filters.kind.as_deref().unwrap_or("any")),
        fmt_opt_u64(filters.element),
        fmt_opt_u64(filters.list_id),
        escape_html(filters.file_contains.as_deref().unwrap_or("any")),
        escape_html(filters.element_path_contains.as_deref().unwrap_or("any")),
        filters.timeline,
        filters.top
    );
    out.push_str(
        r#"<div class="toolbar">
<input id="filter" type="search" placeholder="Filter kind, file, path, list id, key hash" autocomplete="off">
<span id="count" class="muted"></span>
</div>
<table>
<thead>
<tr>
<th>Kind</th>
<th>Window / Frame</th>
<th>Source</th>
<th>Element Path</th>
<th>List / Key</th>
<th>Details</th>
</tr>
</thead>
<tbody>
"#,
    );
    for row in &report.rows {
        let location = format!(
            "{}:{}:{}",
            row.location.file.as_deref().unwrap_or("unknown"),
            row.location.line.unwrap_or(0),
            row.location.column.unwrap_or(0)
        );
        let search = format!(
            "{} {} {} {} {} {}",
            row.kind.as_str(),
            location,
            row.element_path.as_deref().unwrap_or(""),
            row.list_id.map_or(String::new(), |v| v.to_string()),
            row.key_hash.map_or(String::new(), |v| v.to_string()),
            row.element.map_or(String::new(), |v| v.to_string())
        );
        let row_json =
            serde_json::to_string_pretty(&row.to_query_json()).unwrap_or_else(|_| "{}".to_string());
        let _ = writeln!(
            out,
            "<tr data-search=\"{}\"><td><span class=\"kind\">{}</span></td><td>window {}<br><span class=\"muted\">frame {} / snapshot {}</span></td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>list {}<br><span class=\"muted\">key {}</span></td><td>{}<details><summary>JSON</summary><pre>{}</pre></details></td></tr>",
            escape_html(&search),
            escape_html(row.kind.as_str()),
            row.window,
            fmt_opt_u64(row.frame_id),
            row.snapshot_frame_id,
            escape_html(&location),
            escape_html(row.element_path.as_deref().unwrap_or("unknown")),
            fmt_opt_u64(row.list_id),
            fmt_opt_u64(row.key_hash),
            detail_badges(row),
            escape_html(&row_json)
        );
    }
    out.push_str(
        r#"</tbody>
</table>
</main>
</div>
<script>
const input = document.getElementById('filter');
const count = document.getElementById('count');
const searchable = Array.from(document.querySelectorAll('[data-search]'));
function applyFilter() {
  const q = input.value.trim().toLowerCase();
  let visibleRows = 0;
  searchable.forEach((el) => {
    const hit = !q || el.dataset.search.toLowerCase().includes(q);
    el.style.display = hit ? '' : 'none';
    if (hit && el.tagName === 'TR') visibleRows += 1;
  });
  count.textContent = visibleRows + ' rows';
}
input.addEventListener('input', applyFilter);
applyFilter();
</script>
</body>
</html>
"#,
    );
    out
}

fn metric(out: &mut String, label: &str, value: usize) {
    let _ = writeln!(
        out,
        "<div class=\"metric\"><strong>{}</strong><span>{}</span></div>",
        value,
        escape_html(label)
    );
}

fn detail_badges(row: &crate::identity_browser::IdentityWarningBrowserRow) -> String {
    let mut parts = Vec::new();
    if let (Some(first), Some(second)) = (row.first_index, row.second_index) {
        parts.push(format!(
            "<span class=\"warn\">duplicate indices {} / {}</span>",
            first, second
        ));
    }
    if let (Some(previous), Some(next)) = (row.previous_len, row.next_len) {
        parts.push(format!("<span>len {} -> {}</span>", previous, next));
    }
    if parts.is_empty() {
        "<span class=\"muted\">n/a</span>".to_string()
    } else {
        parts.join("<br>")
    }
}

fn fmt_opt_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn escape_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity_browser::{
        IdentityWarningBrowserFilters, collect_identity_warning_browser_report,
    };

    fn html_fixture_bundle() -> serde_json::Value {
        serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1u64,
                "snapshots": [{
                    "tick_id": 10u64,
                    "frame_id": 20u64,
                    "window_snapshot_seq": 30u64,
                    "debug": {
                        "element_runtime": {
                            "identity_warnings": [{
                                "kind": "duplicate_keyed_list_item_key_hash",
                                "frame_id": 21u64,
                                "element": 456u64,
                                "element_path": "root.<panel>&item[key=0x2]",
                                "list_id": 42u64,
                                "key_hash": 9001u64,
                                "first_index": 1u64,
                                "second_index": 2u64,
                                "location": {
                                    "file": "src/<list>&view.rs",
                                    "line": 31u64,
                                    "column": 13u64
                                }
                            }]
                        }
                    }
                }]
            }]
        })
    }

    #[test]
    fn identity_browser_html_renders_summary_groups_rows_and_escapes_text() {
        let bundle = html_fixture_bundle();
        let filters = IdentityWarningBrowserFilters {
            top: 25,
            ..Default::default()
        };
        let report = collect_identity_warning_browser_report(&bundle, &filters);

        let html = render_identity_browser_html("target/<bundle>&.json", &report, &filters);

        assert!(html.contains("Fret Identity Warnings"));
        assert!(html.contains("<strong>1</strong><span>Total</span>"));
        assert!(html.contains("duplicate_keyed_list_item_key_hash"));
        assert!(html.contains("duplicate indices 1 / 2"));
        assert!(html.contains("src/&lt;list&gt;&amp;view.rs"));
        assert!(html.contains("root.&lt;panel&gt;&amp;item[key=0x2]"));
        assert!(html.contains("target/&lt;bundle&gt;&amp;.json"));
        assert!(!html.contains("src/<list>&view.rs"));
    }
}
