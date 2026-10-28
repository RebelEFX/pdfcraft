//! BCG PDF build: the workflow rail, its tool drawers and the Start screen
//! (`claude/bcg-pdf-redesign-plan.md` in the BCG Portal project; the "BCG PDF redesign" canvas).
//!
//! Everything BCG-specific lives here, so PdfCraft updates merge cleanly. The shell is off until
//! an app turns it on with [`enable`] (both BCG PDF apps do, `apps/pdfcraft` and
//! `apps/pdfcraft-web`), so upstream's own tests keep driving upstream's interface unchanged and
//! `tests/ui_group/bcg.rs` covers this one. Upstream files only call in at a few hook points:
//! - `lib.rs`: [`rail::show`] before the tab strip (the rail is full height on the left).
//! - `panels::left_panel`: [`drawer::show`] in place of the All tools list.
//! - `home::show`: [`start::hero`] in place of the welcome and recommended-tools cards.
//! - `chrome::mode_bar`: [`mode_tabs`] hides the mode tabs, which the rail replaces.
//! - `theme::font_definitions_for_platform`: [`fonts::register`] adds the BCG faces (only when
//!   the build embeds them, `BCG_FONTS_DIR`).
//! - `theme::Tokens::for_kind`: the light palette is BCG's (always, shell on or off).
//!
//! The rail drives the app's existing state (`left`, `left_open`, `mode`, the page grid) and
//! runs registered commands; it adds no tool behaviour of its own. Tool panels with live
//! controls (Fill & Sign, stamps, measuring, forms, redaction, text formatting) are still
//! upstream's: commands open them, and their Back button returns to the drawer.

pub mod drawer;
pub mod fonts;
pub mod rail;
pub mod stages;
pub mod start;

use egui::{Color32, Context, Id};

use crate::{LeftPanel, PdfCraftApp};
pub use stages::Stage;

/// BCG palette (`DESIGN.md`, read off bcgreenhouses.com). The rail is always navy; drawers and
/// the Start screen use the theme tokens, whose light values carry the same palette.
pub mod color {
    use egui::Color32;
    pub const NAVY: Color32 = Color32::from_rgb(0x14, 0x30, 0x40);
    pub const NAVY_2: Color32 = Color32::from_rgb(0x1E, 0x42, 0x54);
    /// The rail's icon discs: between navy and navy-2, opaque so the stem passes behind them.
    pub const WELL: Color32 = Color32::from_rgb(0x1B, 0x3A, 0x4C);
    pub const SAGE: Color32 = Color32::from_rgb(0x81, 0xA9, 0xA7);
    pub const SAGE_SOFT: Color32 = Color32::from_rgb(0xB8, 0xD1, 0xCF);
    /// Rail labels at rest: white at 74%.
    pub const RAIL_TEXT: Color32 = Color32::from_rgba_premultiplied(189, 189, 189, 189);
}

fn enabled_id() -> Id {
    Id::new("bcg-shell")
}

/// Turn the BCG shell on for this window: the rail, the drawers and the Start screen. Call it once
/// the app's saved settings are restored. A Tools panel still at upstream's default width starts
/// 304 points wide instead, so the drawers' one-line hints fit; a width the user chose is kept.
pub fn enable(app: &mut PdfCraftApp, ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp(enabled_id(), true));
    widen_default_tools(&mut app.docking);
}

/// Whether [`enable`] turned the shell on for this window.
pub fn enabled(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(enabled_id())).unwrap_or(false)
}

/// Upstream's default Tools width, and the drawers' (`DESIGN.md` 304).
const UPSTREAM_TOOLS_WIDTH: f64 = 272.0;
const TOOLS_WIDTH: f64 = 304.0;

/// Widen the Tools panel only while the layout is exactly upstream's default: the layout type
/// lives in `craft_docking`, so the default is compared and edited through its serialized form.
fn widen_default_tools(ws: &mut crate::docking::Workspace) {
    let (Ok(current), Ok(default)) = (serde_json::to_value(&*ws), serde_json::to_value(crate::docking::Workspace::default())) else {
        return;
    };
    if current != default {
        return;
    }
    let mut wider = default;
    if replace_number(&mut wider, UPSTREAM_TOOLS_WIDTH, TOOLS_WIDTH, 0) == 1
        && let Ok(w) = serde_json::from_value(wider)
    {
        *ws = w;
    }
}

/// Replace every number equal to `from` with `to` (at most 64 levels deep), returning how many.
fn replace_number(v: &mut serde_json::Value, from: f64, to: f64, depth: usize) -> usize {
    if depth > 64 {
        return 0;
    }
    match v {
        serde_json::Value::Number(n) if n.as_f64() == Some(from) => match serde_json::Number::from_f64(to) {
            Some(to) => {
                *n = to;
                1
            }
            None => 0,
        },
        serde_json::Value::Array(items) => items.iter_mut().map(|i| replace_number(i, from, to, depth + 1)).sum(),
        serde_json::Value::Object(map) => map.values_mut().map(|i| replace_number(i, from, to, depth + 1)).sum(),
        _ => 0,
    }
}

/// The mode tabs to show: none with the shell on (the rail replaces them), all of them otherwise.
/// `chrome::mode_bar` iterates this instead of its list.
pub fn mode_tabs<T, const N: usize>(ctx: &Context, tabs: [T; N]) -> impl Iterator<Item = T> + use<T, N> {
    let on = enabled(ctx);
    tabs.into_iter().filter(move |_| !on)
}

fn stage_id() -> Id {
    Id::new("bcg-stage")
}

/// The stage last chosen on the rail (Start until one is).
pub fn chosen(ctx: &Context) -> Stage {
    ctx.data(|d| d.get_temp::<Stage>(stage_id())).unwrap_or(Stage::Start)
}

pub(crate) fn set_chosen(ctx: &Context, stage: Stage) {
    ctx.data_mut(|d| d.insert_temp(stage_id(), stage));
}

/// The stage the rail shows as current, or `None` when none applies (a document is open with
/// its drawer closed). A command that opened one of upstream's tool panels moves the rail with it.
pub fn current(app: &PdfCraftApp, ctx: &Context) -> Option<Stage> {
    let doc_showing = app.active.is_some() || app.combine_showing();
    if app.left_open {
        if app.active.is_some() && crate::organize_panel::showing(app) {
            return Some(Stage::Pages);
        }
        if let LeftPanel::Tool(group) = app.left
            && let Some(stage) = Stage::for_group(group)
        {
            return Some(stage);
        }
        let stage = chosen(ctx);
        if stage != Stage::Start || !doc_showing {
            return Some(stage);
        }
        return None;
    }
    if doc_showing { None } else { Some(Stage::Start) }
}

/// Move to `stage`, as a click on the rail does. Picking the current stage again closes its
/// drawer.
pub fn go(app: &mut PdfCraftApp, ctx: &Context, stage: Stage) {
    let again = current(app, ctx) == Some(stage) && app.left_open && app.left == LeftPanel::AllTools;
    set_chosen(ctx, stage);
    // Leaving Edit puts the editing tools down, as leaving upstream's Edit mode does.
    if stage != Stage::Edit && matches!(app.quick_tool, crate::QuickTool::EditText | crate::QuickTool::AddText) {
        app.put_down_edit_text();
    }
    if stage == Stage::Start {
        app.active = None;
        app.combine_tab.focused = false;
        app.left_open = false;
        return;
    }
    if again {
        app.left_open = false;
        return;
    }
    if app.mode == crate::Mode::Read {
        app.switch_mode(crate::Mode::AllTools, false);
    }
    let organize_open = app.active.is_some() && crate::organize_panel::showing(app);
    if stage == Stage::Pages && app.active.is_some() {
        app.select_tool("organize");
        return;
    }
    if organize_open && let Some(view) = app.active.and_then(|i| app.views.get_mut(i)) {
        view.organize = false;
    }
    app.left = LeftPanel::AllTools;
    app.reveal_tools();
}

/// Whether the left panel's All tools slot shows a BCG drawer (every stage but More tools, with
/// the shell on).
pub fn drawer_wanted(ctx: &Context) -> bool {
    enabled(ctx) && chosen(ctx) != Stage::More
}

/// Seconds for an animation, or none with animation effects off (`motion::animated`).
pub fn secs(ctx: &Context, s: f32) -> f32 {
    if crate::motion::animated(ctx) { s } else { 0.0 }
}

/// Progress 0 to 1 of an entrance that restarts whenever `key` changes for `id`, eased out.
/// With animation effects off it is always 1.
pub fn entrance(ctx: &Context, id: Id, key: u64, duration: f32) -> f32 {
    if !crate::motion::animated(ctx) || duration <= 0.0 {
        return 1.0;
    }
    let now = ctx.input(|i| i.time);
    let start = ctx.data_mut(|d| {
        let entry = d.get_temp_mut_or_insert_with(id, || (key, now));
        if entry.0 != key {
            *entry = (key, now);
        }
        entry.1
    });
    let p = (((now - start) as f32) / duration).clamp(0.0, 1.0);
    if p < 1.0 {
        ctx.request_repaint();
    }
    egui::emath::easing::cubic_out(p)
}

/// `a` towards `b` by `t` (0 to 1), in gamma space like the rest of the app's washes.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_tabs_hide_only_with_the_shell_on() {
        let ctx = Context::default();
        assert_eq!(mode_tabs(&ctx, [1, 2, 3]).count(), 3);
        assert!(!drawer_wanted(&ctx));
        let mut app = PdfCraftApp::new();
        enable(&mut app, &ctx);
        assert!(enabled(&ctx));
        assert_eq!(mode_tabs(&ctx, [1, 2, 3]).count(), 0);
        assert!(drawer_wanted(&ctx));
    }

    fn widths(ws: &crate::docking::Workspace) -> (usize, usize) {
        let mut v = serde_json::to_value(ws).unwrap_or_default();
        let old = replace_number(&mut v.clone(), UPSTREAM_TOOLS_WIDTH, 0.0, 0);
        (old, replace_number(&mut v, TOOLS_WIDTH, 0.0, 0))
    }

    #[test]
    fn the_default_tools_panel_widens_and_a_chosen_width_stays() {
        let mut ws = crate::docking::Workspace::default();
        assert_eq!(widths(&ws), (1, 0), "upstream's default holds one 272");
        widen_default_tools(&mut ws);
        assert_eq!(widths(&ws), (0, 1), "and becomes 304");
        assert!(ws.validate().is_ok());
        // Anything but the exact default is the user's: right-side tools stay as they are.
        let mut moved = crate::docking::Workspace::default();
        moved.set_tools_on_right(true).unwrap();
        let before = serde_json::to_value(&moved).unwrap();
        widen_default_tools(&mut moved);
        assert_eq!(serde_json::to_value(&moved).unwrap(), before);
    }

    #[test]
    fn replace_number_stops_at_its_depth_limit() {
        let mut v = serde_json::json!(272.0);
        for _ in 0..80 {
            v = serde_json::json!([v]);
        }
        assert_eq!(replace_number(&mut v, 272.0, 304.0, 0), 0);
    }
}
