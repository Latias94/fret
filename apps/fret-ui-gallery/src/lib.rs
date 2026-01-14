use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, SemanticsRole, UiServices};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_markdown as markdown;
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::SemanticsProps;
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

const CMD_NAV_SELECT_PREFIX: &str = "ui_gallery.nav.select.";

const PAGE_INTRO: &str = "intro";
const PAGE_LAYOUT: &str = "layout";
const PAGE_BUTTON: &str = "button";
const PAGE_OVERLAY: &str = "overlay";

const CMD_NAV_INTRO: &str = "ui_gallery.nav.select.intro";
const CMD_NAV_LAYOUT: &str = "ui_gallery.nav.select.layout";
const CMD_NAV_BUTTON: &str = "ui_gallery.nav.select.button";
const CMD_NAV_OVERLAY: &str = "ui_gallery.nav.select.overlay";

static NAV_GROUPS: &[NavGroupSpec] = &[
    NavGroupSpec {
        title: "Core",
        items: &[
            NavItemSpec::new(
                PAGE_INTRO,
                "Introduction",
                "Core contracts",
                CMD_NAV_INTRO,
                &["overview", "contracts"],
            ),
            NavItemSpec::new(
                PAGE_LAYOUT,
                "Layout",
                "Layout system",
                CMD_NAV_LAYOUT,
                &["layout", "flex", "stack"],
            ),
        ],
    },
    NavGroupSpec {
        title: "Shadcn",
        items: &[
            NavItemSpec::new(
                PAGE_BUTTON,
                "Button",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON,
                &["button", "variant"],
            ),
            NavItemSpec::new(
                PAGE_OVERLAY,
                "Overlay",
                "Radix-shaped primitives",
                CMD_NAV_OVERLAY,
                &["dialog", "popover"],
            ),
        ],
    },
];

#[derive(Clone, Copy)]
struct NavItemSpec {
    id: &'static str,
    label: &'static str,
    origin: &'static str,
    command: &'static str,
    tags: &'static [&'static str],
}

impl NavItemSpec {
    const fn new(
        id: &'static str,
        label: &'static str,
        origin: &'static str,
        command: &'static str,
        tags: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            label,
            origin,
            command,
            tags,
        }
    }
}

#[derive(Clone, Copy)]
struct NavGroupSpec {
    title: &'static str,
    items: &'static [NavItemSpec],
}

struct UiGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    selected_page: Model<Arc<str>>,
    nav_query: Model<String>,
    content_tab: Model<Option<Arc<str>>>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
}

#[derive(Default)]
struct UiGalleryDriver;

impl UiGalleryDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> UiGalleryWindowState {
        let selected_page = app.models_mut().insert(Arc::<str>::from(PAGE_INTRO));
        let nav_query = app.models_mut().insert(String::new());
        let content_tab = app.models_mut().insert(Some(Arc::<str>::from("preview")));
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        UiGalleryWindowState {
            ui,
            root: None,
            selected_page,
            nav_query,
            content_tab,
            popover_open,
            dialog_open,
        }
    }

    fn handle_nav_command(
        app: &mut App,
        state: &UiGalleryWindowState,
        command: &CommandId,
    ) -> bool {
        let Some(page) = command.as_str().strip_prefix(CMD_NAV_SELECT_PREFIX) else {
            return false;
        };

        let page: Arc<str> = Arc::from(page);
        let _ = app.models_mut().update(&state.selected_page, |v| *v = page);
        true
    }

    fn matches_query(query: &str, item: &NavItemSpec) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return true;
        }

        let q_lower = q.to_ascii_lowercase();
        if item.label.to_ascii_lowercase().contains(&q_lower) {
            return true;
        }
        if item.origin.to_ascii_lowercase().contains(&q_lower) {
            return true;
        }
        item.tags
            .iter()
            .any(|t| t.to_ascii_lowercase().contains(&q_lower))
    }

    fn render_ui(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        bounds: fret_core::Rect,
    ) {
        let selected_page = state.selected_page.clone();
        let nav_query = state.nav_query.clone();
        let content_tab = state.content_tab.clone();
        let popover_open = state.popover_open.clone();
        let dialog_open = state.dialog_open.clone();

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("fret-ui-gallery", |cx| {
                    cx.observe_model(&selected_page, Invalidation::Layout);
                    cx.observe_model(&nav_query, Invalidation::Layout);
                    cx.observe_model(&content_tab, Invalidation::Layout);
                    cx.observe_model(&popover_open, Invalidation::Layout);
                    cx.observe_model(&dialog_open, Invalidation::Layout);

                    let theme = Theme::global(&*cx.app).clone();

                    let selected = cx
                        .app
                        .models()
                        .read(&selected_page, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                    let query = cx
                        .app
                        .models()
                        .read(&nav_query, |v| v.clone())
                        .ok()
                        .unwrap_or_default();

                    let sidebar = sidebar_view(
                        cx,
                        &theme,
                        selected.as_ref(),
                        query.as_str(),
                        nav_query.clone(),
                    );
                    let content = content_view(
                        cx,
                        &theme,
                        selected.as_ref(),
                        content_tab.clone(),
                        popover_open.clone(),
                        dialog_open.clone(),
                    );

                    vec![cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("fret-ui-gallery")),
                            ..Default::default()
                        },
                        |cx| {
                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full().h_full())
                                    .items_stretch()
                                    .gap(Space::N0),
                                |_cx| vec![sidebar, content],
                            )]
                        },
                    )]
                });

        state.root = Some(root);
    }
}

fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
) -> AnyElement {
    let title_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                cx.text("Fret UI Gallery"),
                shadcn::Badge::new("WIP")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        },
    );

    let query_input = shadcn::Input::new(nav_query)
        .a11y_label("Search components")
        .placeholder("Search… (id / tag)")
        .into_element(cx);

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in NAV_GROUPS {
        let mut group_items: Vec<AnyElement> = Vec::new();
        for item in group.items {
            if !UiGalleryDriver::matches_query(query, item) {
                continue;
            }

            let is_selected = selected == item.id;
            let variant = if is_selected {
                shadcn::ButtonVariant::Secondary
            } else {
                shadcn::ButtonVariant::Ghost
            };

            group_items.push(
                shadcn::Button::new(item.label)
                    .variant(variant)
                    .on_click(item.command)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            );
        }

        if group_items.is_empty() {
            continue;
        }

        nav_sections.push(cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(group.title),
            style: None,
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        }));

        nav_sections.push(stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N1),
            |_cx| group_items,
        ));
    }

    let nav_scroll = shadcn::ScrollArea::new(vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| nav_sections,
    )])
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);

    let container = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N4),
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(280.0)))
                .h_full(),
        ),
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| vec![title_row, query_input, nav_scroll],
            )]
        },
    );

    container
}

fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    content_tab: Model<Option<Arc<str>>>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
) -> AnyElement {
    let (title, origin, docs_md, usage_md) = page_meta(selected);

    let header = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    |cx| {
                        vec![
                            cx.text(title),
                            cx.text_props(TextProps {
                                layout: Default::default(),
                                text: Arc::from(origin),
                                style: None,
                                color: Some(theme.color_required("muted-foreground")),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                            }),
                        ]
                    },
                ),
                shadcn::Badge::new(origin)
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx),
            ]
        },
    );

    let preview_panel = page_preview(cx, theme, selected, popover_open, dialog_open);
    let docs_panel = markdown::Markdown::new(Arc::from(docs_md)).into_element(cx);
    let usage_panel = markdown::Markdown::new(Arc::from(usage_md)).into_element(cx);

    let tabs = shadcn::Tabs::new(content_tab)
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(true)
        .items([
            shadcn::TabsItem::new("preview", "Preview", vec![preview_panel]),
            shadcn::TabsItem::new("usage", "Usage", vec![usage_panel]),
            shadcn::TabsItem::new("docs", "Notes", vec![docs_panel]),
        ])
        .into_element(cx);

    let content = shadcn::ScrollArea::new(vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N6),
        |_cx| vec![header, tabs],
    )])
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);

    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("background")))
                .p(Space::N6),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| vec![content],
    )
}

fn page_meta(selected: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match selected {
        PAGE_LAYOUT => (
            "Layout / Stacks & Constraints",
            "fret-ui + fret-ui-kit",
            DOC_LAYOUT,
            USAGE_LAYOUT,
        ),
        PAGE_BUTTON => ("Button", "fret-ui-shadcn", DOC_BUTTON, USAGE_BUTTON),
        PAGE_OVERLAY => (
            "Overlay / Popover & Dialog",
            "fret-ui-shadcn (Radix-shaped primitives)",
            DOC_OVERLAY,
            USAGE_OVERLAY,
        ),
        _ => ("Introduction", "Core contracts", DOC_INTRO, USAGE_INTRO),
    }
}

fn page_preview(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
) -> AnyElement {
    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_BUTTON => preview_button(cx),
        PAGE_OVERLAY => preview_overlay(cx, popover_open, dialog_open),
        _ => preview_intro(cx, theme),
    };

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Preview").into_element(cx),
            shadcn::CardDescription::new("Interactive preview for validating behaviors.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(body).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn preview_intro(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(desc)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full(),
        );
        cx.container(props, |cx| {
            vec![cx.text("Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.")]
        })
    };

    vec![grid, note]
}

fn preview_layout(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                LayoutRefinement::default().w_full(),
            ),
            |cx| vec![cx.text(label)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        cx.text("Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing)."),
        row,
    ]
}

fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let variants = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default").into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .into_element(cx),
            ]
        },
    );

    let sizes = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::Button::new("Default")
                    .size(shadcn::ButtonSize::Default)
                    .into_element(cx),
                shadcn::Button::new("Large")
                    .size(shadcn::ButtonSize::Lg)
                    .into_element(cx),
            ]
        },
    );

    vec![variants, sizes]
}

fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    let popover = shadcn::Popover::new(popover_open.clone())
        .auto_focus(true)
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(popover_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new(vec![
                    cx.text("Popover content"),
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .toggle_model(popover_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx)
            },
        );

    let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Dialog")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(dialog_open.clone())
                .into_element(cx)
        },
        |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Dialog").into_element(cx),
                    shadcn::DialogDescription::new("Escape / overlay click closes")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .toggle_model(dialog_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |_cx| vec![popover, dialog],
    )]
}

const DOC_INTRO: &str = r#"
## Goals

This is an **editor-grade UI** gallery app used to:

- Validate that `fret-ui-shadcn` / `fret-ui-kit` / ecosystem components work under real composition.
- Provide a component-doc-site browsing experience (left navigation, right preview + docs).

Phase 1 intentionally uses hardcoded doc strings to validate the interaction path end-to-end.
"#;

const USAGE_INTRO: &str = r#"
```rust
// Native
cargo run -p fret-ui-gallery

// Web (via fret-demo-web host)
cd apps/fret-demo-web
trunk serve --open
// open: http://127.0.0.1:8080/?demo=ui_gallery
```
"#;

const DOC_LAYOUT: &str = r#"
## LayoutRefinement + stack

The gallery shell is a common editor-like layout:

- Fixed-width left navigation (scrollable)
- Right content area (scrollable)

In Fret, this is typically expressed with:

- `LayoutRefinement`: width/height/min/max/fill constraints
- `stack::{hstack,vstack}`: row/column composition & alignment
- `Theme` tokens: design system values like spacing/color/radius
"#;

const USAGE_LAYOUT: &str = r#"
```rust
let root = stack::hstack(
    cx,
    stack::HStackProps::default()
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_stretch(),
    |_cx| vec![sidebar, content],
);
```
"#;

const DOC_BUTTON: &str = r#"
## Button

Validate `variant` / `size` behaviors and default styling consistency.

This layer is **visual recipes**. Interaction policies (hover intent, focus trap, etc.) should live in `fret-ui-kit` / ecosystem crates.
"#;

const USAGE_BUTTON: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let btn = shadcn::Button::new("Save")
    .variant(shadcn::ButtonVariant::Default)
    .into_element(cx);
```
"#;

const DOC_OVERLAY: &str = r#"
## Overlay / Portal

Popover/Dialog are rendered through overlay/portal mechanisms, outside the normal layout flow.

Goals:

- open/close state model binding
- basic policies (ESC, overlay click, focus behavior)
"#;

const USAGE_OVERLAY: &str = r#"
```rust
let open = app.models_mut().insert(false);

let dialog = shadcn::Dialog::new(open.clone()).into_element(
    cx,
    |cx| shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx),
    |cx| shadcn::DialogContent::new(vec![cx.text("Hello")]).into_element(cx),
);
```
"#;

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-ui-gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(1080.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    UiGalleryDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();

    fret_bootstrap::BootstrapBuilder::new(app, build_driver())
        .configure(move |c| {
            *c = config;
        })
        .with_default_diagnostics()
        .with_default_config_files()?
        .with_lucide_icons()
        .preload_icon_svgs_on_gpu_ready()
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl WinitAppDriver for UiGalleryDriver {
    type WindowState = UiGalleryWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            state,
            ..
        } = context;

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        let _ = Self::handle_nav_command(app, state, &command);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        Self::render_ui(app, services, window, state, bounds);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }
}
