use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use fret_core::{
    Edges, FontId, FontWeight, Px, SvgFit, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_ui::SvgSource;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, ScrollAxis, ScrollProps, SvgIconProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use fret_executor::{Executors, Inbox, InboxDrainer, InboxSender};
use fret_runtime::DispatcherHandle;

use super::{InlineMathInfo, MarkdownTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MathJaxMode {
    Inline,
    Display,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathJaxKey {
    mode: MathJaxMode,
    latex: String,
}

#[derive(Debug, Clone)]
struct MathJaxSvgReady {
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
enum MathJaxSvgEntry {
    Loading,
    Ready(MathJaxSvgReady),
    Error(Arc<str>),
}

#[derive(Debug, Clone)]
struct MathJaxInboxMsg {
    window: fret_core::AppWindowId,
    key: MathJaxKey,
    result: Result<MathJaxSvgReady, Arc<str>>,
}

#[derive(Clone)]
struct MathJaxRuntime {
    inner: Arc<MathJaxRuntimeInner>,
}

struct MathJaxRuntimeInner {
    exec: Executors,
    inbox: Inbox<MathJaxInboxMsg>,
    cache: Mutex<HashMap<MathJaxKey, MathJaxSvgEntry>>,
    registered: AtomicBool,
}

impl MathJaxRuntime {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            inner: Arc::new(MathJaxRuntimeInner {
                exec: Executors::new(dispatcher),
                inbox: Inbox::new(Default::default()),
                cache: Mutex::new(HashMap::new()),
                registered: AtomicBool::new(false),
            }),
        }
    }
}

fn mathjax_inbox_drainer(runtime: Arc<MathJaxRuntimeInner>) -> InboxDrainer<MathJaxInboxMsg> {
    InboxDrainer::new(runtime.inbox.clone(), move |host, _window, msg| {
        if let Ok(mut cache) = runtime.cache.lock() {
            match msg.result {
                Ok(ready) => {
                    cache.insert(msg.key, MathJaxSvgEntry::Ready(ready));
                }
                Err(err) => {
                    cache.insert(msg.key, MathJaxSvgEntry::Error(err));
                }
            }
        }
        host.request_redraw(msg.window);
    })
}

fn mathjax_runtime<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<MathJaxRuntime> {
    let dispatcher = cx.app.global::<DispatcherHandle>()?.clone();
    let runtime = cx.app.with_global_mut_untracked(
        || MathJaxRuntime::new(dispatcher),
        |runtime, _host| runtime.clone(),
    );

    if !runtime.inner.registered.swap(true, Ordering::SeqCst) {
        let inner = runtime.inner.clone();
        cx.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, _host| {
                registry.register(Arc::new(mathjax_inbox_drainer(inner)));
            },
        );
    }

    Some(runtime)
}

pub(super) fn render_math_block_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Display, latex.as_ref());

    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;
    scroll_props.layout.size.width = Length::Fill;

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Fill;
    container.padding = Edges::all(markdown_theme.math_block_padding);
    container.background = Some(markdown_theme.math_block_bg);
    container.border = Edges::all(Px(0.0));
    container.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.md"));

    cx.container(container, |cx| {
        vec![cx.scroll(scroll_props, |cx| match entry {
            MathJaxSvgEntry::Ready(ready) => {
                let mut icon = SvgIconProps::new(SvgSource::Bytes(ready.svg_bytes));
                icon.fit = SvgFit::Contain;
                icon.color = markdown_theme.math_block_fg;
                icon.layout.size.height = Length::Px(markdown_theme.math_block_height);
                icon.layout.aspect_ratio = ready.aspect_ratio;
                vec![cx.svg_icon_props(icon)]
            }
            MathJaxSvgEntry::Loading => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: latex.clone(),
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })],
            MathJaxSvgEntry::Error(err) => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(format!("{latex} (mathjax error: {err})")),
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })],
        })]
    })
}

pub(super) fn render_inline_math_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Inline, info.latex.as_ref());
    match entry {
        MathJaxSvgEntry::Ready(ready) => render_inline_math_svg(
            cx,
            theme,
            markdown_theme,
            ready.svg_bytes,
            ready.aspect_ratio,
        ),
        MathJaxSvgEntry::Loading => render_inline_math_source(cx, theme, markdown_theme, info),
        MathJaxSvgEntry::Error(err) => render_inline_math_source(
            cx,
            theme,
            markdown_theme,
            InlineMathInfo {
                latex: Arc::<str>::from(format!("{} (mathjax error: {err})", info.latex)),
            },
        ),
    }
}

fn render_inline_math_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.padding = Edges {
        top: markdown_theme.inline_math_padding_y,
        right: markdown_theme.inline_math_padding_x,
        bottom: markdown_theme.inline_math_padding_y,
        left: markdown_theme.inline_math_padding_x,
    };
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

    cx.container(props, |cx| {
        let mut icon = SvgIconProps::new(SvgSource::Bytes(svg_bytes));
        icon.fit = SvgFit::Contain;
        icon.color = markdown_theme.inline_math_fg;
        icon.layout.size.height = Length::Px(markdown_theme.inline_math_height);
        icon.layout.aspect_ratio = aspect_ratio;
        vec![cx.svg_icon_props(icon)]
    })
}

fn render_inline_math_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.padding = Edges {
        top: markdown_theme.inline_math_padding_y,
        right: markdown_theme.inline_math_padding_x,
        bottom: markdown_theme.inline_math_padding_y,
        left: markdown_theme.inline_math_padding_x,
    };
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::<str>::from(info.latex.trim().to_string()),
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: TextSlant::Normal,
                line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(markdown_theme.inline_math_fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}

fn mathjax_svg_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: MathJaxMode,
    latex: &str,
) -> MathJaxSvgEntry {
    fn spawn_mathjax_svg(
        runtime: &MathJaxRuntime,
        window: fret_core::AppWindowId,
        key: MathJaxKey,
        inbox: InboxSender<MathJaxInboxMsg>,
    ) {
        let latex = key.latex.clone();
        runtime
            .inner
            .exec
            .spawn_background_to_inbox(Some(window), inbox, move |_| {
                tracing::debug!(
                    target: "fret_markdown::math",
                    mode = ?key.mode,
                    latex_len = latex.len(),
                    "mathjax svg: convert queued"
                );

                let result = std::panic::catch_unwind(|| match key.mode {
                    MathJaxMode::Inline => mathjax_svg::convert_to_svg_inline(&latex),
                    MathJaxMode::Display => mathjax_svg::convert_to_svg(&latex),
                });

                match result {
                    Ok(Ok(svg)) => {
                        let has_current_color =
                            svg.contains("currentColor") || svg.contains("currentcolor");
                        let svg = if has_current_color {
                            svg.replace("currentColor", "#000000")
                                .replace("currentcolor", "#000000")
                        } else {
                            svg
                        };

                        tracing::debug!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            has_current_color,
                            "mathjax svg: converted"
                        );

                        let aspect_ratio = svg_viewbox_aspect_ratio(&svg);
                        MathJaxInboxMsg {
                            window,
                            key,
                            result: Ok(MathJaxSvgReady {
                                svg_bytes: Arc::<[u8]>::from(svg.into_bytes()),
                                aspect_ratio,
                            }),
                        }
                    }
                    Ok(Err(err)) => {
                        tracing::warn!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            error = %err,
                            "mathjax svg: convert failed"
                        );
                        MathJaxInboxMsg {
                            window,
                            key,
                            result: Err(Arc::<str>::from(err.to_string())),
                        }
                    }
                    Err(_) => MathJaxInboxMsg {
                        window,
                        key,
                        result: Err(Arc::<str>::from("mathjax svg: panic")),
                    },
                }
            })
            .detach();
    }

    let latex = latex.trim();
    if latex.is_empty() {
        return MathJaxSvgEntry::Error(Arc::<str>::from("empty latex"));
    }

    let key = MathJaxKey {
        mode,
        latex: latex.to_string(),
    };

    let Some(runtime) = mathjax_runtime(cx) else {
        return MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg: dispatcher unavailable"));
    };

    let mut needs_redraw = false;
    let mut should_spawn = false;

    let entry = match runtime.inner.cache.lock() {
        Ok(mut cache) => match cache.get(&key) {
            Some(existing) => {
                if matches!(existing, MathJaxSvgEntry::Loading) {
                    needs_redraw = true;
                }
                existing.clone()
            }
            None => {
                cache.insert(key.clone(), MathJaxSvgEntry::Loading);
                needs_redraw = true;
                should_spawn = true;
                MathJaxSvgEntry::Loading
            }
        },
        Err(_) => MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg cache lock poisoned")),
    };

    if needs_redraw {
        cx.app.request_redraw(cx.window);
    }

    if should_spawn {
        spawn_mathjax_svg(&runtime, cx.window, key, runtime.inner.inbox.sender());
    }

    entry
}

fn svg_viewbox_aspect_ratio(svg: &str) -> Option<f32> {
    let idx = svg.find("viewBox=")?;
    let rest = &svg[idx + "viewBox=".len()..];
    let mut chars = rest.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = chars.as_str();
    let end = rest.find(quote)?;
    let value = &rest[..end];

    let mut nums: [f32; 4] = [0.0; 4];
    let mut i = 0usize;
    for part in value.split(|c: char| c.is_whitespace() || c == ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if i >= 4 {
            break;
        }
        nums[i] = part.parse::<f32>().ok()?;
        i += 1;
    }
    if i < 4 {
        return None;
    }
    let w = nums[2];
    let h = nums[3];
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(w / h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    use fret_runtime::{
        DispatchPriority, Dispatcher, ExecCapabilities, InboxDrain, InboxDrainHost, Runnable,
    };

    #[derive(Default)]
    struct TestDispatcher {
        background: Mutex<Vec<Runnable>>,
        wakes: AtomicUsize,
    }

    impl TestDispatcher {
        fn drain_background(&self) {
            let tasks = {
                let Ok(mut guard) = self.background.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            for task in tasks {
                task();
            }
        }
    }

    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, _task: Runnable) {}

        fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
            let Ok(mut guard) = self.background.lock() else {
                return;
            };
            guard.push(task);
        }

        fn dispatch_after(&self, _delay: std::time::Duration, _task: Runnable) {}

        fn wake(&self, _window: Option<fret_core::AppWindowId>) {
            self.wakes.fetch_add(1, AtomicOrdering::Relaxed);
        }

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    #[derive(Default)]
    struct TestDrainHost {
        models: fret_runtime::ModelStore,
        redraws: Vec<fret_core::AppWindowId>,
        effects: Vec<fret_runtime::Effect>,
    }

    impl InboxDrainHost for TestDrainHost {
        fn request_redraw(&mut self, window: fret_core::AppWindowId) {
            self.redraws.push(window);
        }

        fn push_effect(&mut self, effect: fret_runtime::Effect) {
            self.effects.push(effect);
        }

        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            &mut self.models
        }
    }

    #[test]
    fn mathjax_background_to_inbox_wakes_and_drains_deterministically() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let runtime = MathJaxRuntime::new(dispatcher.clone());
        let drainer = mathjax_inbox_drainer(runtime.inner.clone());

        let window = fret_core::AppWindowId::default();
        let key = MathJaxKey {
            mode: MathJaxMode::Inline,
            latex: "x".to_string(),
        };

        let svg_bytes: Arc<[u8]> = Arc::from("<svg viewBox=\"0 0 1 1\"/>".as_bytes());
        let msg = MathJaxInboxMsg {
            window,
            key: key.clone(),
            result: Ok(MathJaxSvgReady {
                svg_bytes,
                aspect_ratio: Some(1.0),
            }),
        };

        runtime
            .inner
            .exec
            .spawn_background_to_inbox(Some(window), runtime.inner.inbox.sender(), move |_| msg)
            .detach();

        dispatcher.drain_background();
        assert_eq!(dispatcher.wakes.load(AtomicOrdering::Relaxed), 1);

        let mut host = TestDrainHost::default();
        assert!(drainer.drain(&mut host, None));
        assert_eq!(host.redraws, vec![window]);

        let cache = runtime.inner.cache.lock().unwrap();
        assert!(matches!(cache.get(&key), Some(MathJaxSvgEntry::Ready(_))));
    }
}
