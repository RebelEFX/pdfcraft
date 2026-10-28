//! The BCG interface faces (`DESIGN.md`): Mulish for everything, Yeseva One for the wordmark and
//! the Start heading. Both are SIL OFL 1.1 Google fonts, an optional build input (`BCG_FONTS_DIR`,
//! see `build.rs`) because font files are never committed to this repository (AGENTS.md §1.4).
//! Without them the app keeps Inter, and the display family falls back to the semibold stack.

use egui::epaint::text::VariationCoords;
use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

include!(concat!(env!("OUT_DIR"), "/bcg_fonts.rs"));

/// The display family's name (always defined, so using it never panics).
pub const DISPLAY: &str = "bcg-display";

/// Whether the BCG faces are built in.
pub fn built_in() -> bool {
    MULISH.is_some()
}

/// Put Mulish first in the regular, medium and semibold stacks (Inter and the rest stay as
/// fallbacks for the scripts Mulish lacks), and define the display family.
pub fn register(fonts: &mut FontDefinitions) {
    if let Some(bytes) = MULISH {
        // Mulish's 600 reads light at small sizes, so "semibold" is 700 (DESIGN.md).
        for (name, weight, family) in [
            ("Mulish-400", 400.0, FontFamily::Proportional),
            ("Mulish-600", 600.0, FontFamily::Name("medium".into())),
            ("Mulish-700", 700.0, FontFamily::Name("semibold".into())),
        ] {
            let mut data = FontData::from_static(bytes);
            data.tweak.coords = VariationCoords::new([("wght", weight)]);
            fonts.font_data.insert(name.to_owned(), Arc::new(data));
            fonts.families.entry(family).or_default().insert(0, name.to_owned());
        }
    }
    let mut display = Vec::new();
    if let Some(bytes) = YESEVA_ONE {
        fonts.font_data.insert("YesevaOne".to_owned(), Arc::new(FontData::from_static(bytes)));
        display.push("YesevaOne".to_owned());
    }
    display.extend(fonts.families.get(&FontFamily::Name("semibold".into())).cloned().unwrap_or_default());
    fonts.families.insert(FontFamily::Name(DISPLAY.into()), display);
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_display_family_always_exists() {
        let defs = crate::theme::font_definitions();
        let display = defs.families.get(&egui::FontFamily::Name(super::DISPLAY.into())).expect("display family");
        assert!(!display.is_empty());
    }
}
