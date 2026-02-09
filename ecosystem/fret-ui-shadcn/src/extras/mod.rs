//! Shadcn-styled "extras" blocks/recipes that are intentionally out of scope for shadcn/ui v4
//! taxonomy parity.
//!
//! Design rules:
//! - This is a module surface (`fret_ui_shadcn::extras::*`), not a new crate or taxonomy.
//! - `fret-ui-shadcn` crate root must not glob re-export these items; keep autocomplete aligned with
//!   the v4 surface.
//! - Extras should not require expanding `crates/fret-ui` public contracts (ADR 0066).
//! - Prefer deterministic regression gates: snapshots first, scripted `fretboard diag` for
//!   interaction-heavy blocks.

pub mod announcement;
pub mod avatar_stack;
pub mod banner;
pub mod rating;
pub mod relative_time;
pub mod tags;

pub use announcement::{Announcement, AnnouncementTag, AnnouncementTitle};
pub use avatar_stack::{AvatarStack, AvatarStackItem};
pub use banner::{Banner, BannerAction, BannerClose, BannerIcon, BannerTitle};
pub use rating::Rating;
pub use relative_time::{
    RelativeTime, RelativeTimeClockZone, RelativeTimeTick, RelativeTimeZone, RelativeTimeZoneDate,
    RelativeTimeZoneDisplay, RelativeTimeZoneLabel,
};
pub use tags::{Tag, Tags};
