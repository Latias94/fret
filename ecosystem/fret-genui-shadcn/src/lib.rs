//! GenUI catalog and component resolver backed by `fret-ui-shadcn`.
//!
//! This crate is intended for tooling and experiments that need to map from a declarative
//! component description (a "catalog") to concrete Fret element construction.

pub mod catalog;
pub mod resolver;

#[cfg(test)]
mod surface_policy_tests {
    const RESOLVER_SOURCES: &[(&str, &str)] = &[
        ("resolver/basic.rs", include_str!("resolver/basic.rs")),
        ("resolver/choice.rs", include_str!("resolver/choice.rs")),
        ("resolver/compat.rs", include_str!("resolver/compat.rs")),
        ("resolver/compound.rs", include_str!("resolver/compound.rs")),
        ("resolver/data.rs", include_str!("resolver/data.rs")),
        ("resolver/feedback.rs", include_str!("resolver/feedback.rs")),
        ("resolver/forms.rs", include_str!("resolver/forms.rs")),
        ("resolver/helpers.rs", include_str!("resolver/helpers.rs")),
        ("resolver/mod.rs", include_str!("resolver/mod.rs")),
        (
            "resolver/navigation.rs",
            include_str!("resolver/navigation.rs"),
        ),
        ("resolver/numeric.rs", include_str!("resolver/numeric.rs")),
        ("resolver/overlay.rs", include_str!("resolver/overlay.rs")),
    ];

    #[test]
    fn resolver_uses_curated_shadcn_facade_not_raw_modules() {
        for (label, source) in RESOLVER_SOURCES {
            assert!(
                !source.contains("shadcn::raw::"),
                "{label} should use the curated shadcn facade, not raw shadcn modules"
            );
        }

        for label in [
            "resolver/basic.rs",
            "resolver/choice.rs",
            "resolver/compat.rs",
            "resolver/compound.rs",
            "resolver/data.rs",
            "resolver/feedback.rs",
            "resolver/forms.rs",
            "resolver/helpers.rs",
            "resolver/navigation.rs",
            "resolver/numeric.rs",
            "resolver/overlay.rs",
        ] {
            let source = RESOLVER_SOURCES
                .iter()
                .find_map(|(path, source)| (*path == label).then_some(*source))
                .expect("resolver source should be listed");
            assert!(
                source.contains("use fret_ui_shadcn::facade as shadcn;"),
                "{label} should import the curated shadcn facade"
            );
        }
    }
}
