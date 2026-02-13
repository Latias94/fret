//! Message branching surfaces aligned with AI Elements `message.tsx`.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/message.tsx` (`MessageBranch*`).

use std::sync::Arc;

use fret_core::{Color, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, Length, SemanticsDecoration,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space};
use fret_ui_shadcn::button_group::ButtonGroupText;
use fret_ui_shadcn::{Button, ButtonGroup, ButtonSize, ButtonVariant};

pub type OnMessageBranchChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, usize) + 'static>;

#[derive(Debug, Clone, Default)]
struct MessageBranchState {
    current_branch: Option<Model<usize>>,
}

fn ensure_branch_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_branch: usize,
) -> Model<usize> {
    let existing = cx.with_state(MessageBranchState::default, |st| st.current_branch.clone());
    if let Some(model) = existing {
        return model;
    }

    let model = cx.app.models_mut().insert(default_branch);
    cx.with_state(MessageBranchState::default, |st| {
        st.current_branch = Some(model.clone())
    });
    model
}

#[derive(Clone)]
/// A branching container aligned with AI Elements `MessageBranch`.
///
/// This is a convenience wrapper that:
/// - owns (or accepts) the current branch model,
/// - renders `MessageBranchContent` + `MessageBranchSelector` as a single unit.
pub struct MessageBranch {
    branches: Vec<AnyElement>,
    current_branch: Option<Model<usize>>,
    default_branch: usize,
    on_branch_change: Option<OnMessageBranchChange>,
    show_selector: bool,
    test_id_root: Option<Arc<str>>,
    selector_test_id: Option<Arc<str>>,
    prev_test_id: Option<Arc<str>>,
    next_test_id: Option<Arc<str>>,
    page_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBranch")
            .field("branches_len", &self.branches.len())
            .field("current_branch", &"<model>")
            .field("default_branch", &self.default_branch)
            .field("has_on_branch_change", &self.on_branch_change.is_some())
            .field("show_selector", &self.show_selector)
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl MessageBranch {
    pub fn new(branches: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            branches: branches.into_iter().collect(),
            current_branch: None,
            default_branch: 0,
            on_branch_change: None,
            show_selector: true,
            test_id_root: None,
            selector_test_id: None,
            prev_test_id: None,
            next_test_id: None,
            page_test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    /// Provide a controlled model for the current branch (0-based).
    pub fn current_branch(mut self, current_branch: Model<usize>) -> Self {
        self.current_branch = Some(current_branch);
        self
    }

    /// Uncontrolled initial branch index (AI Elements `defaultBranch`).
    pub fn default_branch(mut self, default_branch: usize) -> Self {
        self.default_branch = default_branch;
        self
    }

    /// Invoked after the current branch changes (AI Elements `onBranchChange`).
    pub fn on_branch_change(mut self, on_branch_change: OnMessageBranchChange) -> Self {
        self.on_branch_change = Some(on_branch_change);
        self
    }

    pub fn show_selector(mut self, show: bool) -> Self {
        self.show_selector = show;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn selector_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.selector_test_id = Some(id.into());
        self
    }

    pub fn prev_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prev_test_id = Some(id.into());
        self
    }

    pub fn next_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.next_test_id = Some(id.into());
        self
    }

    pub fn page_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.page_test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let current_branch = self
            .current_branch
            .clone()
            .unwrap_or_else(|| ensure_branch_model(cx, self.default_branch));

        let total = self.branches.len();

        let content = MessageBranchContent::new(self.branches)
            .current_branch(current_branch.clone())
            .into_element(cx);

        let selector = self.show_selector.then(|| {
            MessageBranchSelector::new(total)
                .current_branch(current_branch.clone())
                .on_branch_change_opt(self.on_branch_change)
                .test_id_root_opt(self.selector_test_id)
                .prev_test_id_opt(self.prev_test_id)
                .next_test_id_opt(self.next_test_id)
                .page_test_id_opt(self.page_test_id)
                .into_element(cx)
        });

        let layout = self.layout;
        let test_id_root = self.test_id_root;
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(layout)
                .gap(Space::N2)
                .items_start(),
            move |_cx| {
                let mut out = vec![content];
                if let Some(sel) = selector {
                    out.push(sel);
                }
                out
            },
        );

        if let Some(test_id) = test_id_root {
            return body.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }

        body
    }
}

#[derive(Clone)]
/// Branch content surface aligned with AI Elements `MessageBranchContent`.
///
/// This keeps all branches mounted but uses `InteractivityGate(present=...)` so inactive branches:
/// - do not participate in layout/paint/semantics,
/// - do not intercept input.
pub struct MessageBranchContent {
    branches: Vec<AnyElement>,
    current_branch: Option<Model<usize>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageBranchContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBranchContent")
            .field("branches_len", &self.branches.len())
            .field("current_branch", &"<model>")
            .field("layout", &self.layout)
            .finish()
    }
}

impl MessageBranchContent {
    pub fn new(branches: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            branches: branches.into_iter().collect(),
            current_branch: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn current_branch(mut self, current_branch: Model<usize>) -> Self {
        self.current_branch = Some(current_branch);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let total = self.branches.len();
        let Some(current_branch) = self.current_branch else {
            return cx.interactivity_gate(false, false, |_cx| Vec::new());
        };

        let current = cx
            .get_model_copied(&current_branch, Invalidation::Layout)
            .unwrap_or(0);
        let current = if total == 0 { 0 } else { current % total };

        let layout = self.layout;
        let branches = self.branches;

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(layout)
                .gap(Space::N2)
                .items_start(),
            move |cx| {
                branches
                    .into_iter()
                    .enumerate()
                    .map(|(idx, branch)| {
                        let present = idx == current;

                        cx.keyed(idx as u64, |cx| {
                            let mut gate_layout = LayoutStyle::default();
                            gate_layout.size.width = Length::Fill;
                            gate_layout.size.height = Length::Auto;

                            cx.interactivity_gate_props(
                                InteractivityGateProps {
                                    layout: gate_layout,
                                    present,
                                    interactive: present,
                                },
                                move |_cx| vec![branch],
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            },
        )
    }
}

#[derive(Clone)]
/// Selector control surface aligned with AI Elements `MessageBranchSelector`.
pub struct MessageBranchSelector {
    total_branches: usize,
    current_branch: Option<Model<usize>>,
    on_branch_change: Option<OnMessageBranchChange>,
    test_id_root: Option<Arc<str>>,
    prev_test_id: Option<Arc<str>>,
    next_test_id: Option<Arc<str>>,
    page_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageBranchSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBranchSelector")
            .field("total_branches", &self.total_branches)
            .field("current_branch", &"<model>")
            .field("has_on_branch_change", &self.on_branch_change.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl MessageBranchSelector {
    pub fn new(total_branches: usize) -> Self {
        Self {
            total_branches,
            current_branch: None,
            on_branch_change: None,
            test_id_root: None,
            prev_test_id: None,
            next_test_id: None,
            page_test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn current_branch(mut self, current_branch: Model<usize>) -> Self {
        self.current_branch = Some(current_branch);
        self
    }

    pub fn on_branch_change(mut self, on_branch_change: OnMessageBranchChange) -> Self {
        self.on_branch_change = Some(on_branch_change);
        self
    }

    fn on_branch_change_opt(mut self, on_branch_change: Option<OnMessageBranchChange>) -> Self {
        self.on_branch_change = on_branch_change;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    fn test_id_root_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id_root = id;
        self
    }

    pub fn prev_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prev_test_id = Some(id.into());
        self
    }

    fn prev_test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.prev_test_id = id;
        self
    }

    pub fn next_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.next_test_id = Some(id.into());
        self
    }

    fn next_test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.next_test_id = id;
        self
    }

    pub fn page_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.page_test_id = Some(id.into());
        self
    }

    fn page_test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.page_test_id = id;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(current_branch) = self.current_branch else {
            return cx.interactivity_gate(false, false, |_cx| Vec::new());
        };
        if self.total_branches <= 1 {
            return cx.interactivity_gate(false, false, |_cx| Vec::new());
        }

        let theme = Theme::global(&*cx.app).clone();
        let current = cx
            .get_model_copied(&current_branch, Invalidation::Layout)
            .unwrap_or(0)
            % self.total_branches;

        let muted_fg = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_required("foreground"));

        let prev_target = if current > 0 {
            current - 1
        } else {
            self.total_branches - 1
        };
        let next_target = if current < self.total_branches - 1 {
            current + 1
        } else {
            0
        };

        let on_branch_change = self.on_branch_change;
        let current_branch = current_branch;

        let prev = {
            let model = current_branch.clone();
            let test_id = self.prev_test_id;
            let on_branch_change = on_branch_change.clone();
            let label = Arc::<str>::from("Previous branch");
            let icon = decl_icon::icon(cx, fret_icons::IconId::new("lucide.chevron-left"));
            let mut btn = Button::new(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .children([icon])
                .on_activate(Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&model, |v| *v = prev_target);
                    if let Some(cb) = on_branch_change.clone() {
                        cb(host, action_cx, prev_target);
                    }
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }));
            if let Some(test_id) = test_id {
                btn = btn.test_id(test_id);
            }
            btn.into_element(cx)
        };

        let page = {
            let text = Arc::<str>::from(format!("{} of {}", current + 1, self.total_branches));
            let chrome = ChromeRefinement::default()
                .bg(ColorRef::Color(Color::TRANSPARENT))
                .shadow_none()
                .border_width(MetricRef::Px(Px(0.0)))
                .text_color(ColorRef::Color(muted_fg));

            // Match AI Elements: `ButtonGroupText` with a "transparent" chrome.
            let mut el = ButtonGroupText::new(text)
                .refine_style(chrome)
                .into_element(cx);
            if let Some(test_id) = self.page_test_id {
                el = el.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Text)
                        .test_id(test_id),
                );
            }
            el
        };

        let next = {
            let model = current_branch.clone();
            let test_id = self.next_test_id;
            let on_branch_change = on_branch_change.clone();
            let label = Arc::<str>::from("Next branch");
            let icon = decl_icon::icon(cx, fret_icons::IconId::new("lucide.chevron-right"));
            let mut btn = Button::new(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .children([icon])
                .on_activate(Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&model, |v| *v = next_target);
                    if let Some(cb) = on_branch_change.clone() {
                        cb(host, action_cx, next_target);
                    }
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }));
            if let Some(test_id) = test_id {
                btn = btn.test_id(test_id);
            }
            btn.into_element(cx)
        };

        let group =
            ButtonGroup::new([prev.into(), page.into(), next.into()]).refine_layout(self.layout);

        if let Some(test_id) = self.test_id_root {
            let el = group.into_element(cx);
            return el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }

        group.into_element(cx)
    }
}
