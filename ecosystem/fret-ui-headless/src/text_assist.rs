//! Headless text-assist list controller for editor completion/history surfaces.
//!
//! This intentionally sits below any concrete UI:
//! - query/filter/highlight math is deterministic and reusable,
//! - active-item navigation skips disabled rows,
//! - and callers remain free to render the result as a popup, inline history list, or command-like
//!   palette while wiring focus/overlay semantics in `fret-ui-kit` / recipe crates.

use std::sync::Arc;

use crate::{cmdk_score, cmdk_selection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAssistItem {
    pub id: Arc<str>,
    pub label: Arc<str>,
    pub aliases: Arc<[Arc<str>]>,
    pub disabled: bool,
}

impl TextAssistItem {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aliases: Arc::from([]),
            disabled: false,
        }
    }

    pub fn aliases(mut self, aliases: impl Into<Arc<[Arc<str>]>>) -> Self {
        self.aliases = aliases.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAssistMatchMode {
    #[default]
    Prefix,
    CmdkFuzzy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAssistMove {
    Next,
    Previous,
    First,
    Last,
    PageDown { amount: usize },
    PageUp { amount: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextAssistMatch {
    pub item_id: Arc<str>,
    pub label: Arc<str>,
    pub score: f32,
    pub source_index: usize,
    pub disabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TextAssistController {
    mode: TextAssistMatchMode,
    wrap_navigation: bool,
    active_item_id: Option<Arc<str>>,
    visible: Vec<TextAssistMatch>,
}

impl TextAssistController {
    pub fn new(mode: TextAssistMatchMode) -> Self {
        Self {
            mode,
            wrap_navigation: false,
            active_item_id: None,
            visible: Vec::new(),
        }
    }

    pub fn with_wrap_navigation(mut self, wrap_navigation: bool) -> Self {
        self.wrap_navigation = wrap_navigation;
        self
    }

    pub fn mode(&self) -> TextAssistMatchMode {
        self.mode
    }

    pub fn visible(&self) -> &[TextAssistMatch] {
        &self.visible
    }

    pub fn active_item_id(&self) -> Option<&Arc<str>> {
        self.active_item_id.as_ref()
    }

    pub fn active_match(&self) -> Option<&TextAssistMatch> {
        let active = self.active_item_id.as_ref()?;
        self.visible.iter().find(|entry| &entry.item_id == active)
    }

    pub fn rebuild(&mut self, items: &[TextAssistItem], query: &str) {
        self.visible = build_visible_matches(items, query, self.mode);
        self.active_item_id = resolve_active_item_id(&self.visible, self.active_item_id.as_deref());
    }

    pub fn set_active_item_id(&mut self, item_id: Option<impl Into<Arc<str>>>) {
        let next = item_id.map(Into::into);
        self.active_item_id = resolve_active_item_id(&self.visible, next.as_deref());
    }

    pub fn move_active(&mut self, movement: TextAssistMove) {
        let disabled: Vec<bool> = self.visible.iter().map(|entry| entry.disabled).collect();
        let current = self.active_match().map(|entry| {
            self.visible
                .iter()
                .position(|candidate| candidate.item_id == entry.item_id)
                .expect("active match must exist in visible list")
        });

        let next = match movement {
            TextAssistMove::Next => {
                cmdk_selection::next_active_index(&disabled, current, true, self.wrap_navigation)
            }
            TextAssistMove::Previous => {
                cmdk_selection::next_active_index(&disabled, current, false, self.wrap_navigation)
            }
            TextAssistMove::First => cmdk_selection::first_enabled(&disabled),
            TextAssistMove::Last => cmdk_selection::last_enabled(&disabled),
            TextAssistMove::PageDown { amount } => cmdk_selection::advance_active_index(
                &disabled,
                current,
                true,
                self.wrap_navigation,
                amount.max(1),
            ),
            TextAssistMove::PageUp { amount } => cmdk_selection::advance_active_index(
                &disabled,
                current,
                false,
                self.wrap_navigation,
                amount.max(1),
            ),
        };

        self.active_item_id =
            next.and_then(|idx| self.visible.get(idx).map(|entry| entry.item_id.clone()));
    }
}

pub fn build_visible_matches(
    items: &[TextAssistItem],
    query: &str,
    mode: TextAssistMatchMode,
) -> Vec<TextAssistMatch> {
    let query = query.trim();
    let mut out: Vec<TextAssistMatch> = items
        .iter()
        .enumerate()
        .filter_map(|(source_index, item)| match score_item(item, query, mode) {
            Some(score) => Some(TextAssistMatch {
                item_id: item.id.clone(),
                label: item.label.clone(),
                score,
                source_index,
                disabled: item.disabled,
            }),
            None => None,
        })
        .collect();

    if matches!(mode, TextAssistMatchMode::CmdkFuzzy) && !query.is_empty() {
        out.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.label.as_ref().cmp(b.label.as_ref()))
                .then_with(|| a.source_index.cmp(&b.source_index))
        });
    }

    out
}

fn score_item(item: &TextAssistItem, query: &str, mode: TextAssistMatchMode) -> Option<f32> {
    if query.is_empty() {
        return Some(1.0);
    }

    match mode {
        TextAssistMatchMode::Prefix => prefix_matches(item, query).then_some(1.0),
        TextAssistMatchMode::CmdkFuzzy => {
            let aliases: Vec<&str> = item.aliases.iter().map(|alias| alias.as_ref()).collect();
            let score = cmdk_score::command_score(item.label.as_ref(), query, &aliases);
            (score > 0.0).then_some(score)
        }
    }
}

fn prefix_matches(item: &TextAssistItem, query: &str) -> bool {
    if starts_with_case_folded(item.label.as_ref(), query) {
        return true;
    }

    item.aliases
        .iter()
        .any(|alias| starts_with_case_folded(alias.as_ref(), query))
}

fn starts_with_case_folded(haystack: &str, needle: &str) -> bool {
    haystack
        .trim_start()
        .to_ascii_lowercase()
        .starts_with(&needle.to_ascii_lowercase())
}

fn resolve_active_item_id(visible: &[TextAssistMatch], current: Option<&str>) -> Option<Arc<str>> {
    if let Some(current) = current
        && visible
            .iter()
            .any(|entry| !entry.disabled && entry.item_id.as_ref() == current)
    {
        return Some(Arc::from(current));
    }

    visible
        .iter()
        .find(|entry| !entry.disabled)
        .map(|entry| entry.item_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<TextAssistItem> {
        vec![
            TextAssistItem::new("alpha", "Alpha"),
            TextAssistItem::new("beta", "Beta").aliases(Arc::from([Arc::from("Second")])),
            TextAssistItem::new("alpine", "Alpine").disabled(true),
            TextAssistItem::new("gamma", "Gamma"),
        ]
    }

    #[test]
    fn prefix_mode_matches_labels_and_aliases_in_input_order() {
        let items = sample_items();
        let matches = build_visible_matches(&items, "se", TextAssistMatchMode::Prefix);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].item_id.as_ref(), "beta");
        assert_eq!(matches[0].source_index, 1);
    }

    #[test]
    fn fuzzy_mode_ranks_matches_by_score() {
        let items = vec![
            TextAssistItem::new("open-file", "Open File"),
            TextAssistItem::new("open-folder", "Open Folder"),
            TextAssistItem::new("close", "Close"),
        ];

        let matches = build_visible_matches(&items, "opf", TextAssistMatchMode::CmdkFuzzy);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].item_id.as_ref(), "open-file");
        assert!(matches[0].score >= matches[1].score);
    }

    #[test]
    fn rebuild_preserves_active_item_when_still_visible_and_enabled() {
        let items = sample_items();
        let mut controller = TextAssistController::new(TextAssistMatchMode::Prefix);
        controller.rebuild(&items, "");
        controller.set_active_item_id(Some(Arc::<str>::from("gamma")));

        controller.rebuild(&items, "g");

        assert_eq!(
            controller.active_item_id().map(|id| id.as_ref()),
            Some("gamma")
        );
    }

    #[test]
    fn rebuild_clamps_active_item_when_previous_match_disappears() {
        let items = sample_items();
        let mut controller = TextAssistController::new(TextAssistMatchMode::Prefix);
        controller.rebuild(&items, "");
        controller.set_active_item_id(Some(Arc::<str>::from("beta")));

        controller.rebuild(&items, "ga");

        assert_eq!(
            controller.active_item_id().map(|id| id.as_ref()),
            Some("gamma")
        );
    }

    #[test]
    fn navigation_skips_disabled_entries() {
        let items = sample_items();
        let mut controller = TextAssistController::new(TextAssistMatchMode::Prefix);
        controller.rebuild(&items, "a");

        assert_eq!(
            controller.active_item_id().map(|id| id.as_ref()),
            Some("alpha")
        );

        controller.move_active(TextAssistMove::Next);

        assert_eq!(
            controller.active_item_id(),
            Some(&Arc::<str>::from("alpha"))
        );
    }

    #[test]
    fn page_navigation_uses_headless_selection_math() {
        let items = vec![
            TextAssistItem::new("a", "Alpha"),
            TextAssistItem::new("b", "Beta"),
            TextAssistItem::new("c", "Gamma"),
            TextAssistItem::new("d", "Delta"),
        ];
        let mut controller =
            TextAssistController::new(TextAssistMatchMode::Prefix).with_wrap_navigation(true);
        controller.rebuild(&items, "");

        controller.move_active(TextAssistMove::PageDown { amount: 2 });

        assert_eq!(controller.active_item_id().map(|id| id.as_ref()), Some("c"));

        controller.move_active(TextAssistMove::PageUp { amount: 1 });

        assert_eq!(controller.active_item_id().map(|id| id.as_ref()), Some("b"));
    }
}
