use std::path::Path;

use crate::util::touch;

#[derive(Debug, Clone)]
struct InspectConfigV1 {
    schema_version: u32,
    enabled: bool,
    consume_clicks: bool,
}

fn read_inspect_config(path: &Path) -> Option<InspectConfigV1> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("schema_version")?.as_u64()? != 1 {
        return None;
    }
    let enabled = v.get("enabled")?.as_bool()?;
    let consume_clicks = v
        .get("consume_clicks")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Some(InspectConfigV1 {
        schema_version: 1,
        enabled,
        consume_clicks,
    })
}

fn write_inspect_config(path: &Path, cfg: InspectConfigV1) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let v = serde_json::json!({
        "schema_version": cfg.schema_version,
        "enabled": cfg.enabled,
        "consume_clicks": cfg.consume_clicks,
    });
    let bytes = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

pub(crate) fn cmd_inspect(
    rest: &[String],
    inspect_path: &Path,
    inspect_trigger_path: &Path,
    inspect_consume_clicks: Option<bool>,
) -> Result<(), String> {
    let Some(action) = rest.first().cloned() else {
        return Err(
            "missing inspect action (try: fretboard diag inspect on|off|toggle|status)".to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    match action.as_str() {
        "status" => {
            let cfg = read_inspect_config(inspect_path);
            let (enabled, consume_clicks) = match cfg {
                Some(c) => (c.enabled, c.consume_clicks),
                None => (false, true),
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "enabled": enabled,
                "consume_clicks": consume_clicks,
                "inspect_path": inspect_path.display().to_string(),
                "inspect_trigger_path": inspect_trigger_path.display().to_string(),
            });
            println!(
                "{}",
                serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
            );
            Ok(())
        }
        "on" | "off" | "toggle" => {
            let prev = read_inspect_config(inspect_path);
            let prev_enabled = prev.as_ref().map(|c| c.enabled).unwrap_or(false);
            let prev_consume_clicks = prev.as_ref().map(|c| c.consume_clicks).unwrap_or(true);

            let next_enabled = match action.as_str() {
                "on" => true,
                "off" => false,
                "toggle" => !prev_enabled,
                _ => unreachable!(),
            };
            let next_consume_clicks = inspect_consume_clicks.unwrap_or(prev_consume_clicks);

            write_inspect_config(
                inspect_path,
                InspectConfigV1 {
                    schema_version: 1,
                    enabled: next_enabled,
                    consume_clicks: next_consume_clicks,
                },
            )?;
            touch(inspect_trigger_path)?;
            println!("{}", inspect_trigger_path.display());
            Ok(())
        }
        other => Err(format!("unknown inspect action: {other}")),
    }
}
