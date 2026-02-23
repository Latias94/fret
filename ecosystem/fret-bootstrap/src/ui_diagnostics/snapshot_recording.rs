use super::*;

pub(super) fn changed_model_sources_top(
    app: &App,
    changed_models: &[u64],
) -> Vec<UiChangedModelSourceHotspotV1> {
    if !cfg!(debug_assertions) || changed_models.is_empty() {
        return Vec::new();
    }

    let mut counts: HashMap<(String, String, u32, u32), u32> = HashMap::new();
    for &model in changed_models {
        let id = ModelId::from(KeyData::from_ffi(model));
        let Some(info) = app.models().debug_last_changed_info_for_id(id) else {
            continue;
        };
        let ty = info.type_name.to_string();
        *counts
            .entry((ty, info.file.to_string(), info.line, info.column))
            .or_insert(0) += 1;
    }

    let mut out: Vec<UiChangedModelSourceHotspotV1> = counts
        .into_iter()
        .map(
            |((type_name, file, line, column), count)| UiChangedModelSourceHotspotV1 {
                type_name,
                changed_at: UiSourceLocationV1 { file, line, column },
                count,
            },
        )
        .collect();
    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.type_name.cmp(&b.type_name))
            .then_with(|| a.changed_at.file.cmp(&b.changed_at.file))
            .then_with(|| a.changed_at.line.cmp(&b.changed_at.line))
            .then_with(|| a.changed_at.column.cmp(&b.changed_at.column))
    });
    out.truncate(8);
    out
}

