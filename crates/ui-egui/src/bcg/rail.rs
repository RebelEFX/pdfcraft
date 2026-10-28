//! The navy workflow rail: the BCG wordmark, the stages joined by a sage stem, a marker that
//! glides to the current stage, and More tools and Settings at the foot.
//!
//! Motion (the "Buttons and motion" spec): hover lightens the button in 160 ms and fills the
//! icon with sage from the bottom in 240 ms; a press shrinks it to 96%; the marker glides in
//! 260 ms. With animation effects off every change is immediate ([`super::secs`]).

use egui::{Align2, CornerRadius, Id, Rect, Sense, Stroke, Vec2, pos2, vec2};

use super::color::{NAVY, NAVY_2, RAIL_TEXT, SAGE, SAGE_SOFT, WELL};
use super::{Stage, mix, secs};
use crate::{PdfCraftApp, theme};

pub const WIDTH: f32 = 96.0;
const BUTTON: Vec2 = vec2(80.0, 64.0);
const SMALL: Vec2 = vec2(80.0, 50.0);
const GAP: f32 = 6.0;
const DISC: f32 = 38.0;
/// The gap above a stage's disc.
const DISC_TOP: f32 = 7.0;

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    if !super::enabled(&ctx) {
        return;
    }
    // Start has no drawer. When the tools panel opens while Start is still the stage, a document
    // that just opened moves the rail to Edit (as in the design); with no document showing (at
    // launch) the panel stays closed.
    if super::chosen(&ctx) == Stage::Start && app.left_open && app.left == crate::LeftPanel::AllTools {
        if app.active.is_some() || app.combine_showing() {
            super::set_chosen(&ctx, Stage::Edit);
        } else {
            app.left_open = false;
        }
    }
    let current = super::current(app, &ctx);
    let mut picked = None;
    let mut settings = false;
    egui::Panel::left("bcg_rail")
        .exact_size(WIDTH)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(NAVY).inner_margin(egui::Margin { left: 8, right: 8, top: 16, bottom: 12 }))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, GAP);
            ui.vertical_centered(|ui| {
                wordmark(ui);
                ui.add_space(12.0);
            });
            let foot = SMALL.y * 2.0 + GAP * 2.0;
            let list_height = (ui.available_height() - foot).max(BUTTON.y);
            egui::ScrollArea::vertical()
                .id_salt("bcg_rail_scroll")
                .max_height(list_height)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        // The stem joins each stage to the next, in the gaps between a label
                        // and the next disc, so it never runs through the text.
                        let stem = ui.painter().add(egui::Shape::Noop);
                        let mut rects: Vec<Rect> = Vec::new();
                        let mut marker = None;
                        for stage in Stage::RAIL {
                            let spec = stage.spec();
                            let on = current == Some(stage);
                            let resp = button(ui, spec.icon, spec.label, &accessible_name(stage), on, false);
                            rects.push(resp.rect);
                            if on {
                                marker = Some(resp.rect);
                                // A short window scrolls the stages: bring a newly current one
                                // into view (a tool panel opened by a command can move it).
                                let seen = Id::new("bcg_rail_seen");
                                if ctx.data(|d| d.get_temp::<Stage>(seen)) != Some(stage) {
                                    ctx.data_mut(|d| d.insert_temp(seen, stage));
                                    ui.scroll_to_rect(resp.rect, None);
                                }
                            }
                            if resp.clicked() {
                                picked = Some(stage);
                            }
                        }
                        let stroke = Stroke::new(2.0, SAGE.gamma_multiply(0.3));
                        let segments: Vec<egui::Shape> = rects
                            .windows(2)
                            .filter_map(|w| match w {
                                [a, b] => {
                                    let top = pos2(a.center().x, a.bottom() - 1.0);
                                    let bottom = pos2(a.center().x, b.top() + DISC_TOP + 1.0);
                                    (bottom.y > top.y).then(|| egui::Shape::line_segment([top, bottom], stroke))
                                }
                                _ => None,
                            })
                            .collect();
                        ui.painter().set(stem, egui::Shape::Vec(segments));
                        paint_marker(ui, marker);
                    });
                });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if button(ui, "settings", "Settings", "App settings", false, true).clicked() {
                    settings = true;
                }
                let more_on = current == Some(Stage::More);
                if button(ui, Stage::More.spec().icon, "More tools", "More tools", more_on, true).clicked() {
                    picked = Some(Stage::More);
                }
            });
        });
    if let Some(stage) = picked {
        super::go(app, &ctx, stage);
    }
    if settings {
        app.run_command("app.preferences");
    }
}

fn wordmark(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(WIDTH - 16.0, 46.0), Sense::hover());
    let p = ui.painter();
    p.text(rect.center_top() + vec2(0.0, 2.0), Align2::CENTER_TOP, "BCG", super::start::display(25.0), egui::Color32::WHITE);
    p.text(rect.center_bottom(), Align2::CENTER_BOTTOM, "P  D  F", theme::semibold(10.5), SAGE_SOFT);
}

fn disc_center(rect: Rect, small: bool) -> egui::Pos2 {
    let size = if small { 30.0 } else { DISC };
    pos2(rect.center().x, rect.top() + (if small { 6.0 } else { DISC_TOP }) + size / 2.0)
}

/// The sage marker at the rail's edge, gliding to the current stage. Hidden when no stage is
/// current.
fn paint_marker(ui: &egui::Ui, target: Option<Rect>) {
    let ctx = ui.ctx();
    let id = Id::new("bcg_rail_marker");
    let shown = ctx.animate_bool_with_time(id.with("shown"), target.is_some(), secs(ctx, 0.16));
    let Some(rect) = target.or_else(|| ctx.data(|d| d.get_temp::<Rect>(id))) else { return };
    ctx.data_mut(|d| d.insert_temp(id, rect));
    let y = ctx.animate_value_with_time(id.with("y"), rect.center().y, secs(ctx, 0.26));
    if shown <= 0.01 {
        return;
    }
    let x = ui.max_rect().left() - 8.0;
    let bar = Rect::from_center_size(pos2(x + 2.0, y), vec2(4.0, 40.0));
    ui.painter().with_clip_rect(ui.clip_rect().expand2(vec2(10.0, 0.0))).rect_filled(
        bar,
        CornerRadius { nw: 0, sw: 0, ne: 4, se: 4 },
        SAGE.gamma_multiply(shown),
    );
}

/// What assistive technology (and tests) call a stage's rail button: "Edit tools", "Start
/// screen". The short visible labels ("Edit", "Start") would clash with menus and dialogs.
pub fn accessible_name(stage: Stage) -> String {
    match stage {
        Stage::Start => "Start screen".to_owned(),
        Stage::More => "More tools".to_owned(),
        _ => format!("{} tools", stage.spec().label),
    }
}

/// One rail button: a disc with the icon, and the label under it.
fn button(ui: &mut egui::Ui, icon: &str, label: &str, name: &str, on: bool, small: bool) -> egui::Response {
    let size = if small { SMALL } else { BUTTON };
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, name));
    let resp = theme::hand(resp);
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let ctx = ui.ctx();
    let hot = resp.hovered() || resp.is_pointer_button_down_on();
    let hover = ctx.animate_bool_with_time(resp.id.with("bcg-h"), hot, secs(ctx, 0.16));
    let wipe = ctx.animate_bool_with_time(resp.id.with("bcg-w"), hot && !on, secs(ctx, 0.24));
    let down = ctx.animate_bool_with_time(resp.id.with("bcg-d"), resp.is_pointer_button_down_on(), secs(ctx, 0.09));
    let sel = ctx.animate_bool_with_time(resp.id.with("bcg-on"), on, secs(ctx, 0.2));

    // A press shrinks the whole button to 96%.
    let body = Rect::from_center_size(rect.center(), rect.size() * (1.0 - 0.04 * down));
    let p = ui.painter();
    let wash = NAVY_2.gamma_multiply(hover.max(down));
    if wash.a() > 0 {
        p.rect_filled(body, CornerRadius::same(10), wash);
    }

    let disc_size = if small { 30.0 } else { DISC };
    let center = disc_center(body, small);
    if !small {
        if sel > 0.01 {
            p.circle_filled(center, disc_size / 2.0 + 4.0 * sel, SAGE.gamma_multiply(0.18 * sel));
        }
        p.circle_filled(center, disc_size / 2.0, mix(WELL, SAGE, sel));
    } else if sel > 0.01 {
        p.circle_filled(center, disc_size / 2.0, SAGE.gamma_multiply(sel));
    }
    let icon_rect = Rect::from_center_size(center, Vec2::splat(20.0));
    // At rest the icon is soft white; on the current stage it is navy on the sage disc. The
    // small foot buttons have no disc at rest, so hover brightens their icon instead.
    let rest = if small { mix(RAIL_TEXT, egui::Color32::WHITE, hover) } else { RAIL_TEXT };
    let base = mix(rest, NAVY, sel);
    crate::icons::paint(ui, icon_rect, icon, 20.0, base);
    // Hover: the icon fills with sage from the bottom up.
    if wipe > 0.01 {
        let clip = Rect::from_min_max(pos2(icon_rect.left() - 2.0, icon_rect.bottom() - icon_rect.height() * wipe), icon_rect.max + vec2(2.0, 2.0));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(icon_rect.expand(2.0)));
        child.set_clip_rect(clip.intersect(ui.clip_rect()));
        crate::icons::paint(&child, icon_rect, icon, 20.0, SAGE_SOFT);
    }

    let text = mix(RAIL_TEXT, egui::Color32::WHITE, hover.max(sel));
    let label_y = if small { body.bottom() - 7.0 } else { body.bottom() - 6.0 };
    let font = theme::semibold(if small { 11.0 } else { 11.5 });
    ui.painter().text(pos2(body.center().x, label_y), Align2::CENTER_BOTTOM, label, font, text);

    if resp.has_focus() {
        ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(12), Stroke::new(2.0, SAGE_SOFT), egui::StrokeKind::Outside);
    }
    resp
}
