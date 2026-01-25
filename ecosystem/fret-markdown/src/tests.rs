use super::*;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use crate::mermaid::MermaidDiagramType;
use fret_core::{AppWindowId, ClipboardToken, ImageUploadToken, Point, PointerId, TimerToken};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost, TickId, TimeHost,
};
use fret_ui::ThemeConfig;

#[derive(Default)]
struct ThemeTestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drags: HashMap<PointerId, DragSession>,
    next_drag_session_id: u64,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_image_upload_token: u64,
}

impl GlobalsHost for ThemeTestHost {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        #[derive(Debug)]
        struct GlobalLeaseMarker;

        struct Guard<T: Any> {
            type_id: TypeId,
            value: Option<T>,
            globals: *mut HashMap<TypeId, Box<dyn Any>>,
        }

        impl<T: Any> Drop for Guard<T> {
            fn drop(&mut self) {
                let Some(value) = self.value.take() else {
                    return;
                };
                unsafe {
                    (*self.globals).insert(self.type_id, Box::new(value));
                }
            }
        }

        let type_id = TypeId::of::<T>();
        let existing = self
            .globals
            .insert(type_id, Box::new(GlobalLeaseMarker) as Box<dyn Any>);

        let existing = match existing {
            None => None,
            Some(v) => {
                if v.is::<GlobalLeaseMarker>() {
                    panic!("global already leased: {type_id:?}");
                }
                Some(*v.downcast::<T>().expect("global type id must match"))
            }
        };

        let mut guard = Guard::<T> {
            type_id,
            value: Some(existing.unwrap_or_else(init)),
            globals: &mut self.globals as *mut _,
        };

        let result = {
            let value = guard.value.as_mut().expect("guard value exists");
            f(value, self)
        };

        drop(guard);
        result
    }
}

impl ModelHost for ThemeTestHost {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl ModelsHost for ThemeTestHost {
    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for ThemeTestHost {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for ThemeTestHost {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

impl TimeHost for ThemeTestHost {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        TimerToken(self.next_timer_token)
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        ClipboardToken(self.next_clipboard_token)
    }

    fn next_image_upload_token(&mut self) -> ImageUploadToken {
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        ImageUploadToken(self.next_image_upload_token)
    }
}

impl DragHost for ThemeTestHost {
    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drags.values().any(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<PointerId> {
        self.drags
            .values()
            .find(|d| predicate(d))
            .map(|d| d.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<PointerId> {
        let to_cancel: Vec<PointerId> = self
            .drags
            .values()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
            .collect();
        for pointer_id in &to_cancel {
            self.cancel_drag(*pointer_id);
        }
        to_cancel
    }

    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: PointerId) {
        self.drags.remove(&pointer_id);
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        let session_id = DragSessionId(self.next_drag_session_id);
        self.drags.insert(
            pointer_id,
            DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
        );
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        let session_id = DragSessionId(self.next_drag_session_id);
        self.drags.insert(
            pointer_id,
            DragSession::new_cross_window(
                session_id,
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ),
        );
    }
}

fn theme_with_metrics(metrics: &[(&str, f32)]) -> Theme {
    let mut host = ThemeTestHost::default();
    Theme::with_global_mut(&mut host, |theme| {
        let mut cfg = ThemeConfig {
            name: theme.name.clone(),
            author: theme.author.clone(),
            url: theme.url.clone(),
            colors: HashMap::new(),
            metrics: HashMap::new(),
            ..ThemeConfig::default()
        };
        for (k, v) in metrics {
            cfg.metrics.insert((*k).to_string(), *v);
        }
        theme.apply_config(&cfg);
    });

    Theme::global(&host).clone()
}

fn count_top_level_list_items(events: &[pulldown_cmark::Event<'static>]) -> usize {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut list_depth = 0usize;
    let mut count = 0usize;

    for e in events {
        match e {
            Event::Start(Tag::List(_)) => list_depth += 1,
            Event::End(TagEnd::List(_)) => list_depth = list_depth.saturating_sub(1),
            Event::Start(Tag::Item) if list_depth == 1 => count += 1,
            _ => {}
        }
    }

    count
}

#[test]
fn parses_fenced_language_variants() {
    assert_eq!(parse_fenced_code_language("rust").as_deref(), Some("rust"));
    assert_eq!(
        parse_fenced_code_language("rust,ignore").as_deref(),
        Some("rust")
    );
    assert_eq!(
        parse_fenced_code_language("language-rust").as_deref(),
        Some("rust")
    );
    assert_eq!(
        parse_fenced_code_language("{.rust .numberLines}").as_deref(),
        Some("rust")
    );
}

#[test]
fn detects_mermaid_diagram_type() {
    assert_eq!(
        detect_mermaid_diagram_type("flowchart TD\n  A --> B"),
        MermaidDiagramType::Flowchart
    );
    assert_eq!(
        detect_mermaid_diagram_type("%% comment\nsequenceDiagram\n  A->>B: hi"),
        MermaidDiagramType::Sequence
    );
    assert_eq!(
        detect_mermaid_diagram_type("classDiagram\n  A <|-- B"),
        MermaidDiagramType::Class
    );
    assert_eq!(detect_mermaid_diagram_type(""), MermaidDiagramType::Unknown);
}

#[test]
fn is_mermaid_language_is_case_insensitive() {
    assert!(is_mermaid_language(Some("mermaid")));
    assert!(is_mermaid_language(Some("Mermaid")));
    assert!(!is_mermaid_language(Some("rust")));
    assert!(!is_mermaid_language(None));
}

#[test]
fn mdstream_assigns_stable_ids_for_full_source() {
    let source = "# A\n\nB\n\n```rust\nfn main() {}\n```\n";

    let mut s1 = mdstream::MdStream::default();
    let mut st1 = MarkdownPulldownState::new();
    st1.apply_update(s1.append(source));
    let ids1: Vec<_> = st1.doc().committed().iter().map(|b| b.id).collect();

    let mut s2 = mdstream::MdStream::default();
    let mut st2 = MarkdownPulldownState::new();
    st2.apply_update(s2.append(source));
    let ids2: Vec<_> = st2.doc().committed().iter().map(|b| b.id).collect();

    assert!(!ids1.is_empty());
    assert_eq!(ids1, ids2);
}

#[test]
fn mdstream_pulldown_state_applies_incrementally() {
    let mut stream = mdstream::MdStream::default();
    let mut state = MarkdownPulldownState::new();

    let u1 = stream.append("Hello\n\n```rust\nfn main() {");
    let a1 = state.apply_update(u1);
    assert!(!a1.reset);
    assert_eq!(state.doc().committed().len(), 1);
    assert!(state.doc().pending().is_some());

    let u2 = stream.append("}\n```\n");
    let _a2 = state.apply_update(u2);
    assert_eq!(state.doc().committed().len(), 2);
    assert!(state.doc().pending().is_none());
}

#[test]
fn markdown_stream_state_keeps_blocks_with_footnotes() {
    let source = r#"# A

Footnotes are supported.[^note]

[^note]: This is a footnote definition.

$$
\int_0^1 x^2\,dx = \frac{1}{3}
$$
"#;

    let mut st = MarkdownStreamState::new();
    st.append(source);
    st.finalize();

    let committed = st.state().doc().committed();
    assert!(committed.len() > 1);
    assert!(
        committed
            .iter()
            .any(|b| b.kind == mdstream::BlockKind::FootnoteDefinition)
    );
    assert!(
        committed
            .iter()
            .any(|b| b.kind == mdstream::BlockKind::MathBlock)
    );
}

#[test]
fn parses_list_items() {
    let info = parse_list_info("- a\n- b\n  c\n");
    assert!(!info.ordered);
    assert_eq!(info.items.len(), 2);
    assert_eq!(info.items[0].as_ref(), "a");
    assert_eq!(info.items[1].as_ref(), "b\nc");

    let info = parse_list_info("2. a\n3. b\n");
    assert!(info.ordered);
    assert_eq!(info.start, 2);
    assert_eq!(info.items.len(), 2);
    assert_eq!(info.items[0].as_ref(), "a");
    assert_eq!(info.items[1].as_ref(), "b");
}

#[test]
fn strips_blockquote_prefixes() {
    let text = Arc::<str>::from("> a\n> b\n  > c\n");
    let out = strip_blockquote_prefix(&text);
    assert_eq!(out.as_ref(), "a\nb\nc");
}

#[test]
fn pulldown_extracts_link_and_strong() {
    let events = parse_events("Hello **world** [link](https://example.com)\n");
    let pieces = inline_pieces_from_events_unwrapped(&events);
    assert!(pieces.iter().any(|p| p.style.strong));
    assert!(pieces.iter().any(|p| p.style.link.is_some()));
}

#[test]
fn pulldown_counts_list_items() {
    let events = parse_events("- a\n- b\n");
    assert_eq!(count_top_level_list_items(&events), 2);
}

#[test]
fn pulldown_parses_gfm_task_list_marker() {
    use pulldown_cmark::Event;
    let events = parse_events("- [x] done\n- [ ] todo\n");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TaskListMarker(true)))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TaskListMarker(false)))
    );
}

#[test]
fn pulldown_parses_gfm_autolinks_when_enabled() {
    let events = parse_events("<https://example.com>\n");
    let pieces = inline_pieces_from_events_unwrapped(&events);

    assert!(pieces.iter().any(|p| {
        let InlinePieceKind::Text(text) = &p.kind else {
            return false;
        };
        text.contains("https://example.com")
            && p.style.link.as_deref() == Some("https://example.com")
    }));
}

#[test]
fn pulldown_parses_strikethrough_when_enabled() {
    use pulldown_cmark::{Event, Tag};
    let events = parse_events("~~gone~~\n");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Strikethrough)))
    );
}

#[test]
fn pulldown_parses_image_and_collects_alt_text() {
    let events = parse_events("![alt **bold** `code`](https://example.com/a.png \"t\")\n");
    let pieces = inline_pieces_from_events_unwrapped(&events);

    let imgs: Vec<_> = pieces
        .iter()
        .filter_map(|p| match &p.kind {
            InlinePieceKind::Image(info) => Some(info),
            _ => None,
        })
        .collect();

    assert_eq!(imgs.len(), 1);
    assert_eq!(imgs[0].src.as_ref(), "https://example.com/a.png");
    assert_eq!(imgs[0].alt.as_ref(), "alt bold code");
    assert_eq!(imgs[0].title.as_deref(), Some("t"));
    assert!(!imgs[0].is_svg);
}

#[test]
fn pulldown_maps_inline_br_html_to_line_break() {
    let events = parse_events("a<br>b\n");
    let pieces = inline_pieces_from_events_unwrapped(&events);
    assert!(
        pieces
            .iter()
            .any(|p| matches!(&p.kind, InlinePieceKind::Text(t) if t.contains('\n')))
    );
}

#[test]
fn autolinks_bare_urls_in_plain_text() {
    let style = InlineStyle {
        strong: false,
        emphasis: false,
        strikethrough: false,
        code: false,
        link: None,
    };
    let pieces = split_piece_into_tokens("See https://example.com).", &style);

    assert!(pieces.iter().any(|p| {
        let InlinePieceKind::Text(text) = &p.kind else {
            return false;
        };
        text == "https://example.com" && p.style.link.as_deref() == Some("https://example.com")
    }));
}

#[test]
fn pulldown_parses_inline_math_when_enabled() {
    use pulldown_cmark::Event;
    let events = parse_events("$x^2$\n");
    assert!(events.iter().any(|e| matches!(e, Event::InlineMath(_))));
}

#[test]
fn pulldown_parses_display_math_when_enabled() {
    use pulldown_cmark::Event;
    let events = parse_events("$$x^2$$\n");
    assert!(events.iter().any(|e| matches!(e, Event::DisplayMath(_))));
}

#[test]
fn pulldown_parses_multiline_display_math_when_enabled() {
    use pulldown_cmark::Event;
    let events = parse_events("$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n");
    assert!(events.iter().any(|e| matches!(e, Event::DisplayMath(_))));
}

#[test]
fn mdstream_math_block_body_strips_common_delimiters() {
    let mut stream = mdstream::MdStream::default();
    let update = stream.append("$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n");

    let mut state = MarkdownPulldownState::new();
    state.apply_update(update);

    let blocks = state.doc().committed();
    let math = blocks
        .iter()
        .find(|b| matches!(b.kind, mdstream::BlockKind::MathBlock))
        .expect("math block exists");

    let body = parse_math_block_body(math.display_or_raw());
    assert!(body.contains("\\int_0^1"));
}

#[test]
fn detects_display_math_only_events() {
    let events = parse_events("$$x^2$$\n");
    let latex = display_math_only_events(&events);
    assert!(latex.is_some());
    assert_eq!(latex.unwrap().as_ref(), "x^2");
}

#[test]
fn open_url_filter_is_conservative() {
    assert!(is_safe_open_url("https://example.com"));
    assert!(is_safe_open_url("http://example.com"));
    assert!(is_safe_open_url("mailto:test@example.com"));

    assert!(!is_safe_open_url(""));
    assert!(!is_safe_open_url("   "));
    assert!(!is_safe_open_url("javascript:alert(1)"));
    assert!(!is_safe_open_url("data:text/html;base64,PHNjcmlwdD4="));
    assert!(!is_safe_open_url("file:///etc/passwd"));
}

#[test]
fn code_block_max_height_prefers_fret_namespace() {
    let theme = theme_with_metrics(&[
        ("markdown.code_block.max_height", 111.0),
        ("fret.markdown.code_block.max_height", 222.0),
    ]);
    let mut options = fret_code_view::CodeBlockUiOptions::default();
    options.max_height = None;
    resolve_code_block_ui(&theme, &mut options);
    assert_eq!(options.max_height, Some(Px(222.0)));
}

#[test]
fn code_block_max_height_falls_back_to_markdown_namespace() {
    let theme = theme_with_metrics(&[("markdown.code_block.max_height", 123.0)]);
    let mut options = fret_code_view::CodeBlockUiOptions::default();
    options.max_height = None;
    resolve_code_block_ui(&theme, &mut options);
    assert_eq!(options.max_height, Some(Px(123.0)));
}

#[test]
fn code_block_max_height_does_not_override_explicit_option() {
    let theme = theme_with_metrics(&[("fret.markdown.code_block.max_height", 999.0)]);
    let mut options = fret_code_view::CodeBlockUiOptions::default();
    options.max_height = Some(Px(321.0));
    resolve_code_block_ui(&theme, &mut options);
    assert_eq!(options.max_height, Some(Px(321.0)));
}
