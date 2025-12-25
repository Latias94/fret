use crate::{
    Theme, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Effect, InputContext, Menu, MenuItem};
use std::{borrow::Cow, collections::HashSet, hash::Hash, sync::Arc};

use crate::ContextMenuRequest;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtualListRowHeight {
    Fixed(Px),
    Measured { min: Px },
}

impl Default for VirtualListRowHeight {
    fn default() -> Self {
        Self::Fixed(Px(20.0))
    }
}

#[derive(Debug, Default, Clone)]
struct FenwickTree {
    tree: Vec<f32>,
}

impl FenwickTree {
    fn lowbit(i: usize) -> usize {
        i & i.wrapping_neg()
    }

    fn rebuild_from_px(&mut self, values: &[Px]) {
        let n = values.len();
        self.tree.clear();
        self.tree.resize(n + 1, 0.0);
        for (i, v) in values.iter().enumerate() {
            self.tree[i + 1] = v.0;
        }
        for i in 1..=n {
            let j = i + Self::lowbit(i);
            if j <= n {
                self.tree[j] += self.tree[i];
            }
        }
    }

    fn total(&self) -> Px {
        Px(self.prefix_sum(self.tree.len().saturating_sub(1)))
    }

    fn prefix_sum(&self, end: usize) -> f32 {
        let mut i = end.min(self.tree.len().saturating_sub(1));
        let mut sum = 0.0;
        while i > 0 {
            sum += self.tree[i];
            i &= i - 1;
        }
        sum
    }

    fn offset_of_index(&self, index: usize) -> Px {
        Px(self.prefix_sum(index))
    }

    fn add(&mut self, index: usize, delta: f32) {
        let mut i = index + 1;
        while i < self.tree.len() {
            self.tree[i] += delta;
            i += Self::lowbit(i);
        }
    }

    fn lower_bound(&self, target: f32) -> usize {
        let n = self.tree.len().saturating_sub(1);
        if n == 0 {
            return 0;
        }

        let mut idx = 0usize;
        let mut bit = 1usize;
        while bit << 1 <= n {
            bit <<= 1;
        }

        let mut acc = 0.0f32;
        let mut b = bit;
        while b != 0 {
            let next = idx + b;
            if next <= n && acc + self.tree[next] <= target {
                acc += self.tree[next];
                idx = next;
            }
            b >>= 1;
        }

        idx
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListStyle {
    pub padding_x: Px,
    pub padding_y: Px,
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    pub row_hover: Color,
    pub row_selected: Color,
    pub row_highlight_inset_y: Px,
    pub text_color: Color,
    pub text_style: TextStyle,
    pub secondary_text_color: Color,
    pub secondary_text_style: TextStyle,
    pub trailing_text_color: Color,
    pub trailing_text_style: TextStyle,
    pub row_gap_y: Px,
    pub trailing_gap_x: Px,
    pub separator_color: Color,
    pub separator_inset_x: Px,
    pub header_text_color: Color,
    pub header_text_style: TextStyle,
    pub wrap: TextWrap,
}

impl Default for VirtualListStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(8.0),
            padding_y: Px(2.0),
            background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            corner_radii: Corners::all(Px(8.0)),
            row_hover: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            row_selected: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            row_highlight_inset_y: Px(0.0),
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
            secondary_text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.70,
            },
            secondary_text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
            trailing_text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.70,
            },
            trailing_text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
            row_gap_y: Px(2.0),
            trailing_gap_x: Px(8.0),
            separator_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.12,
            },
            separator_inset_x: Px(8.0),
            header_text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.85,
            },
            header_text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
            wrap: TextWrap::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualListRowKind {
    Item,
    Header,
    Separator,
}

#[derive(Debug, Clone)]
pub struct VirtualListRow<'a> {
    pub text: Cow<'a, str>,
    pub secondary_text: Option<Cow<'a, str>>,
    pub trailing_text: Option<Cow<'a, str>>,
    pub leading_text: Option<Cow<'a, str>>,
    pub indent_x: Px,
    pub enabled: bool,
    pub selectable: bool,
    pub kind: VirtualListRowKind,
}

impl<'a> VirtualListRow<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            text: text.into(),
            secondary_text: None,
            trailing_text: None,
            leading_text: None,
            indent_x: Px(0.0),
            enabled: true,
            selectable: true,
            kind: VirtualListRowKind::Item,
        }
    }

    pub fn with_indent_x(mut self, indent_x: Px) -> Self {
        self.indent_x = indent_x;
        self
    }

    pub fn with_secondary_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }

    pub fn with_trailing_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }

    pub fn with_leading_text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.leading_text = Some(text.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn not_selectable(mut self) -> Self {
        self.selectable = false;
        self
    }

    pub fn header(mut self) -> Self {
        self.kind = VirtualListRowKind::Header;
        self.selectable = false;
        self
    }

    pub fn separator() -> Self {
        Self {
            text: Cow::Borrowed(""),
            secondary_text: None,
            trailing_text: None,
            leading_text: None,
            indent_x: Px(0.0),
            enabled: false,
            selectable: false,
            kind: VirtualListRowKind::Separator,
        }
    }
}

pub trait VirtualListDataSource {
    type Key: Copy + Eq + Hash;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn key_at(&self, index: usize) -> Self::Key;
    fn row_at(&self, index: usize) -> VirtualListRow<'_>;

    fn index_of_key(&self, key: Self::Key) -> Option<usize> {
        let len = self.len();
        (0..len).find(|&i| self.key_at(i) == key)
    }
}

#[derive(Debug, Clone)]
pub struct VecStringDataSource {
    items: Vec<String>,
}

impl VecStringDataSource {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl VirtualListDataSource for VecStringDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        self.items.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        VirtualListRow::new(self.items[index].as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VisibleRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct PreparedRow<K> {
    index: usize,
    key: K,
    indent_x: Px,
    kind: VirtualListRowKind,
    enabled: bool,
    selectable: bool,
    leading_blob: Option<fret_core::TextBlobId>,
    leading_metrics: Option<fret_core::TextMetrics>,
    primary_blob: fret_core::TextBlobId,
    primary_metrics: fret_core::TextMetrics,
    secondary_blob: Option<fret_core::TextBlobId>,
    secondary_metrics: Option<fret_core::TextMetrics>,
    trailing_blob: Option<fret_core::TextBlobId>,
    trailing_metrics: Option<fret_core::TextMetrics>,
    height: Px,
}

#[derive(Debug)]
pub struct VirtualList<D: VirtualListDataSource> {
    data: D,
    row_height: VirtualListRowHeight,
    style: VirtualListStyle,
    style_override: bool,
    last_theme_revision: Option<u64>,
    scrollbar_width: Px,

    offset_y: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,

    hovered: Option<usize>,
    selected_keys: HashSet<D::Key>,
    selection_anchor: Option<D::Key>,
    selection_lead: Option<D::Key>,
    selection_lead_index: Option<usize>,
    context_menu_target: Option<D::Key>,

    last_bounds: Rect,
    last_content_height: Px,
    last_viewport_height: Px,
    last_visible: VisibleRange,
    prepared: Vec<PreparedRow<D::Key>>,
    last_prepared_width: Px,
    prepared_dirty: bool,

    measured_heights_by_key: std::collections::HashMap<D::Key, Px>,
    heights_by_index: Vec<Px>,
    heights_tree: FenwickTree,
    heights_dirty: bool,
    last_height_width: Px,
    last_height_scale_factor: Option<f32>,
    last_height_theme_revision: Option<u64>,
}

impl VirtualList<VecStringDataSource> {
    pub fn from_items(items: Vec<String>) -> Self {
        Self::new(VecStringDataSource::new(items))
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.set_data(VecStringDataSource::new(items));
    }
}

impl<D: VirtualListDataSource> VirtualList<D> {
    pub fn new(data: D) -> Self {
        Self {
            data,
            row_height: VirtualListRowHeight::default(),
            style: VirtualListStyle::default(),
            style_override: false,
            last_theme_revision: None,
            scrollbar_width: Px(10.0),
            offset_y: Px(0.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            hovered: None,
            selected_keys: HashSet::new(),
            selection_anchor: None,
            selection_lead: None,
            selection_lead_index: None,
            context_menu_target: None,
            last_bounds: Rect::default(),
            last_content_height: Px(0.0),
            last_viewport_height: Px(0.0),
            last_visible: VisibleRange { start: 0, end: 0 },
            prepared: Vec::new(),
            last_prepared_width: Px(0.0),
            prepared_dirty: false,

            measured_heights_by_key: std::collections::HashMap::new(),
            heights_by_index: Vec::new(),
            heights_tree: FenwickTree::default(),
            heights_dirty: true,
            last_height_width: Px(0.0),
            last_height_scale_factor: None,
            last_height_theme_revision: None,
        }
    }

    pub fn with_row_height(mut self, height: VirtualListRowHeight) -> Self {
        self.row_height = height;
        self.heights_dirty = true;
        self
    }

    pub fn set_row_height(&mut self, height: VirtualListRowHeight) {
        if self.row_height == height {
            return;
        }
        self.row_height = height;
        self.prepared_dirty = true;
        self.heights_dirty = true;
        if !matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            self.measured_heights_by_key.clear();
        }
        self.clamp_offset();
    }

    pub fn with_wrap(mut self, wrap: TextWrap) -> Self {
        self.style.wrap = wrap;
        self.heights_dirty = true;
        self
    }

    pub fn set_wrap(&mut self, wrap: TextWrap) {
        if self.style.wrap == wrap {
            return;
        }
        self.style.wrap = wrap;
        self.prepared_dirty = true;
        self.heights_dirty = true;
        self.measured_heights_by_key.clear();
        self.clamp_offset();
    }

    pub fn invalidate_measurements(&mut self) {
        if !matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            return;
        }
        self.measured_heights_by_key.clear();
        self.prepared_dirty = true;
        self.heights_dirty = true;
        self.clamp_offset();
    }

    pub fn with_style(mut self, style: VirtualListStyle) -> Self {
        self.style = style;
        self.style_override = true;
        self
    }

    pub fn set_style(&mut self, style: VirtualListStyle) {
        self.style = style;
        self.style_override = true;
        self.prepared_dirty = true;
        self.heights_dirty = true;
        if matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            self.measured_heights_by_key.clear();
        }
        self.clamp_offset();
    }

    pub fn style(&self) -> &VirtualListStyle {
        &self.style
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        self.scrollbar_width = theme.metrics.scrollbar_width;

        if self.style_override {
            return;
        }
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.style.padding_x = theme
            .metric_by_key("metric.list.padding_x")
            .unwrap_or(theme.metrics.padding_md);
        self.style.background = theme.colors.list_background;
        self.style.border_color = theme.colors.list_border;
        self.style.corner_radii = Corners::all(theme.metrics.radius_md);
        self.style.row_hover = theme.colors.list_row_hover;
        self.style.row_selected = theme.colors.list_row_selected;
        self.style.row_highlight_inset_y = theme
            .metric_by_key("metric.list.row_highlight_inset_y")
            .unwrap_or(Px(0.0));
        self.style.text_color = theme.colors.text_primary;
        self.style.secondary_text_color = theme.colors.text_muted;
        self.style.trailing_text_color = theme.colors.text_muted;
        self.style.header_text_color = theme.colors.text_muted;
        self.style.separator_color = theme.colors.panel_border;
        self.style.padding_y = theme
            .metric_by_key("metric.list.padding_y")
            .unwrap_or(theme.metrics.padding_sm);
        self.style.row_gap_y = theme
            .metric_by_key("metric.list.row_gap_y")
            .unwrap_or(theme.metrics.padding_sm);
        self.style.trailing_gap_x = theme
            .metric_by_key("metric.list.trailing_gap_x")
            .unwrap_or(theme.metrics.padding_sm);
        self.style.separator_inset_x = theme
            .metric_by_key("metric.list.separator_inset_x")
            .unwrap_or(theme.metrics.padding_md);

        let text_px = theme
            .metric_by_key("font.size")
            .unwrap_or(theme.metrics.font_size);
        self.style.text_style.size = text_px;
        let small_px = Px((text_px.0 - 1.0).max(0.0));
        self.style.secondary_text_style.size = small_px;
        self.style.trailing_text_style.size = small_px;
        self.style.header_text_style.size = small_px;

        // Font and row height participate in measurement; force height cache refresh.
        self.heights_dirty = true;
        if matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            self.measured_heights_by_key.clear();
        }
    }

    pub fn row_height(&self) -> VirtualListRowHeight {
        self.row_height
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn selected_lead_key(&self) -> Option<D::Key> {
        self.selection_lead
    }

    pub fn selected_keys(&self) -> &HashSet<D::Key> {
        &self.selected_keys
    }

    pub fn clear_selection(&mut self) {
        self.selected_keys.clear();
        self.selection_anchor = None;
        self.selection_lead = None;
        self.selection_lead_index = None;
    }

    pub fn set_selected_key(&mut self, key: Option<D::Key>) {
        self.selected_keys.clear();
        if let Some(key) = key {
            if !self.is_key_selectable(key) {
                self.selection_anchor = None;
                self.selection_lead = None;
                self.selection_lead_index = None;
                return;
            }
            self.selected_keys.insert(key);
            self.selection_anchor = Some(key);
            self.selection_lead = Some(key);
            self.selection_lead_index = self.data.index_of_key(key);
        } else {
            self.selection_anchor = None;
            self.selection_lead = None;
            self.selection_lead_index = None;
        }
    }

    pub fn set_selected_keys(
        &mut self,
        keys: impl IntoIterator<Item = D::Key>,
        lead: Option<D::Key>,
    ) {
        self.selected_keys.clear();
        for k in keys {
            if self.is_key_selectable(k) {
                self.selected_keys.insert(k);
            }
        }

        let lead = lead.filter(|k| self.selected_keys.contains(k));
        let lead = lead.or_else(|| self.selected_keys.iter().next().copied());

        self.selection_anchor = lead;
        self.selection_lead = lead;
        self.selection_lead_index = lead.and_then(|k| self.data.index_of_key(k));
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn set_data(&mut self, data: D) {
        self.data = data;
        self.hovered = None;
        self.prepared_dirty = true;
        self.heights_dirty = true;
        let prev_selected = std::mem::take(&mut self.selected_keys);
        for key in prev_selected {
            let Some(index) = self.data.index_of_key(key) else {
                continue;
            };
            let row = self.data.row_at(index);
            if row.enabled && row.selectable && row.kind == VirtualListRowKind::Item {
                self.selected_keys.insert(key);
            }
        }
        if let Some(anchor) = self.selection_anchor {
            let ok = self
                .data
                .index_of_key(anchor)
                .is_some_and(|index| self.is_index_selectable(index));
            if !ok {
                self.selection_anchor = None;
            }
        }
        if let Some(lead) = self.selection_lead {
            let index = self.data.index_of_key(lead);
            if index.is_some_and(|i| self.is_index_selectable(i)) {
                self.selection_lead_index = index;
            } else {
                self.selection_lead = None;
                self.selection_lead_index = None;
            }
        } else {
            self.selection_lead_index = None;
        }
        self.clamp_offset();
    }

    fn lead_index(&self) -> Option<usize> {
        self.selection_lead_index
            .or_else(|| self.selection_lead.and_then(|k| self.data.index_of_key(k)))
    }

    fn is_index_selectable(&self, index: usize) -> bool {
        if index >= self.data.len() {
            return false;
        }
        let row = self.data.row_at(index);
        row.enabled && row.selectable && row.kind == VirtualListRowKind::Item
    }

    fn is_key_selectable(&self, key: D::Key) -> bool {
        let Some(index) = self.data.index_of_key(key) else {
            return false;
        };
        self.is_index_selectable(index)
    }

    fn set_lead_index(&mut self, index: usize) {
        let key = self.data.key_at(index);
        self.selection_lead_index = Some(index);
        self.selection_lead = Some(key);
    }

    fn select_range(&mut self, a: usize, b: usize, extend: bool) {
        let start = a.min(b);
        let end = a.max(b);
        if !extend {
            self.selected_keys.clear();
        }
        for i in start..=end {
            if self.is_index_selectable(i) {
                self.selected_keys.insert(self.data.key_at(i));
            }
        }
    }

    fn apply_click_selection(&mut self, index: usize, modifiers: Modifiers) {
        if !self.is_index_selectable(index) {
            return;
        }
        let clicked = self.data.key_at(index);

        if modifiers.shift {
            let anchor_key = self
                .selection_anchor
                .or(self.selection_lead)
                .unwrap_or(clicked);
            let anchor_index = self.data.index_of_key(anchor_key).unwrap_or(index);
            let extend = modifiers.ctrl || modifiers.meta;
            self.select_range(anchor_index, index, extend);
            self.selection_lead = Some(clicked);
            self.selection_lead_index = Some(index);
            return;
        }

        if modifiers.ctrl || modifiers.meta {
            let prev_lead = self.selection_lead;
            if self.selected_keys.contains(&clicked) {
                self.selected_keys.remove(&clicked);
                if self.selected_keys.is_empty() {
                    self.selection_anchor = None;
                    self.selection_lead = None;
                    self.selection_lead_index = None;
                } else if prev_lead == Some(clicked) {
                    let fallback = self.selected_keys.iter().next().copied();
                    self.selection_lead = fallback;
                    self.selection_lead_index = fallback.and_then(|k| self.data.index_of_key(k));
                }
            } else {
                self.selected_keys.insert(clicked);
                self.selection_lead = Some(clicked);
                self.selection_lead_index = Some(index);
            }
            self.selection_anchor = Some(clicked);
            return;
        }

        self.selected_keys.clear();
        self.selected_keys.insert(clicked);
        self.selection_anchor = Some(clicked);
        self.selection_lead = Some(clicked);
        self.selection_lead_index = Some(index);
    }

    fn max_offset(&self) -> Px {
        Px((self.last_content_height.0 - self.last_viewport_height.0).max(0.0))
    }

    fn clamp_offset(&mut self) {
        let max = self.max_offset();
        self.offset_y = Px(self.offset_y.0.clamp(0.0, max.0));
    }

    pub fn content_bounds(&self) -> Rect {
        let scrollbar_w = self.scrollbar_width;
        if self.last_content_height.0 > self.last_viewport_height.0 {
            Rect::new(
                self.last_bounds.origin,
                Size::new(
                    Px((self.last_bounds.size.width.0 - scrollbar_w.0).max(0.0)),
                    self.last_bounds.size.height,
                ),
            )
        } else {
            self.last_bounds
        }
    }

    fn min_row_height(&self) -> Px {
        match self.row_height {
            VirtualListRowHeight::Fixed(h) => h,
            VirtualListRowHeight::Measured { min } => min,
        }
    }

    fn row_height_at(&self, index: usize) -> Px {
        self.heights_by_index
            .get(index)
            .copied()
            .unwrap_or_else(|| self.min_row_height())
    }

    fn row_top_offset(&self, index: usize) -> Px {
        self.heights_tree.offset_of_index(index)
    }

    fn ensure_heights(&mut self, content_width: Px, scale_factor: f32, theme_revision: u64) {
        let len = self.data.len();
        if len == 0 {
            self.heights_by_index.clear();
            self.heights_tree.rebuild_from_px(&[]);
            self.last_content_height = Px(0.0);
            self.heights_dirty = false;
            self.last_height_width = content_width;
            self.last_height_scale_factor = Some(scale_factor);
            self.last_height_theme_revision = Some(theme_revision);
            return;
        }

        if matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            let width_changed = self.last_height_width != content_width;
            let scale_changed = self.last_height_scale_factor != Some(scale_factor);
            let theme_changed = self.last_height_theme_revision != Some(theme_revision);
            if width_changed || scale_changed || theme_changed {
                self.measured_heights_by_key.clear();
                self.heights_dirty = true;
            }
        }

        if !self.heights_dirty && self.heights_by_index.len() == len {
            self.last_content_height = self.heights_tree.total();
            self.last_height_width = content_width;
            self.last_height_scale_factor = Some(scale_factor);
            self.last_height_theme_revision = Some(theme_revision);
            return;
        }

        self.heights_by_index.clear();
        self.heights_by_index.reserve(len);

        match self.row_height {
            VirtualListRowHeight::Fixed(h) => {
                self.heights_by_index.resize(len, h);
            }
            VirtualListRowHeight::Measured { min } => {
                for i in 0..len {
                    let key = self.data.key_at(i);
                    let h = self
                        .measured_heights_by_key
                        .get(&key)
                        .copied()
                        .unwrap_or(min);
                    self.heights_by_index.push(h);
                }
            }
        }

        self.heights_tree.rebuild_from_px(&self.heights_by_index);
        self.last_content_height = self.heights_tree.total();
        self.clamp_offset();

        self.heights_dirty = false;
        self.last_height_width = content_width;
        self.last_height_scale_factor = Some(scale_factor);
        self.last_height_theme_revision = Some(theme_revision);
    }

    fn row_index_from_y(&self, local_y: Px) -> Option<usize> {
        if self.data.len() == 0 || self.min_row_height().0 <= 0.0 {
            return None;
        }

        match self.row_height {
            VirtualListRowHeight::Fixed(h) => {
                let y = (local_y.0 + self.offset_y.0).max(0.0);
                let idx = (y / h.0).floor() as isize;
                if idx < 0 {
                    return None;
                }
                let idx = idx as usize;
                if idx >= self.data.len() {
                    return None;
                }
                Some(idx)
            }
            VirtualListRowHeight::Measured { .. } => {
                let y = (local_y.0 + self.offset_y.0).max(0.0);
                let idx = self.heights_tree.lower_bound(y);
                (idx < self.data.len()).then_some(idx)
            }
        }
    }

    pub fn row_index_at(&self, position: fret_core::Point) -> Option<usize> {
        let content = self.content_bounds();
        if !content.contains(position) {
            return None;
        }
        let local_y = Px(position.y.0 - content.origin.y.0);
        self.row_index_from_y(local_y)
    }

    pub fn row_rect(&self, index: usize) -> Option<Rect> {
        if index >= self.data.len() {
            return None;
        }
        let content = self.content_bounds();
        let y = content.origin.y.0 + self.row_top_offset(index).0 - self.offset_y.0;
        let h = self.row_height_at(index);
        Some(Rect::new(
            fret_core::Point::new(content.origin.x, Px(y)),
            Size::new(content.size.width, h),
        ))
    }

    pub fn ensure_visible(&mut self, index: usize) {
        if self.min_row_height().0 <= 0.0 || self.last_viewport_height.0 <= 0.0 {
            return;
        }
        let row_top = self.row_top_offset(index).0;
        let row_bottom = row_top + self.row_height_at(index).0;
        let viewport_top = self.offset_y.0;
        let viewport_bottom = self.offset_y.0 + self.last_viewport_height.0;

        if row_top < viewport_top {
            self.offset_y = Px(row_top);
        } else if row_bottom > viewport_bottom {
            self.offset_y = Px(row_bottom - self.last_viewport_height.0);
        }
        self.clamp_offset();
    }

    fn scrollbar_geometry(&self) -> Option<(Rect, Rect)> {
        let viewport_h = self.last_viewport_height;
        if viewport_h.0 <= 0.0 {
            return None;
        }

        let content_h = self.last_content_height;
        if content_h.0 <= viewport_h.0 {
            return None;
        }

        let w = self.scrollbar_width;
        let track = Rect::new(
            fret_core::Point::new(
                Px(self.last_bounds.origin.x.0 + self.last_bounds.size.width.0 - w.0),
                self.last_bounds.origin.y,
            ),
            Size::new(w, self.last_bounds.size.height),
        );

        let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
        let min_thumb = 24.0;
        let thumb_h = Px((viewport_h.0 * ratio).max(min_thumb).min(viewport_h.0));

        let max_offset = self.max_offset().0;
        let t = if max_offset <= 0.0 {
            0.0
        } else {
            (self.offset_y.0 / max_offset).clamp(0.0, 1.0)
        };
        let travel = (viewport_h.0 - thumb_h.0).max(0.0);
        let thumb_y = Px(track.origin.y.0 + travel * t);

        let thumb = Rect::new(
            fret_core::Point::new(track.origin.x, thumb_y),
            Size::new(w, thumb_h),
        );

        Some((track, thumb))
    }

    fn set_offset_from_thumb_y(&mut self, thumb_top_y: Px) {
        let Some((track, thumb)) = self.scrollbar_geometry() else {
            return;
        };

        let viewport_h = self.last_viewport_height.0;
        let travel = (viewport_h - thumb.size.height.0).max(0.0);
        if travel <= 0.0 {
            self.offset_y = Px(0.0);
            return;
        }

        let t = ((thumb_top_y.0 - track.origin.y.0) / travel).clamp(0.0, 1.0);
        let max = self.max_offset().0;
        self.offset_y = Px(max * t);
    }

    fn release_prepared_row(text: &mut dyn fret_core::TextService, row: PreparedRow<D::Key>) {
        if let Some(blob) = row.leading_blob {
            text.release(blob);
        }
        text.release(row.primary_blob);
        if let Some(blob) = row.secondary_blob {
            text.release(blob);
        }
        if let Some(blob) = row.trailing_blob {
            text.release(blob);
        }
    }

    fn update_measured_height(&mut self, index: usize, key: D::Key, height: Px) {
        self.measured_heights_by_key.insert(key, height);
        if index < self.heights_by_index.len() {
            let prev = self.heights_by_index[index];
            if prev != height {
                self.heights_by_index[index] = height;
                self.heights_tree.add(index, height.0 - prev.0);
                self.last_content_height = self.heights_tree.total();
            }
        }
    }

    fn prepare_row(
        &mut self,
        text: &mut dyn fret_core::TextService,
        scale_factor: f32,
        width: Px,
        index: usize,
    ) {
        let key = self.data.key_at(index);
        let VirtualListRow {
            text: row_text,
            secondary_text,
            trailing_text,
            leading_text,
            indent_x,
            enabled,
            selectable,
            kind,
        } = self.data.row_at(index);

        let indent_x_f = indent_x.0;
        let row_inner_w = Px((width.0 - self.style.padding_x.0 * 2.0 - indent_x_f).max(0.0));

        let mut leading_blob = None;
        let mut leading_metrics = None;
        let mut secondary_blob = None;
        let mut secondary_metrics = None;
        let mut trailing_blob = None;
        let mut trailing_metrics = None;

        let (primary_blob, primary_metrics, kind, enabled, selectable, height) = match kind {
            VirtualListRowKind::Separator => {
                let constraints = TextConstraints {
                    max_width: Some(Px(0.0)),
                    wrap: TextWrap::None,
                    scale_factor,
                };
                let (primary_blob, primary_metrics) =
                    text.prepare("", self.style.text_style, constraints);
                let height = match self.row_height {
                    VirtualListRowHeight::Fixed(h) => h,
                    VirtualListRowHeight::Measured { min } => {
                        let h = Px((self.style.padding_y.0 * 2.0 + 1.0).max(min.0));
                        self.update_measured_height(index, key, h);
                        h
                    }
                };
                (primary_blob, primary_metrics, kind, false, false, height)
            }
            VirtualListRowKind::Header => {
                let constraints = TextConstraints {
                    max_width: Some(row_inner_w),
                    wrap: TextWrap::None,
                    scale_factor,
                };
                let (primary_blob, primary_metrics) =
                    text.prepare(row_text.as_ref(), self.style.header_text_style, constraints);
                drop(row_text);

                let height = match self.row_height {
                    VirtualListRowHeight::Fixed(h) => h,
                    VirtualListRowHeight::Measured { min } => {
                        let measured =
                            Px(primary_metrics.size.height.0 + self.style.padding_y.0 * 2.0);
                        let h = Px(measured.0.max(min.0));
                        self.update_measured_height(index, key, h);
                        h
                    }
                };
                (primary_blob, primary_metrics, kind, false, false, height)
            }
            VirtualListRowKind::Item => {
                if let Some(s) = trailing_text.as_ref() {
                    let constraints = TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        scale_factor,
                    };
                    let (blob, metrics) =
                        text.prepare(s.as_ref(), self.style.trailing_text_style, constraints);
                    trailing_blob = Some(blob);
                    trailing_metrics = Some(metrics);
                }

                if let Some(s) = leading_text.as_ref() {
                    let constraints = TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        scale_factor,
                    };
                    let (blob, metrics) =
                        text.prepare(s.as_ref(), self.style.text_style, constraints);
                    leading_blob = Some(blob);
                    leading_metrics = Some(metrics);
                }

                let mut max_width_f = row_inner_w.0;
                if let Some(metrics) = leading_metrics.as_ref() {
                    max_width_f -= metrics.size.width.0 + self.style.trailing_gap_x.0;
                }
                if let Some(metrics) = trailing_metrics.as_ref() {
                    max_width_f -= metrics.size.width.0 + self.style.trailing_gap_x.0;
                }
                let max_width = Px(max_width_f.max(0.0));

                let constraints = TextConstraints {
                    max_width: Some(max_width),
                    wrap: self.style.wrap,
                    scale_factor,
                };
                let (primary_blob, primary_metrics) =
                    text.prepare(row_text.as_ref(), self.style.text_style, constraints);
                drop(row_text);

                if let Some(s) = secondary_text.as_ref() {
                    let constraints = TextConstraints {
                        max_width: Some(max_width),
                        wrap: self.style.wrap,
                        scale_factor,
                    };
                    let (blob, metrics) =
                        text.prepare(s.as_ref(), self.style.secondary_text_style, constraints);
                    secondary_blob = Some(blob);
                    secondary_metrics = Some(metrics);
                }

                let first_line_h = {
                    let mut h = primary_metrics.size.height;
                    if let Some(metrics) = leading_metrics.as_ref() {
                        h = Px(h.0.max(metrics.size.height.0));
                    }
                    if let Some(metrics) = trailing_metrics.as_ref() {
                        h = Px(h.0.max(metrics.size.height.0));
                    }
                    h
                };
                let text_block_h = if let Some(metrics) = secondary_metrics.as_ref() {
                    Px(first_line_h.0 + self.style.row_gap_y.0 + metrics.size.height.0)
                } else {
                    first_line_h
                };

                let height = match self.row_height {
                    VirtualListRowHeight::Fixed(h) => h,
                    VirtualListRowHeight::Measured { min } => {
                        let measured = Px(text_block_h.0 + self.style.padding_y.0 * 2.0);
                        let h = Px(measured.0.max(min.0));
                        self.update_measured_height(index, key, h);
                        h
                    }
                };

                (
                    primary_blob,
                    primary_metrics,
                    kind,
                    enabled,
                    selectable,
                    height,
                )
            }
        };

        self.prepared.push(PreparedRow {
            index,
            key,
            indent_x,
            kind,
            enabled,
            selectable,
            leading_blob,
            leading_metrics,
            primary_blob,
            primary_metrics,
            secondary_blob,
            secondary_metrics,
            trailing_blob,
            trailing_metrics,
            height,
        });
    }

    fn compute_visible_range(&self) -> VisibleRange {
        if self.data.len() == 0
            || self.min_row_height().0 <= 0.0
            || self.last_viewport_height.0 <= 0.0
        {
            return VisibleRange { start: 0, end: 0 };
        }

        let overscan = 2usize;
        match self.row_height {
            VirtualListRowHeight::Fixed(h) => {
                let start = (self.offset_y.0 / h.0).floor().max(0.0) as usize;
                let viewport_rows = (self.last_viewport_height.0 / h.0).ceil() as usize;
                let start = start.saturating_sub(overscan);
                let end = (start + viewport_rows + overscan * 2).min(self.data.len());
                VisibleRange { start, end }
            }
            VirtualListRowHeight::Measured { .. } => {
                let top = self.offset_y.0.max(0.0);
                let bottom = (self.offset_y.0 + self.last_viewport_height.0).max(top);
                let start = self.heights_tree.lower_bound(top);
                let end = self.heights_tree.lower_bound(bottom) + 1;
                let start = start.saturating_sub(overscan);
                let end = (end + overscan).min(self.data.len());
                VisibleRange { start, end }
            }
        }
    }

    fn rebuild_prepared_rows(
        &mut self,
        text: &mut dyn fret_core::TextService,
        scale_factor: f32,
        width: Px,
    ) {
        let anchor = self.capture_scroll_anchor();
        let anchor_top = anchor.map(|(index, _)| self.row_top_offset(index));

        for row in self.prepared.drain(..) {
            Self::release_prepared_row(text, row);
        }

        let visible = self.compute_visible_range();
        self.last_visible = visible;
        self.last_prepared_width = width;

        if visible.start >= visible.end {
            return;
        }

        for i in visible.start..visible.end {
            self.prepare_row(text, scale_factor, width, i);
        }
        self.prepared.sort_by_key(|r| r.index);

        if let Some(anchor) = anchor {
            self.restore_scroll_anchor(anchor, anchor_top);
            self.clamp_offset();
        }
    }

    fn ensure_prepared(
        &mut self,
        text: &mut dyn fret_core::TextService,
        scale_factor: f32,
        width: Px,
    ) {
        let anchor = self.capture_scroll_anchor();
        let anchor_top = anchor.map(|(index, _)| self.row_top_offset(index));

        if self.prepared_dirty {
            self.prepared_dirty = false;
            self.rebuild_prepared_rows(text, scale_factor, width);
            return;
        }

        let visible = self.compute_visible_range();
        if visible == self.last_visible && width == self.last_prepared_width {
            return;
        }

        if width != self.last_prepared_width || visible.start >= visible.end {
            self.rebuild_prepared_rows(text, scale_factor, width);
            return;
        }

        let old = self.last_visible;
        let overlap_start = old.start.max(visible.start);
        let overlap_end = old.end.min(visible.end);
        if overlap_start >= overlap_end {
            self.rebuild_prepared_rows(text, scale_factor, width);
            return;
        }

        let mut kept = Vec::with_capacity(self.prepared.len());
        for row in self.prepared.drain(..) {
            if row.index >= visible.start && row.index < visible.end {
                kept.push(row);
            } else {
                Self::release_prepared_row(text, row);
            }
        }
        self.prepared = kept;

        for i in visible.start..visible.end {
            if self.prepared.iter().any(|r| r.index == i) {
                continue;
            }
            self.prepare_row(text, scale_factor, width, i);
        }
        self.prepared.sort_by_key(|r| r.index);

        self.last_visible = visible;
        self.last_prepared_width = width;

        if let Some(anchor) = anchor {
            self.restore_scroll_anchor(anchor, anchor_top);
            self.clamp_offset();
        }
    }

    fn update_hover(&mut self, content: Rect, position: fret_core::Point) -> bool {
        if !content.contains(position) {
            if self.hovered.take().is_some() {
                return true;
            }
            return false;
        }
        let local_y = Px(position.y.0 - content.origin.y.0);
        let next = self.row_index_from_y(local_y);
        if next != self.hovered {
            self.hovered = next;
            return true;
        }
        false
    }

    fn capture_scroll_anchor(&self) -> Option<(usize, Px)> {
        if !matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            return None;
        }
        if self.data.len() == 0 || self.heights_by_index.is_empty() {
            return None;
        }

        let y = self.offset_y.0.max(0.0);
        let mut index = self.heights_tree.lower_bound(y);
        if index >= self.data.len() {
            index = self.data.len().saturating_sub(1);
        }
        let top = self.row_top_offset(index).0;
        let in_row = Px((y - top).max(0.0));
        Some((index, in_row))
    }

    fn restore_scroll_anchor(&mut self, anchor: (usize, Px), old_top: Option<Px>) {
        if !matches!(self.row_height, VirtualListRowHeight::Measured { .. }) {
            return;
        }

        let (index, in_row) = anchor;
        if index >= self.data.len() || self.heights_by_index.is_empty() {
            return;
        }

        let new_top = self.row_top_offset(index);
        if let Some(old_top) = old_top
            && old_top == new_top
        {
            return;
        }

        let row_h = self.row_height_at(index).0.max(0.0);
        let mut clamped_in_row = in_row.0.max(0.0);
        if row_h > 0.0 {
            clamped_in_row = clamped_in_row.min(row_h);
        } else {
            clamped_in_row = 0.0;
        }

        self.offset_y = Px(new_top.0 + clamped_in_row);
    }

    fn handle_keyboard_nav(&mut self, key: KeyCode, modifiers: Modifiers) -> bool {
        if modifiers.ctrl || modifiers.meta || modifiers.alt {
            return false;
        }

        if self.data.len() == 0 {
            return false;
        }

        let current = self
            .lead_index()
            .unwrap_or(0)
            .min(self.data.len().saturating_sub(1));
        let base_row_h = self.min_row_height();
        let viewport_rows = if base_row_h.0 <= 0.0 {
            1
        } else {
            (self.last_viewport_height.0 / base_row_h.0)
                .floor()
                .max(1.0) as usize
        };

        let len = self.data.len();
        let (mut next, direction) = match key {
            KeyCode::ArrowUp => (current.saturating_sub(1), -1),
            KeyCode::ArrowDown => ((current + 1).min(len.saturating_sub(1)), 1),
            KeyCode::Home => (0, 1),
            KeyCode::End => (len.saturating_sub(1), -1),
            KeyCode::PageUp => (current.saturating_sub(viewport_rows), -1),
            KeyCode::PageDown => ((current + viewport_rows).min(len.saturating_sub(1)), 1),
            _ => return false,
        };

        if !self.is_index_selectable(next) {
            loop {
                if direction < 0 {
                    if next == 0 {
                        return false;
                    }
                    next = next.saturating_sub(1);
                } else {
                    next = next.saturating_add(1);
                    if next >= len {
                        return false;
                    }
                }
                if self.is_index_selectable(next) {
                    break;
                }
            }
        }

        if modifiers.shift {
            if self.selection_anchor.is_none() {
                self.selection_anchor = self
                    .selection_lead
                    .or_else(|| Some(self.data.key_at(current)));
            }
            let anchor_key = self.selection_anchor.or(self.selection_lead);
            let anchor_index = anchor_key
                .and_then(|k| self.data.index_of_key(k))
                .unwrap_or(current);
            self.select_range(anchor_index, next, false);
            self.set_lead_index(next);
        } else {
            self.selected_keys.clear();
            self.selected_keys.insert(self.data.key_at(next));
            self.selection_anchor = Some(self.data.key_at(next));
            self.set_lead_index(next);
        }
        self.ensure_visible(next);
        true
    }
}

impl<H: UiHost, D: VirtualListDataSource> Widget<H> for VirtualList<D> {
    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::List);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        // Events can arrive before the first layout/paint pass; always keep bounds up-to-date so
        // hover hit-testing works from the very first pointer move.
        self.last_bounds = cx.bounds;
        self.last_viewport_height = cx.bounds.size.height;
        if self.heights_dirty {
            let theme_rev = cx.theme().revision();
            let scale_factor = self.last_height_scale_factor.unwrap_or(1.0);
            let mut content_width = cx.bounds.size.width;
            if self.last_content_height.0 > self.last_viewport_height.0 {
                content_width = Px((content_width.0 - self.scrollbar_width.0).max(0.0));
            }
            self.ensure_heights(content_width, scale_factor, theme_rev);
        }
        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Wheel {
                    position, delta, ..
                } => {
                    self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                    self.clamp_offset();
                    let content = self.content_bounds();
                    self.update_hover(content, *position);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Down {
                    position,
                    button,
                    modifiers,
                } => {
                    if *button == MouseButton::Left {
                        if let Some((track, thumb)) = self.scrollbar_geometry()
                            && track.contains(*position)
                        {
                            if thumb.contains(*position) {
                                self.dragging_thumb = true;
                                self.drag_pointer_start_y = position.y;
                                self.drag_offset_start_y = self.offset_y;
                                cx.capture_pointer(cx.node);
                            } else {
                                let centered = Px(position.y.0 - thumb.size.height.0 * 0.5);
                                self.set_offset_from_thumb_y(centered);
                                self.clamp_offset();
                            }

                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }
                    } else if *button != MouseButton::Right {
                        return;
                    }

                    let content = self.content_bounds();
                    if !content.contains(*position) {
                        return;
                    }

                    cx.request_focus(cx.node);
                    let local_y = Px(position.y.0 - content.origin.y.0);
                    if let Some(idx) = self.row_index_from_y(local_y) {
                        if *button == MouseButton::Right && !self.is_index_selectable(idx) {
                            return;
                        }
                        let key = self.data.key_at(idx);
                        if *button == MouseButton::Left {
                            self.apply_click_selection(idx, *modifiers);
                            self.ensure_visible(idx);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        } else {
                            self.set_selected_key(Some(key));
                            self.context_menu_target = Some(key);
                            self.ensure_visible(idx);

                            if cx.window.is_some() {
                                let inv_ctx = InputContext {
                                    platform: cx.input_ctx.platform,
                                    caps: cx.input_ctx.caps.clone(),
                                    ui_has_modal: cx.input_ctx.ui_has_modal,
                                    focus_is_text_input: false,
                                };

                                let menu = Menu {
                                    title: Arc::from("List"),
                                    items: vec![
                                        MenuItem::Command {
                                            command: CommandId::from("virtual_list.copy_label"),
                                            when: None,
                                        },
                                        MenuItem::Separator,
                                        MenuItem::Submenu {
                                            title: Arc::from("Selection"),
                                            when: None,
                                            items: vec![MenuItem::Command {
                                                command: CommandId::from(
                                                    "virtual_list.clear_selection",
                                                ),
                                                when: None,
                                            }],
                                        },
                                    ],
                                };

                                cx.open_context_menu_request(ContextMenuRequest {
                                    position: *position,
                                    menu,
                                    input_ctx: inv_ctx,
                                    menu_bar: None,
                                });
                            }
                            cx.invalidate_self(Invalidation::Paint);
                        }
                    }
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Move { position, .. } => {
                    if self.dragging_thumb && cx.captured == Some(cx.node) {
                        let dy = position.y.0 - self.drag_pointer_start_y.0;
                        let Some((_, thumb)) = self.scrollbar_geometry() else {
                            return;
                        };

                        let max_offset = self.max_offset().0;
                        let travel = (self.last_viewport_height.0 - thumb.size.height.0).max(0.0);
                        if travel <= 0.0 || max_offset <= 0.0 {
                            return;
                        }

                        let offset_delta = dy / travel * max_offset;
                        self.offset_y = Px(self.drag_offset_start_y.0 + offset_delta);
                        self.clamp_offset();
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    let content = self.content_bounds();
                    if self.update_hover(content, *position) {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::PointerEvent::Up { button, .. } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if self.dragging_thumb && cx.captured == Some(cx.node) {
                        self.dragging_thumb = false;
                        cx.release_pointer_capture();
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            },
            Event::KeyDown { key, modifiers, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.handle_keyboard_nav(*key, *modifiers) {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn command(&mut self, cx: &mut crate::widget::CommandCx<'_, H>, command: &CommandId) -> bool {
        match command.as_str() {
            "virtual_list.copy_label" => {
                let Some(key) = self.context_menu_target.or(self.selection_lead) else {
                    return false;
                };
                let Some(index) = self.data.index_of_key(key) else {
                    return false;
                };
                let text = self.data.row_at(index).text.into_owned();
                cx.app.push_effect(Effect::ClipboardSetText { text });
                cx.stop_propagation();
                true
            }
            "virtual_list.clear_selection" => {
                self.clear_selection();
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        self.last_viewport_height = cx.available.height;
        let theme_rev = cx.theme().revision();

        let mut content_width = cx.available.width;
        self.ensure_heights(content_width, cx.scale_factor, theme_rev);
        if self.last_content_height.0 > self.last_viewport_height.0 {
            let w = Px((cx.available.width.0 - self.scrollbar_width.0).max(0.0));
            if w != content_width {
                content_width = w;
                self.ensure_heights(content_width, cx.scale_factor, theme_rev);
            }
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        self.last_viewport_height = cx.bounds.size.height;
        let theme_rev = cx.theme().revision();
        let mut content_width = cx.bounds.size.width;
        self.ensure_heights(content_width, cx.scale_factor, theme_rev);
        if self.last_content_height.0 > self.last_viewport_height.0 {
            let w = Px((cx.bounds.size.width.0 - self.scrollbar_width.0).max(0.0));
            if w != content_width {
                content_width = w;
                self.ensure_heights(content_width, cx.scale_factor, theme_rev);
            }
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        let content = self.content_bounds();
        self.ensure_prepared(cx.text, cx.scale_factor, content.size.width);
        cx.scene.push(SceneOp::PushClipRect { rect: content });

        for row in &self.prepared {
            let y = content.origin.y.0 + self.row_top_offset(row.index).0 - self.offset_y.0;
            let row_rect = Rect::new(
                fret_core::Point::new(content.origin.x, Px(y)),
                Size::new(content.size.width, row.height),
            );

            let is_selected = self.selected_keys.contains(&row.key);
            let is_hovered = self.hovered == Some(row.index);

            if row_rect.size.width.0 <= 0.0 || row_rect.size.height.0 <= 0.0 {
                continue;
            }

            // Clip each row to avoid any text/background bleeding into adjacent rows.
            cx.scene.push(SceneOp::PushClipRect { rect: row_rect });
            match row.kind {
                VirtualListRowKind::Separator => {
                    let inset = self.style.separator_inset_x;
                    let left = Px(row_rect.origin.x.0 + inset.0 + row.indent_x.0);
                    let right = Px(row_rect.origin.x.0 + row_rect.size.width.0 - inset.0);
                    let w = Px((right.0 - left.0).max(0.0));
                    if w.0 > 0.0 {
                        let line_y = Px(row_rect.origin.y.0 + row.height.0 * 0.5);
                        let rect =
                            Rect::new(fret_core::Point::new(left, line_y), Size::new(w, Px(1.0)));
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(0),
                            rect,
                            background: self.style.separator_color,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                }
                VirtualListRowKind::Header => {
                    let text_h = row.primary_metrics.size.height;
                    let max_pad_y = self.style.padding_y.0.max(0.0);
                    let available_pad_y = Px(((row.height.0 - text_h.0) * 0.5).max(0.0));
                    let pad_y = Px(max_pad_y.min(available_pad_y.0));
                    let block_h = Px(text_h.0 + pad_y.0 * 2.0);
                    let block_top =
                        Px(row_rect.origin.y.0 + ((row.height.0 - block_h.0) * 0.5).max(0.0));
                    let baseline_y = Px(block_top.0 + pad_y.0 + row.primary_metrics.baseline.0);
                    let x = Px(row_rect.origin.x.0 + self.style.padding_x.0 + row.indent_x.0);
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(0),
                        origin: fret_core::Point::new(x, baseline_y),
                        text: row.primary_blob,
                        color: self.style.header_text_color,
                    });
                }
                VirtualListRowKind::Item => {
                    let can_highlight = row.enabled && row.selectable;
                    if can_highlight && (is_selected || is_hovered) {
                        let bg = if is_selected {
                            self.style.row_selected
                        } else {
                            self.style.row_hover
                        };
                        let inset_y = self.style.row_highlight_inset_y.0.max(0.0);
                        let rect = if inset_y > 0.0 {
                            Rect::new(
                                fret_core::Point::new(
                                    row_rect.origin.x,
                                    Px(row_rect.origin.y.0 + inset_y),
                                ),
                                Size::new(
                                    row_rect.size.width,
                                    Px((row_rect.size.height.0 - inset_y * 2.0).max(0.0)),
                                ),
                            )
                        } else {
                            row_rect
                        };
                        if rect.size.height.0 > 0.0 && rect.size.width.0 > 0.0 {
                            cx.scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: bg,
                                border: Edges::all(Px(0.0)),
                                border_color: Color::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }
                    }

                    let mut left_x =
                        Px(row_rect.origin.x.0 + self.style.padding_x.0 + row.indent_x.0);
                    let right_x =
                        Px(row_rect.origin.x.0 + row_rect.size.width.0 - self.style.padding_x.0);

                    let first_line_h = {
                        let mut h = row.primary_metrics.size.height;
                        if let Some(m) = row.leading_metrics.as_ref() {
                            h = Px(h.0.max(m.size.height.0));
                        }
                        if let Some(m) = row.trailing_metrics.as_ref() {
                            h = Px(h.0.max(m.size.height.0));
                        }
                        h
                    };
                    let content_h = if let Some(m) = row.secondary_metrics.as_ref() {
                        Px(first_line_h.0 + self.style.row_gap_y.0 + m.size.height.0)
                    } else {
                        first_line_h
                    };
                    let max_pad_y = self.style.padding_y.0.max(0.0);
                    let available_pad_y = Px(((row.height.0 - content_h.0) * 0.5).max(0.0));
                    let pad_y = Px(max_pad_y.min(available_pad_y.0));
                    let block_h = Px(content_h.0 + pad_y.0 * 2.0);
                    let block_top =
                        Px(row_rect.origin.y.0 + ((row.height.0 - block_h.0) * 0.5).max(0.0));

                    let baseline_y = Px(block_top.0 + pad_y.0 + row.primary_metrics.baseline.0);

                    let trailing_x = row
                        .trailing_metrics
                        .as_ref()
                        .map(|m| Px(right_x.0 - m.size.width.0));

                    if let Some(metrics) = row.leading_metrics.as_ref()
                        && row.leading_blob.is_some()
                    {
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(0),
                            origin: fret_core::Point::new(left_x, baseline_y),
                            text: row.leading_blob.expect("leading_blob checked"),
                            color: if row.enabled {
                                self.style.text_color
                            } else {
                                self.style.secondary_text_color
                            },
                        });
                        left_x = Px(left_x.0 + metrics.size.width.0 + self.style.trailing_gap_x.0);
                    }

                    if let Some(((blob, _metrics), x)) = row
                        .trailing_blob
                        .zip(row.trailing_metrics.as_ref())
                        .zip(trailing_x)
                    {
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(0),
                            origin: fret_core::Point::new(x, baseline_y),
                            text: blob,
                            color: if row.enabled {
                                self.style.trailing_text_color
                            } else {
                                self.style.secondary_text_color
                            },
                        });
                    }

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(0),
                        origin: fret_core::Point::new(left_x, baseline_y),
                        text: row.primary_blob,
                        color: if row.enabled {
                            self.style.text_color
                        } else {
                            self.style.secondary_text_color
                        },
                    });

                    if let Some((blob, metrics)) =
                        row.secondary_blob.zip(row.secondary_metrics.as_ref())
                    {
                        let second_baseline_y = Px(block_top.0
                            + pad_y.0
                            + first_line_h.0
                            + self.style.row_gap_y.0
                            + metrics.baseline.0);
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(0),
                            origin: fret_core::Point::new(left_x, second_baseline_y),
                            text: blob,
                            color: self.style.secondary_text_color,
                        });
                    }
                }
            }
            cx.scene.push(SceneOp::PopClip);
        }

        cx.scene.push(SceneOp::PopClip);

        if let Some((track, thumb)) = self.scrollbar_geometry() {
            let (track_bg, thumb_bg, thumb_hover_bg, radius) = {
                let theme = cx.theme();
                (
                    theme.colors.scrollbar_track,
                    theme.colors.scrollbar_thumb,
                    theme.colors.scrollbar_thumb_hover,
                    theme.metrics.radius_sm,
                )
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: track_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });

            let thumb_bg = if self.dragging_thumb {
                thumb_hover_bg
            } else {
                thumb_bg
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: thumb_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{AppWindowId, NodeId, Point, Rect, Scene, Size, TextBlobId, TextMetrics};
    use fret_runtime::InputContext;

    #[derive(Default)]
    struct FakeTextService;

    impl fret_core::TextService for FakeTextService {
        fn prepare(
            &mut self,
            text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            let h = Px((text.len().max(1)) as f32);
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(80.0), h),
                    baseline: Px(h.0 * 0.7),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    #[derive(Debug, Clone)]
    struct TestDataSource {
        rows: Vec<String>,
    }

    impl VirtualListDataSource for TestDataSource {
        type Key = usize;

        fn len(&self) -> usize {
            self.rows.len()
        }

        fn key_at(&self, index: usize) -> Self::Key {
            index
        }

        fn row_at(&self, index: usize) -> VirtualListRow<'_> {
            VirtualListRow::new(self.rows[index].as_str())
        }
    }

    #[test]
    fn measured_row_height_affects_row_rect_and_hit_testing() {
        let mut app = TestHost::new();
        let mut text = FakeTextService::default();

        let data = TestDataSource {
            rows: vec![
                "aaa".to_string(),      // height 3
                "bbbbbbbb".to_string(), // height 8
                "ccccc".to_string(),    // height 5
            ],
        };

        let style = VirtualListStyle {
            padding_y: Px(0.0),
            ..VirtualListStyle::default()
        };

        let mut list =
            VirtualList::new(data).with_row_height(VirtualListRowHeight::Measured { min: Px(4.0) });
        list = list.with_style(style);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(200.0)),
        );

        let mut observe_model = |_model, _inv| {};
        let mut layout_child =
            |_child: NodeId, _bounds: Rect| -> Size { panic!("virtual list has no children") };

        let mut layout_cx = LayoutCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            available: bounds.size,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            layout_child: &mut layout_child,
        };
        let _ = list.layout(&mut layout_cx);

        let mut scene = Scene::default();
        let mut paint_child = |_child: NodeId, _bounds: Rect| {};
        let child_bounds = |_child: NodeId| None;
        let mut paint_cx = PaintCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            scene: &mut scene,
            paint_child: &mut paint_child,
            child_bounds: &child_bounds,
        };
        list.paint(&mut paint_cx);

        let r0 = list.row_rect(0).expect("row 0 rect");
        let r1 = list.row_rect(1).expect("row 1 rect");
        let r2 = list.row_rect(2).expect("row 2 rect");

        assert_eq!(r0.size.height, Px(4.0)); // min wins (3 -> 4)
        assert_eq!(r1.size.height, Px(8.0));
        assert_eq!(r2.size.height, Px(5.0));
        assert_eq!(r1.origin.y, Px(4.0));
        assert_eq!(r2.origin.y, Px(12.0));

        let hit = list.row_index_at(Point::new(Px(10.0), Px(6.0)));
        assert_eq!(hit, Some(1));
    }

    #[test]
    fn measured_row_height_updates_preserve_scroll_anchor() {
        let mut app = TestHost::new();
        let mut text = FakeTextService::default();

        let data = TestDataSource {
            rows: vec![
                "aaaaaaaaaa".to_string(), // height 10
                "bbbbbbbbbb".to_string(), // height 10
                "c".to_string(),          // height 1 (min wins -> 4)
            ],
        };

        let style = VirtualListStyle {
            padding_y: Px(0.0),
            ..VirtualListStyle::default()
        };

        let mut list =
            VirtualList::new(data).with_row_height(VirtualListRowHeight::Measured { min: Px(4.0) });
        list = list.with_style(style);

        // Scroll to the third row using the initial min heights (2 * 4px = 8px).
        list.offset_y = Px(8.0);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(4.0)));

        let mut observe_model = |_model, _inv| {};
        let mut layout_child =
            |_child: NodeId, _bounds: Rect| -> Size { panic!("virtual list has no children") };

        let mut layout_cx = LayoutCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            available: bounds.size,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            layout_child: &mut layout_child,
        };
        let _ = list.layout(&mut layout_cx);

        let mut scene = Scene::default();
        let mut paint_child = |_child: NodeId, _bounds: Rect| {};
        let child_bounds = |_child: NodeId| None;
        let mut paint_cx = PaintCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            scene: &mut scene,
            paint_child: &mut paint_child,
            child_bounds: &child_bounds,
        };

        // Overscan prepares and measures rows above the viewport, which changes their heights.
        list.paint(&mut paint_cx);

        // The top of row 2 becomes 10 + 10 = 20 after measurement; we should stay anchored.
        assert_eq!(list.offset_y, Px(20.0));
    }

    #[test]
    fn hover_updates_before_first_layout_pass() {
        let mut app = TestHost::new();
        let mut text = FakeTextService::default();

        let data = TestDataSource {
            rows: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let mut list =
            VirtualList::new(data).with_row_height(VirtualListRowHeight::Fixed(Px(20.0)));

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        let mut cx = EventCx {
            app: &mut app,
            text: &mut text,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            input_ctx: InputContext::default(),
            children: &[],
            focus: None,
            captured: None,
            bounds,
            invalidations: Vec::new(),
            requested_focus: None,
            requested_capture: None,
            requested_cursor: None,
            stop_propagation: false,
        };

        let event = Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
        });
        list.event(&mut cx, &event);

        assert_eq!(list.hovered, Some(0));
    }

    #[test]
    fn wheel_scroll_updates_hover_under_pointer() {
        let mut app = TestHost::new();
        let mut text = FakeTextService::default();

        let data = TestDataSource {
            rows: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let mut list =
            VirtualList::new(data).with_row_height(VirtualListRowHeight::Fixed(Px(20.0)));

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

        // Pre-seed layout-dependent fields so scrolling is meaningful in the test.
        let theme_rev = Theme::global(&app).revision();
        list.last_bounds = bounds;
        list.last_viewport_height = bounds.size.height;
        list.ensure_heights(bounds.size.width, 1.0, theme_rev);
        assert_eq!(list.max_offset(), Px(20.0));

        let mut cx = EventCx {
            app: &mut app,
            text: &mut text,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            input_ctx: InputContext::default(),
            children: &[],
            focus: None,
            captured: None,
            bounds,
            invalidations: Vec::new(),
            requested_focus: None,
            requested_capture: None,
            requested_cursor: None,
            stop_propagation: false,
        };

        // No prior pointer move: wheel scroll should still update hover based on the wheel position.
        let wheel = Event::Pointer(fret_core::PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-20.0)),
            modifiers: Modifiers::default(),
        });
        list.event(&mut cx, &wheel);

        assert_eq!(list.offset_y, Px(20.0));
        assert_eq!(list.hovered, Some(1));
    }

    #[test]
    fn paint_clips_each_row() {
        let mut app = TestHost::new();
        let mut text = FakeTextService::default();

        let data = TestDataSource {
            rows: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let mut list =
            VirtualList::new(data).with_row_height(VirtualListRowHeight::Fixed(Px(20.0)));

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));

        let mut observe_model = |_model, _inv| {};
        let mut layout_child =
            |_child: NodeId, _bounds: Rect| -> Size { panic!("virtual list has no children") };

        let mut layout_cx = LayoutCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            available: bounds.size,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            layout_child: &mut layout_child,
        };
        let _ = list.layout(&mut layout_cx);

        let mut scene = Scene::default();
        let mut paint_child = |_child: NodeId, _bounds: Rect| {};
        let child_bounds = |_child: NodeId| None;
        let mut paint_cx = PaintCx {
            app: &mut app,
            node: NodeId::default(),
            window: Some(AppWindowId::default()),
            focus: None,
            children: &[],
            bounds,
            scale_factor: 1.0,
            text: &mut text,
            observe_model: &mut observe_model,
            scene: &mut scene,
            paint_child: &mut paint_child,
            child_bounds: &child_bounds,
        };
        list.paint(&mut paint_cx);

        let row_pushes = scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
            .count();
        // One content clip + one clip per visible row.
        assert_eq!(row_pushes, 4);
    }
}
