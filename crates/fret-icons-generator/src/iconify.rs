use std::collections::BTreeMap;

use crate::GeneratePackError;
use crate::contracts::{IconifyCollectionSource, PresentationRenderMode};
use crate::naming::normalize_icon_name;
use crate::svg_dir::CollectedSvg;

#[derive(Debug, serde::Deserialize)]
struct IconifyCollection {
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    icons: BTreeMap<String, IconifyIcon>,
    #[serde(default)]
    aliases: BTreeMap<String, IconifyAlias>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct IconifyIcon {
    body: String,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    rotate: Option<u8>,
    #[serde(default, rename = "hFlip")]
    h_flip: Option<bool>,
    #[serde(default, rename = "vFlip")]
    v_flip: Option<bool>,
    #[serde(default)]
    hidden: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct IconifyAlias {
    parent: String,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    rotate: Option<u8>,
    #[serde(default, rename = "hFlip")]
    h_flip: Option<bool>,
    #[serde(default, rename = "vFlip")]
    v_flip: Option<bool>,
    #[serde(default)]
    hidden: Option<bool>,
}

#[derive(Debug, Clone)]
struct ResolvedIcon {
    body: String,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    rotate: u8,
    h_flip: bool,
    v_flip: bool,
    hidden: bool,
}

pub(crate) fn collect_iconify_collection(
    source: &IconifyCollectionSource,
) -> Result<Vec<CollectedSvg>, GeneratePackError> {
    if !source.file.exists() {
        return Err(GeneratePackError::MissingSourceFile(
            source.file.display().to_string(),
        ));
    }
    if !source.file.is_file() {
        return Err(GeneratePackError::SourcePathNotFile(
            source.file.display().to_string(),
        ));
    }

    let content = std::fs::read_to_string(&source.file)?;
    let collection: IconifyCollection = serde_json::from_str(&content)?;
    if collection.icons.is_empty() && collection.aliases.is_empty() {
        return Err(GeneratePackError::EmptyIconifyCollection(
            source.file.display().to_string(),
        ));
    }

    let _ = collection.prefix.as_deref();

    let mut resolved = Vec::new();
    for icon_name in collection.icons.keys() {
        resolved.push(collect_named_icon(&collection, icon_name, "icons")?);
    }
    for alias_name in collection.aliases.keys() {
        resolved.push(collect_named_icon(&collection, alias_name, "aliases")?);
    }

    resolved.sort_by(|left, right| left.icon_name.cmp(&right.icon_name));
    for pair in resolved.windows(2) {
        if pair[0].icon_name == pair[1].icon_name {
            return Err(GeneratePackError::IconNameCollision {
                icon_name: pair[0].icon_name.clone(),
                first: pair[0].source_relative_path.clone(),
                second: pair[1].source_relative_path.clone(),
            });
        }
    }

    Ok(resolved)
}

fn collect_named_icon(
    collection: &IconifyCollection,
    icon_name: &str,
    source_group: &str,
) -> Result<CollectedSvg, GeneratePackError> {
    let normalized_name = normalize_icon_name(icon_name)?;
    let resolved_icon = resolve_icon(collection, icon_name, &mut Vec::new())?;
    let svg = render_svg(&resolved_icon);
    Ok(CollectedSvg {
        icon_name: normalized_name,
        source_relative_path: format!("{source_group}/{icon_name}"),
        render_mode: PresentationRenderMode::Mask,
        svg_bytes: svg.into_bytes(),
    })
}

fn resolve_icon(
    collection: &IconifyCollection,
    icon_name: &str,
    chain: &mut Vec<String>,
) -> Result<ResolvedIcon, GeneratePackError> {
    if chain.iter().any(|entry| entry == icon_name) {
        let mut loop_chain = chain.clone();
        loop_chain.push(icon_name.to_string());
        return Err(GeneratePackError::IconifyAliasLoop {
            chain: loop_chain.join(" -> "),
        });
    }
    chain.push(icon_name.to_string());

    let resolved = if let Some(icon) = collection.icons.get(icon_name) {
        ResolvedIcon {
            body: icon.body.clone(),
            left: icon.left.or(collection.left).unwrap_or(0.0),
            top: icon.top.or(collection.top).unwrap_or(0.0),
            width: icon.width.or(collection.width).unwrap_or(24.0),
            height: icon.height.or(collection.height).unwrap_or(24.0),
            rotate: icon.rotate.unwrap_or(0) % 4,
            h_flip: icon.h_flip.unwrap_or(false),
            v_flip: icon.v_flip.unwrap_or(false),
            hidden: icon.hidden.unwrap_or(false),
        }
    } else if let Some(alias) = collection.aliases.get(icon_name) {
        let parent = resolve_icon(collection, &alias.parent, chain)?;
        ResolvedIcon {
            body: parent.body,
            left: alias.left.unwrap_or(parent.left),
            top: alias.top.unwrap_or(parent.top),
            width: alias.width.unwrap_or(parent.width),
            height: alias.height.unwrap_or(parent.height),
            rotate: (parent.rotate + alias.rotate.unwrap_or(0)) % 4,
            h_flip: parent.h_flip ^ alias.h_flip.unwrap_or(false),
            v_flip: parent.v_flip ^ alias.v_flip.unwrap_or(false),
            hidden: alias.hidden.unwrap_or(parent.hidden),
        }
    } else {
        return Err(GeneratePackError::MissingIconifyParent {
            icon_name: icon_name.to_string(),
        });
    };

    chain.pop();
    Ok(resolved)
}

fn render_svg(icon: &ResolvedIcon) -> String {
    let (view_left, view_top, view_width, view_height) =
        view_box_after_rotation(icon.left, icon.top, icon.width, icon.height, icon.rotate);
    let mut body = icon.body.clone();

    if icon.h_flip {
        let tx = 2.0 * icon.left + icon.width;
        body = wrap_transform(&body, &format!("translate({} 0) scale(-1 1)", fmt_num(tx)));
    }
    if icon.v_flip {
        let ty = 2.0 * icon.top + icon.height;
        body = wrap_transform(&body, &format!("translate(0 {}) scale(1 -1)", fmt_num(ty)));
    }
    if icon.rotate != 0 {
        let cx = icon.left + icon.width / 2.0;
        let cy = icon.top + icon.height / 2.0;
        let angle = icon.rotate as u16 * 90;
        body = wrap_transform(
            &body,
            &format!("rotate({angle} {} {})", fmt_num(cx), fmt_num(cy)),
        );
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" data-hidden="{}">{}</svg>"#,
        fmt_num(view_left),
        fmt_num(view_top),
        fmt_num(view_width),
        fmt_num(view_height),
        icon.hidden,
        body
    )
}

fn wrap_transform(body: &str, transform: &str) -> String {
    format!(r#"<g transform="{transform}">{body}</g>"#)
}

fn view_box_after_rotation(
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    rotate: u8,
) -> (f64, f64, f64, f64) {
    if rotate % 2 == 0 {
        return (left, top, width, height);
    }

    let delta = (height - width) / 2.0;
    (left - delta, top + delta, height, width)
}

fn fmt_num(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        let mut text = format!("{value:.4}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::collect_iconify_collection;
    use crate::GeneratePackError;
    use crate::contracts::IconifyCollectionSource;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn iconify_collection_supports_icons_and_aliases() {
        let root = make_temp_dir("fret-icons-generator-iconify");
        let source_file = root.join("demo.json");
        std::fs::write(
            &source_file,
            r#"{
  "prefix": "demo",
  "width": 24,
  "height": 24,
  "icons": {
    "search": { "body": "<path d='M10 10h4'/>" }
  },
  "aliases": {
    "search-rotated": { "parent": "search", "rotate": 1 }
  }
}"#,
        )
        .expect("write iconify snapshot");

        let icons = collect_iconify_collection(&IconifyCollectionSource {
            file: source_file,
            label: "demo-iconify".to_string(),
        })
        .expect("iconify collection should load");

        assert_eq!(icons.len(), 2);
        assert_eq!(icons[0].icon_name, "search");
        assert_eq!(icons[0].source_relative_path, "icons/search");
        assert_eq!(icons[1].icon_name, "search-rotated");
        assert_eq!(icons[1].source_relative_path, "aliases/search-rotated");
        let rotated_svg = String::from_utf8(icons[1].svg_bytes.clone()).expect("utf8 svg");
        assert!(rotated_svg.contains("rotate(90 12 12)"));
    }

    #[test]
    fn iconify_collection_preserves_multicolor_svg_body() {
        let root = make_temp_dir("fret-icons-generator-iconify-multicolor");
        let source_file = root.join("demo.json");
        std::fs::write(
            &source_file,
            r#"{
  "prefix": "demo",
  "width": 24,
  "height": 24,
  "icons": {
    "palette": {
      "body": "<path fill='#ff0000' d='M0 0h12v24H0z'/><path fill='#0000ff' d='M12 0h12v24H12z'/>"
    }
  }
}"#,
        )
        .expect("write iconify snapshot");

        let icons = collect_iconify_collection(&IconifyCollectionSource {
            file: source_file,
            label: "demo-iconify".to_string(),
        })
        .expect("iconify collection should load");

        let svg = String::from_utf8(icons[0].svg_bytes.clone()).expect("utf8 svg");
        assert!(svg.contains("fill='#ff0000'"));
        assert!(svg.contains("fill='#0000ff'"));
    }

    #[test]
    fn iconify_collection_rejects_alias_loops() {
        let root = make_temp_dir("fret-icons-generator-iconify-loop");
        let source_file = root.join("demo.json");
        std::fs::write(
            &source_file,
            r#"{
  "prefix": "demo",
  "icons": {},
  "aliases": {
    "a": { "parent": "b" },
    "b": { "parent": "a" }
  }
}"#,
        )
        .expect("write iconify snapshot");

        let err = collect_iconify_collection(&IconifyCollectionSource {
            file: source_file,
            label: "demo-iconify".to_string(),
        })
        .expect_err("alias loop should fail");

        assert!(matches!(err, GeneratePackError::IconifyAliasLoop { .. }));
    }
}
