//! Cookbook examples crate.
//!
//! This crate intentionally keeps a tiny surface:
//! - helpers shared by `examples/`,
//! - no reusable product APIs (those belong in ecosystem crates).

use fret::prelude::*;

pub fn install_cookbook_defaults(app: &mut App) {
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
