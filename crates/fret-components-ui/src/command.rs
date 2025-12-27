use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: Arc<str>,
    pub label: Arc<str>,
    pub keywords: Vec<Arc<str>>,
    pub detail: Option<Arc<str>>,
    pub shortcut: Option<Arc<str>>,
    pub group: Option<Arc<str>>,
    pub enabled: bool,
}

impl CommandItem {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            keywords: Vec::new(),
            detail: None,
            shortcut: None,
            group: None,
            enabled: true,
        }
    }

    pub fn keyword(mut self, keyword: impl Into<Arc<str>>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<Arc<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn group(mut self, group: impl Into<Arc<str>>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

fn matches_query(item: &CommandItem, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.trim();
    if q.is_empty() {
        return true;
    }

    let label = item.label.as_ref().to_ascii_lowercase();
    if label.contains(q) {
        return true;
    }
    item.keywords
        .iter()
        .any(|k| k.as_ref().to_ascii_lowercase().contains(q))
}

pub fn visible_item_ids(items: &[CommandItem], query: &str) -> Vec<Arc<str>> {
    let q = query.trim().to_ascii_lowercase();
    let mut filtered: Vec<CommandItem> = items
        .iter()
        .filter(|i| matches_query(i, &q))
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        let ag = a.group.as_deref().unwrap_or("");
        let bg = b.group.as_deref().unwrap_or("");
        ag.cmp(bg)
            .then_with(|| a.label.as_ref().cmp(b.label.as_ref()))
    });

    filtered
        .into_iter()
        .filter(|i| i.enabled)
        .map(|i| i.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_item_ids_sorts_by_group_and_label_and_skips_disabled() {
        let items = vec![
            CommandItem::new("b", "Bravo").group("B"),
            CommandItem::new("a", "Alpha").group("A"),
            CommandItem::new("a2", "Alpha 2").group("A"),
            CommandItem::new("x", "Disabled").group("A").disabled(),
        ];

        let ids = visible_item_ids(&items, "");
        assert_eq!(
            ids,
            vec![
                Arc::<str>::from("a"),
                Arc::<str>::from("a2"),
                Arc::<str>::from("b")
            ]
        );
    }
}
