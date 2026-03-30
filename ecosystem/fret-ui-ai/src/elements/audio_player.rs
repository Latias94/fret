//! AI Elements-aligned `AudioPlayer` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/audio-player.tsx`.

use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, ui};
use fret_ui_shadcn::facade::{
    Button, ButtonGroup, ButtonGroupItem, ButtonGroupOrientation, ButtonGroupText, ButtonSize,
    ButtonVariant, Slider,
};

pub type OnAudioPlayerPlayChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, bool) + 'static>;
pub type OnAudioPlayerMuteChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, bool) + 'static>;
pub type OnAudioPlayerSeekTo =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, f32) + 'static>;
pub type OnAudioPlayerVolumeChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, f32) + 'static>;

#[derive(Clone)]
pub struct AudioPlayerController {
    pub playing: Model<bool>,
    pub muted: Model<bool>,
    /// Current time in seconds (single-thumb slider model).
    pub time: Model<Vec<f32>>,
    /// Total duration in seconds.
    pub duration_secs: Model<f32>,
    /// Volume in [0..=1] (single-thumb slider model).
    pub volume: Model<Vec<f32>>,

    pub on_playing_change: Option<OnAudioPlayerPlayChange>,
    pub on_mute_change: Option<OnAudioPlayerMuteChange>,
    pub on_seek_to: Option<OnAudioPlayerSeekTo>,
    pub on_volume_change: Option<OnAudioPlayerVolumeChange>,
}

impl std::fmt::Debug for AudioPlayerController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayerController")
            .field("playing", &self.playing.id())
            .field("muted", &self.muted.id())
            .field("time", &self.time.id())
            .field("duration_secs", &self.duration_secs.id())
            .field("volume", &self.volume.id())
            .field("has_on_playing_change", &self.on_playing_change.is_some())
            .field("has_on_mute_change", &self.on_mute_change.is_some())
            .field("has_on_seek_to", &self.on_seek_to.is_some())
            .field("has_on_volume_change", &self.on_volume_change.is_some())
            .finish()
    }
}

pub fn use_audio_player_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<AudioPlayerController> {
    cx.provided::<AudioPlayerController>().cloned()
}

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn transparent_button_group_text_chrome(theme: &Theme) -> ChromeRefinement {
    ChromeRefinement::default()
        .bg(ColorRef::Color(Color::TRANSPARENT))
        .text_color(ColorRef::Color(muted_fg(theme)))
}

fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

fn read_single(values: &[f32], fallback: f32) -> f32 {
    values.first().copied().unwrap_or(fallback)
}

fn format_clock(seconds: f32) -> Arc<str> {
    let s = seconds.max(0.0).round() as u64;
    let h = s / 3600;
    let m = (s / 60) % 60;
    let sec = s % 60;
    if h > 0 {
        Arc::<str>::from(format!("{h}:{m:02}:{sec:02}"))
    } else {
        Arc::<str>::from(format!("{m}:{sec:02}"))
    }
}

/// Root provider aligned with AI Elements `AudioPlayer`.
#[derive(Clone)]
pub struct AudioPlayer {
    playing: Option<Model<bool>>,
    default_playing: bool,
    muted: Option<Model<bool>>,
    default_muted: bool,
    time: Option<Model<Vec<f32>>>,
    default_time_secs: f32,
    duration_secs: Option<Model<f32>>,
    default_duration_secs: f32,
    volume: Option<Model<Vec<f32>>>,
    default_volume: f32,

    disabled: bool,

    on_playing_change: Option<OnAudioPlayerPlayChange>,
    on_mute_change: Option<OnAudioPlayerMuteChange>,
    on_seek_to: Option<OnAudioPlayerSeekTo>,
    on_volume_change: Option<OnAudioPlayerVolumeChange>,

    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("has_playing", &self.playing.is_some())
            .field("has_muted", &self.muted.is_some())
            .field("has_time", &self.time.is_some())
            .field("has_duration_secs", &self.duration_secs.is_some())
            .field("has_volume", &self.volume.is_some())
            .field("disabled", &self.disabled)
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            playing: None,
            default_playing: false,
            muted: None,
            default_muted: false,
            time: None,
            default_time_secs: 0.0,
            duration_secs: None,
            default_duration_secs: 0.0,
            volume: None,
            default_volume: 1.0,
            disabled: false,
            on_playing_change: None,
            on_mute_change: None,
            on_seek_to: None,
            on_volume_change: None,
            test_id_root: None,
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn playing_model(mut self, model: Model<bool>) -> Self {
        self.playing = Some(model);
        self
    }

    pub fn default_playing(mut self, playing: bool) -> Self {
        self.default_playing = playing;
        self
    }

    pub fn muted_model(mut self, model: Model<bool>) -> Self {
        self.muted = Some(model);
        self
    }

    pub fn default_muted(mut self, muted: bool) -> Self {
        self.default_muted = muted;
        self
    }

    pub fn time_model(mut self, model: Model<Vec<f32>>) -> Self {
        self.time = Some(model);
        self
    }

    pub fn default_time_secs(mut self, secs: f32) -> Self {
        self.default_time_secs = secs.max(0.0);
        self
    }

    pub fn duration_secs_model(mut self, model: Model<f32>) -> Self {
        self.duration_secs = Some(model);
        self
    }

    pub fn default_duration_secs(mut self, secs: f32) -> Self {
        self.default_duration_secs = secs.max(0.0);
        self
    }

    pub fn volume_model(mut self, model: Model<Vec<f32>>) -> Self {
        self.volume = Some(model);
        self
    }

    pub fn default_volume(mut self, volume: f32) -> Self {
        self.default_volume = clamp_f32(volume, 0.0, 1.0);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_playing_change(mut self, cb: OnAudioPlayerPlayChange) -> Self {
        self.on_playing_change = Some(cb);
        self
    }

    pub fn on_mute_change(mut self, cb: OnAudioPlayerMuteChange) -> Self {
        self.on_mute_change = Some(cb);
        self
    }

    pub fn on_seek_to(mut self, cb: OnAudioPlayerSeekTo) -> Self {
        self.on_seek_to = Some(cb);
        self
    }

    pub fn on_volume_change(mut self, cb: OnAudioPlayerVolumeChange) -> Self {
        self.on_volume_change = Some(cb);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn children<I, C>(self, children: I) -> AudioPlayerWithChildren
    where
        I: IntoIterator<Item = C>,
        C: Into<AudioPlayerChild>,
    {
        AudioPlayerWithChildren {
            root: self,
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn element(self, element: AudioPlayerElement) -> AudioPlayerWithChildren {
        self.children([AudioPlayerChild::Element(element)])
    }

    pub fn control_bar(self, control_bar: AudioPlayerControlBar) -> AudioPlayerWithChildren {
        self.children([AudioPlayerChild::ControlBar(control_bar)])
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, AudioPlayerController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = self.layout;
        let chrome = self.chrome;
        let disabled = self.disabled;

        let controlled_playing = self.playing.clone();
        let default_playing = self.default_playing;
        let controlled_muted = self.muted.clone();
        let default_muted = self.default_muted;
        let controlled_time = self.time.clone();
        let default_time_secs = self.default_time_secs;
        let controlled_duration = self.duration_secs.clone();
        let default_duration_secs = self.default_duration_secs;
        let controlled_volume = self.volume.clone();
        let default_volume = self.default_volume;

        let on_playing_change = self.on_playing_change.clone();
        let on_mute_change = self.on_mute_change.clone();
        let on_seek_to = self.on_seek_to.clone();
        let on_volume_change = self.on_volume_change.clone();
        let test_id_root = self.test_id_root.clone();

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                let playing = controllable_state::use_controllable_model(
                    cx,
                    controlled_playing.clone(),
                    || default_playing,
                )
                .model();
                let muted = controllable_state::use_controllable_model(
                    cx,
                    controlled_muted.clone(),
                    || default_muted,
                )
                .model();

                let time =
                    controllable_state::use_controllable_model(cx, controlled_time.clone(), || {
                        vec![default_time_secs.max(0.0)]
                    })
                    .model();
                let duration_secs = controllable_state::use_controllable_model(
                    cx,
                    controlled_duration.clone(),
                    || default_duration_secs.max(0.0),
                )
                .model();
                let volume = controllable_state::use_controllable_model(
                    cx,
                    controlled_volume.clone(),
                    || vec![clamp_f32(default_volume, 0.0, 1.0)],
                )
                .model();

                let controller = AudioPlayerController {
                    playing,
                    muted,
                    time,
                    duration_secs,
                    volume,
                    on_playing_change: on_playing_change.clone(),
                    on_mute_change: on_mute_change.clone(),
                    on_seek_to: on_seek_to.clone(),
                    on_volume_change: on_volume_change.clone(),
                };

                cx.provide(controller.clone(), |cx| {
                    let mut out = children(cx, controller);
                    if disabled {
                        // Keep the subtree mounted for layout, but non-interactive.
                        out = vec![cx.interactivity_gate_props(
                            InteractivityGateProps {
                                layout: LayoutStyle::default(),
                                present: true,
                                interactive: false,
                            },
                            move |_cx| out,
                        )];
                    }
                    out
                })
            },
        );

        let Some(test_id) = test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

pub enum AudioPlayerChild {
    Element(AudioPlayerElement),
    ControlBar(AudioPlayerControlBar),
    Node(AnyElement),
}

impl From<AudioPlayerElement> for AudioPlayerChild {
    fn from(value: AudioPlayerElement) -> Self {
        Self::Element(value)
    }
}

impl From<AudioPlayerControlBar> for AudioPlayerChild {
    fn from(value: AudioPlayerControlBar) -> Self {
        Self::ControlBar(value)
    }
}

impl From<AnyElement> for AudioPlayerChild {
    fn from(value: AnyElement) -> Self {
        Self::Node(value)
    }
}

pub struct AudioPlayerWithChildren {
    root: AudioPlayer,
    children: Vec<AudioPlayerChild>,
}

impl AudioPlayerWithChildren {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<AudioPlayerChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn element(self, element: AudioPlayerElement) -> Self {
        self.children([AudioPlayerChild::Element(element)])
    }

    pub fn control_bar(self, control_bar: AudioPlayerControlBar) -> Self {
        self.children([AudioPlayerChild::ControlBar(control_bar)])
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self { root, children } = self;
        root.into_element_with_children(cx, move |cx, _controller| {
            children
                .into_iter()
                .map(|child| match child {
                    AudioPlayerChild::Element(value) => value.into_element(cx),
                    AudioPlayerChild::ControlBar(value) => value.into_element(cx),
                    AudioPlayerChild::Node(value) => value,
                })
                .collect()
        })
    }
}

/// Wrapper aligned with AI Elements `AudioPlayerControlBar`.
pub struct AudioPlayerControlBar {
    children: Vec<AudioPlayerControlBarChild>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AudioPlayerControlBar {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn new<I, C>(children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<AudioPlayerControlBarChild>,
    {
        Self::empty().children(children)
    }

    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<AudioPlayerControlBarChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn play_button(self, button: AudioPlayerPlayButton) -> Self {
        self.children([AudioPlayerControlBarChild::PlayButton(button)])
    }

    pub fn seek_backward_button(self, button: AudioPlayerSeekBackwardButton) -> Self {
        self.children([AudioPlayerControlBarChild::SeekBackwardButton(button)])
    }

    pub fn seek_forward_button(self, button: AudioPlayerSeekForwardButton) -> Self {
        self.children([AudioPlayerControlBarChild::SeekForwardButton(button)])
    }

    pub fn time_display(self, display: AudioPlayerTimeDisplay) -> Self {
        self.children([AudioPlayerControlBarChild::TimeDisplay(display)])
    }

    pub fn time_range(self, range: AudioPlayerTimeRange) -> Self {
        self.children([AudioPlayerControlBarChild::TimeRange(range)])
    }

    pub fn duration_display(self, display: AudioPlayerDurationDisplay) -> Self {
        self.children([AudioPlayerControlBarChild::DurationDisplay(display)])
    }

    pub fn mute_button(self, button: AudioPlayerMuteButton) -> Self {
        self.children([AudioPlayerControlBarChild::MuteButton(button)])
    }

    pub fn volume_range(self, range: AudioPlayerVolumeRange) -> Self {
        self.children([AudioPlayerControlBarChild::VolumeRange(range)])
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let items = self
            .children
            .into_iter()
            .map(|child| child.into_group_item(cx))
            .collect::<Vec<_>>();

        let mut el = ButtonGroup::new(items).orientation(ButtonGroupOrientation::Horizontal);
        el = el.refine_layout(self.layout);
        let el = el.into_element(cx);

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

impl Default for AudioPlayerControlBar {
    fn default() -> Self {
        Self::empty()
    }
}

pub enum AudioPlayerControlBarChild {
    Item(ButtonGroupItem),
    PlayButton(AudioPlayerPlayButton),
    SeekBackwardButton(AudioPlayerSeekBackwardButton),
    SeekForwardButton(AudioPlayerSeekForwardButton),
    TimeDisplay(AudioPlayerTimeDisplay),
    TimeRange(AudioPlayerTimeRange),
    DurationDisplay(AudioPlayerDurationDisplay),
    MuteButton(AudioPlayerMuteButton),
    VolumeRange(AudioPlayerVolumeRange),
    Node(AnyElement),
}

impl AudioPlayerControlBarChild {
    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        match self {
            Self::Item(value) => value,
            Self::PlayButton(value) => value.into_group_item(cx),
            Self::SeekBackwardButton(value) => value.into_group_item(cx),
            Self::SeekForwardButton(value) => value.into_group_item(cx),
            Self::TimeDisplay(value) => value.into_group_item(cx),
            Self::TimeRange(value) => value.into_group_item(cx),
            Self::DurationDisplay(value) => value.into_group_item(cx),
            Self::MuteButton(value) => value.into_group_item(cx),
            Self::VolumeRange(value) => value.into_group_item(cx),
            Self::Node(value) => value.into(),
        }
    }
}

impl From<ButtonGroupItem> for AudioPlayerControlBarChild {
    fn from(value: ButtonGroupItem) -> Self {
        Self::Item(value)
    }
}

impl From<AudioPlayerPlayButton> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerPlayButton) -> Self {
        Self::PlayButton(value)
    }
}

impl From<AudioPlayerSeekBackwardButton> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerSeekBackwardButton) -> Self {
        Self::SeekBackwardButton(value)
    }
}

impl From<AudioPlayerSeekForwardButton> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerSeekForwardButton) -> Self {
        Self::SeekForwardButton(value)
    }
}

impl From<AudioPlayerTimeDisplay> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerTimeDisplay) -> Self {
        Self::TimeDisplay(value)
    }
}

impl From<AudioPlayerTimeRange> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerTimeRange) -> Self {
        Self::TimeRange(value)
    }
}

impl From<AudioPlayerDurationDisplay> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerDurationDisplay) -> Self {
        Self::DurationDisplay(value)
    }
}

impl From<AudioPlayerMuteButton> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerMuteButton) -> Self {
        Self::MuteButton(value)
    }
}

impl From<AudioPlayerVolumeRange> for AudioPlayerControlBarChild {
    fn from(value: AudioPlayerVolumeRange) -> Self {
        Self::VolumeRange(value)
    }
}

impl From<AnyElement> for AudioPlayerControlBarChild {
    fn from(value: AnyElement) -> Self {
        Self::Node(value)
    }
}

/// Outline icon button aligned with AI Elements `AudioPlayerPlayButton`.
#[derive(Debug, Clone, Default)]
pub struct AudioPlayerPlayButton {
    test_id: Option<Arc<str>>,
}

impl AudioPlayerPlayButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_button<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<Button> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let playing_now = cx
            .get_model_copied(&controller.playing, Invalidation::Layout)
            .unwrap_or(false);

        let label = if playing_now { "Pause" } else { "Play" };
        let icon_id = if playing_now {
            fret_icons::IconId::new_static("lucide.pause")
        } else {
            fret_icons::IconId::new_static("lucide.play")
        };
        let icon = decl_icon::icon(cx, icon_id);

        let playing = controller.playing.clone();
        let on_playing_change = controller.on_playing_change.clone();

        let mut button = Button::new(label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .children([icon])
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut next = playing_now;
                let _ = host.models_mut().update(&playing, |v| {
                    next = !*v;
                    *v = next;
                });
                if let Some(cb) = on_playing_change.clone() {
                    cb(host, action_cx, next);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        Some(button)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_button(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_button(cx)
            .map(|button| button.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Outline icon button aligned with AI Elements `AudioPlayerSeekBackwardButton`.
#[derive(Debug, Clone)]
pub struct AudioPlayerSeekBackwardButton {
    seek_offset_secs: f32,
    test_id: Option<Arc<str>>,
}

impl Default for AudioPlayerSeekBackwardButton {
    fn default() -> Self {
        Self {
            seek_offset_secs: 10.0,
            test_id: None,
        }
    }
}

impl AudioPlayerSeekBackwardButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seek_offset_secs(mut self, offset: f32) -> Self {
        self.seek_offset_secs = offset.max(0.0);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_button<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<Button> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let icon = decl_icon::icon(cx, fret_icons::IconId::new_static("lucide.rotate-ccw"));

        let time = controller.time.clone();
        let duration_secs = controller.duration_secs.clone();
        let on_seek_to = controller.on_seek_to.clone();
        let offset = self.seek_offset_secs;

        let duration_now = cx
            .get_model_copied(&duration_secs, Invalidation::Layout)
            .unwrap_or(0.0);
        let max_now = duration_now.max(0.0);

        let mut button = Button::new("Seek backward")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .children([icon])
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut next_secs = 0.0;
                let _ = host.models_mut().update(&time, |v| {
                    let cur = read_single(v, 0.0);
                    next_secs = clamp_f32(cur - offset, 0.0, max_now);
                    if v.is_empty() {
                        v.push(next_secs);
                    } else {
                        v[0] = next_secs;
                    }
                });
                if let Some(cb) = on_seek_to.clone() {
                    cb(host, action_cx, next_secs);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        Some(button)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_button(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_button(cx)
            .map(|button| button.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Outline icon button aligned with AI Elements `AudioPlayerSeekForwardButton`.
#[derive(Debug, Clone)]
pub struct AudioPlayerSeekForwardButton {
    seek_offset_secs: f32,
    test_id: Option<Arc<str>>,
}

impl Default for AudioPlayerSeekForwardButton {
    fn default() -> Self {
        Self {
            seek_offset_secs: 10.0,
            test_id: None,
        }
    }
}

impl AudioPlayerSeekForwardButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seek_offset_secs(mut self, offset: f32) -> Self {
        self.seek_offset_secs = offset.max(0.0);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_button<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<Button> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let icon = decl_icon::icon(cx, fret_icons::IconId::new_static("lucide.rotate-cw"));

        let time = controller.time.clone();
        let duration_secs = controller.duration_secs.clone();
        let on_seek_to = controller.on_seek_to.clone();
        let offset = self.seek_offset_secs;

        let duration_now = cx
            .get_model_copied(&duration_secs, Invalidation::Layout)
            .unwrap_or(0.0);
        let max_now = duration_now.max(0.0);

        let mut button = Button::new("Seek forward")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .children([icon])
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut next_secs = 0.0;
                let _ = host.models_mut().update(&time, |v| {
                    let cur = read_single(v, 0.0);
                    next_secs = clamp_f32(cur + offset, 0.0, max_now);
                    if v.is_empty() {
                        v.push(next_secs);
                    } else {
                        v[0] = next_secs;
                    }
                });
                if let Some(cb) = on_seek_to.clone() {
                    cb(host, action_cx, next_secs);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        Some(button)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_button(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_button(cx)
            .map(|button| button.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Text label aligned with AI Elements `AudioPlayerTimeDisplay`.
#[derive(Debug, Clone, Default)]
pub struct AudioPlayerTimeDisplay {
    test_id: Option<Arc<str>>,
}

impl AudioPlayerTimeDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_text<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<ButtonGroupText> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let theme = Theme::global(&*cx.app).clone();
        let time_values = cx
            .get_model_cloned(&controller.time, Invalidation::Layout)
            .unwrap_or_default();
        let time_now = read_single(&time_values, 0.0);
        let text = format_clock(time_now);

        let mut text =
            ButtonGroupText::new(text).refine_style(transparent_button_group_text_chrome(&theme));
        if let Some(test_id) = self.test_id {
            text = text.test_id(test_id);
        }
        Some(text)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_text(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_text(cx)
            .map(|text| text.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Slider aligned with AI Elements `AudioPlayerTimeRange`.
#[derive(Debug, Clone)]
pub struct AudioPlayerTimeRange {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Default for AudioPlayerTimeRange {
    fn default() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default()
                .w_px(Px(240.0))
                .min_w_0()
                .min_h_0(),
        }
    }
}

impl AudioPlayerTimeRange {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn build_slider<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controller: AudioPlayerController,
        test_id: Option<Arc<str>>,
        layout: LayoutRefinement,
    ) -> Slider {
        let duration_now = cx
            .get_model_copied(&controller.duration_secs, Invalidation::Layout)
            .unwrap_or(0.0);
        let max = duration_now.max(1.0);
        let on_seek_to = controller.on_seek_to.clone();

        let mut slider = Slider::new(controller.time.clone())
            .range(0.0, max)
            .step(1.0)
            .a11y_label("Seek")
            .refine_layout(layout)
            .on_value_commit(move |host, action_cx, values| {
                let secs = read_single(&values, 0.0);
                if let Some(cb) = on_seek_to.clone() {
                    cb(host, action_cx, secs);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            });
        if let Some(test_id) = test_id {
            slider = slider.test_id(test_id);
        }
        slider
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        let Some(controller) = use_audio_player_controller(cx) else {
            return hidden(cx).into();
        };
        let theme = Theme::global(&*cx.app).clone();
        let slider = Self::build_slider(
            cx,
            controller,
            self.test_id,
            LayoutRefinement::default().w_full().min_w_0().min_h_0(),
        )
        .into_element(cx);
        let slider = ui::h_flex(move |_cx| [slider])
            .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .items_center()
            .into_element(cx);

        ButtonGroupText::new_children([slider])
            .refine_layout(self.layout)
            .refine_style(transparent_button_group_text_chrome(&theme))
            .into()
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_audio_player_controller(cx) else {
            return hidden(cx);
        };

        Self::build_slider(cx, controller, self.test_id, self.layout).into_element(cx)
    }
}

/// Text label aligned with AI Elements `AudioPlayerDurationDisplay`.
#[derive(Debug, Clone, Default)]
pub struct AudioPlayerDurationDisplay {
    test_id: Option<Arc<str>>,
}

impl AudioPlayerDurationDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_text<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<ButtonGroupText> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let theme = Theme::global(&*cx.app).clone();
        let duration_now = cx
            .get_model_copied(&controller.duration_secs, Invalidation::Layout)
            .unwrap_or(0.0);
        let text = format_clock(duration_now);

        let mut text =
            ButtonGroupText::new(text).refine_style(transparent_button_group_text_chrome(&theme));
        if let Some(test_id) = self.test_id {
            text = text.test_id(test_id);
        }
        Some(text)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_text(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_text(cx)
            .map(|text| text.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Outline icon button aligned with AI Elements `AudioPlayerMuteButton`.
#[derive(Debug, Clone, Default)]
pub struct AudioPlayerMuteButton {
    test_id: Option<Arc<str>>,
}

impl AudioPlayerMuteButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_button<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Option<Button> {
        let Some(controller) = use_audio_player_controller(cx) else {
            return None;
        };

        let muted_now = cx
            .get_model_copied(&controller.muted, Invalidation::Layout)
            .unwrap_or(false);

        let label = if muted_now { "Unmute" } else { "Mute" };
        let icon_id = if muted_now {
            fret_icons::IconId::new_static("lucide.volume-x")
        } else {
            fret_icons::IconId::new_static("lucide.volume-2")
        };
        let icon = decl_icon::icon(cx, icon_id);

        let muted = controller.muted.clone();
        let on_mute_change = controller.on_mute_change.clone();

        let mut button = Button::new(label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .children([icon])
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut next = muted_now;
                let _ = host.models_mut().update(&muted, |v| {
                    next = !*v;
                    *v = next;
                });
                if let Some(cb) = on_mute_change.clone() {
                    cb(host, action_cx, next);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        Some(button)
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        self.build_button(cx)
            .map(ButtonGroupItem::from)
            .unwrap_or_else(|| hidden(cx).into())
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build_button(cx)
            .map(|button| button.into_element(cx))
            .unwrap_or_else(|| hidden(cx))
    }
}

/// Slider aligned with AI Elements `AudioPlayerVolumeRange`.
#[derive(Debug, Clone)]
pub struct AudioPlayerVolumeRange {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Default for AudioPlayerVolumeRange {
    fn default() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default().w_px(Px(96.0)).min_w_0(),
        }
    }
}

impl AudioPlayerVolumeRange {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn build_slider<H: UiHost>(
        _cx: &mut ElementContext<'_, H>,
        controller: AudioPlayerController,
        test_id: Option<Arc<str>>,
        layout: LayoutRefinement,
    ) -> Slider {
        let on_volume_change = controller.on_volume_change.clone();

        let mut slider = Slider::new(controller.volume.clone())
            .range(0.0, 1.0)
            .step(0.01)
            .a11y_label("Volume")
            .refine_layout(layout)
            .on_value_commit(move |host, action_cx, values| {
                let v = clamp_f32(read_single(&values, 1.0), 0.0, 1.0);
                if let Some(cb) = on_volume_change.clone() {
                    cb(host, action_cx, v);
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            });
        if let Some(test_id) = test_id {
            slider = slider.test_id(test_id);
        }
        slider
    }

    fn into_group_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> ButtonGroupItem {
        let Some(controller) = use_audio_player_controller(cx) else {
            return hidden(cx).into();
        };
        let theme = Theme::global(&*cx.app).clone();
        let slider = Self::build_slider(
            cx,
            controller,
            self.test_id,
            LayoutRefinement::default().w_full().min_w_0(),
        )
        .into_element(cx);
        let slider = ui::h_flex(move |_cx| [slider])
            .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .items_center()
            .into_element(cx);

        ButtonGroupText::new_children([slider])
            .refine_layout(self.layout)
            .refine_style(transparent_button_group_text_chrome(&theme))
            .into()
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_audio_player_controller(cx) else {
            return hidden(cx);
        };

        Self::build_slider(cx, controller, self.test_id, self.layout).into_element(cx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPlayerSpeechData {
    pub base64: Arc<str>,
    pub format: Arc<str>,
    pub media_type: Arc<str>,
}

impl AudioPlayerSpeechData {
    pub fn new(
        base64: impl Into<Arc<str>>,
        format: impl Into<Arc<str>>,
        media_type: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            base64: base64.into(),
            format: format.into(),
            media_type: media_type.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AudioPlayerElementSource {
    Remote(Arc<str>),
    SpeechData(AudioPlayerSpeechData),
}

/// Minimal placeholder aligned with AI Elements `AudioPlayerElement`.
///
/// Note: `fret-ui-ai` does not embed an audio backend. Apps are expected to own actual playback
/// and drive the `AudioPlayerController` models. The source builders exist so the public
/// authoring surface can stay aligned with the official AI Elements docs/examples.
#[derive(Debug, Clone, Default)]
pub struct AudioPlayerElement {
    source: Option<AudioPlayerElementSource>,
    test_id: Option<Arc<str>>,
}

impl AudioPlayerElement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn src(mut self, src: impl Into<Arc<str>>) -> Self {
        self.source = Some(AudioPlayerElementSource::Remote(src.into()));
        self
    }

    pub fn speech_data(mut self, data: AudioPlayerSpeechData) -> Self {
        self.source = Some(AudioPlayerElementSource::SpeechData(data));
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let el = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(""),
            style: Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            )),
            color: Some(theme.color_token("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Generic)
                .test_id(test_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};
    use fret_ui_kit::{LengthRefinement, MetricRef};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    fn has_test_id(element: &AnyElement, expected: &str) -> bool {
        element
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.test_id.as_deref())
            .is_some_and(|test_id| test_id == expected)
            || element
                .children
                .iter()
                .any(|child| has_test_id(child, expected))
    }

    #[test]
    fn audio_player_root_provides_controller_to_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                AudioPlayer::new().into_element_with_children(cx, |cx, _controller| {
                    vec![
                        AudioPlayerElement::new()
                            .test_id("audio-play")
                            .into_element(cx),
                    ]
                })
            });

        assert!(has_test_id(&element, "audio-play"));
    }

    #[test]
    fn audio_player_docs_shaped_children_surface_renders_element_and_control_bar() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                AudioPlayer::new()
                    .element(AudioPlayerElement::new().test_id("audio-player-element"))
                    .control_bar(
                        AudioPlayerControlBar::empty()
                            .test_id("audio-player-controls")
                            .play_button(AudioPlayerPlayButton::new().test_id("audio-player-play"))
                            .time_display(AudioPlayerTimeDisplay::new()),
                    )
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "audio-player-element"));
        assert!(has_test_id(&element, "audio-player-controls"));
    }

    #[test]
    fn audio_player_element_docs_shaped_sources_are_stored() {
        let remote = AudioPlayerElement::new().src("https://example.com/audio.mp3");
        assert_eq!(
            remote.source,
            Some(AudioPlayerElementSource::Remote(Arc::<str>::from(
                "https://example.com/audio.mp3"
            )))
        );

        let speech = AudioPlayerElement::new().speech_data(AudioPlayerSpeechData::new(
            "ZGF0YQ==",
            "mp3",
            "audio/mpeg",
        ));
        assert_eq!(
            speech.source,
            Some(AudioPlayerElementSource::SpeechData(
                AudioPlayerSpeechData::new("ZGF0YQ==", "mp3", "audio/mpeg",)
            ))
        );
    }

    #[test]
    fn audio_player_element_uses_shared_sm_typography_preset() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                AudioPlayerElement::new().into_element(cx)
            });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected AudioPlayerElement to render a text placeholder");
        };

        let theme = Theme::global(&app).clone();
        assert_eq!(
            props.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            ))
        );
    }

    #[test]
    fn audio_player_defaults_to_intrinsic_width_layout() {
        let root = AudioPlayer::new();

        assert!(
            root.layout
                .size
                .as_ref()
                .and_then(|size| size.width.as_ref())
                .is_none()
        );
        assert!(matches!(
            root.layout
                .size
                .as_ref()
                .and_then(|size| size.min_width.as_ref()),
            Some(LengthRefinement::Px(MetricRef::Px(px))) if *px == Px(0.0)
        ));
    }

    #[test]
    fn audio_player_control_bar_defaults_to_intrinsic_width_layout() {
        let control_bar = AudioPlayerControlBar::empty();

        assert!(
            control_bar
                .layout
                .size
                .as_ref()
                .and_then(|size| size.width.as_ref())
                .is_none()
        );
        assert!(matches!(
            control_bar
                .layout
                .size
                .as_ref()
                .and_then(|size| size.min_width.as_ref()),
            Some(LengthRefinement::Px(MetricRef::Px(px))) if *px == Px(0.0)
        ));
    }

    #[test]
    fn audio_player_time_range_defaults_to_docs_width_instead_of_flex_fill() {
        let time_range = AudioPlayerTimeRange::new();

        assert!(matches!(
            time_range
                .layout
                .size
                .as_ref()
                .and_then(|size| size.width.as_ref()),
            Some(LengthRefinement::Px(MetricRef::Px(px))) if *px == Px(240.0)
        ));
        assert!(
            time_range
                .layout
                .flex_item
                .as_ref()
                .and_then(|item| item.grow)
                .is_none()
        );
    }
}
