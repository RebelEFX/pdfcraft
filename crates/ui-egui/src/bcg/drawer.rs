//! The tool drawer beside the rail: the stage's heading, one navy main action, its tools as
//! buttons with a line on what each does, format cards for Export, and the tools PdfCraft
//! hasn't finished under "Coming later".
//!
//! BCG wording is English only, so these strings deliberately avoid the `tl!` macro: the
//! catalogues' coverage tests scan the source for its literals.

use egui::{Align2, CornerRadius, Id, Rect, Sense, Stroke, StrokeKind, vec2};

use super::color::SAGE;
use super::stages::{Action, Entry, Stage};
use super::{mix, secs};
use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, widgets};

const ROW_H: f32 = 52.0;
const ICON_BOX: f32 = 36.0;

/// Whether an entry can run now, and why not when it can't.
fn usable(app: &PdfCraftApp, action: Action) -> Result<(), &'static str> {
    let needs_doc = || if app.active.is_none() { "Open a PDF first" } else { "Not available for this document" };
    match action {
        Action::Tool(_) => Ok(()),
        Action::Command(c) => match pdfcraft_engine::commands::command(c) {
            Some(spec) if app.command_enabled(spec) => Ok(()),
            _ => Err(needs_doc()),
        },
        // Add stays usable with no document open: it asks for a file first (as upstream's menu).
        Action::Menu(_) => Ok(()),
    }
}

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui, t: &Tokens) {
    let stage = super::chosen(ui.ctx());
    let spec = stage.spec();
    let ctx = ui.ctx().clone();
    // New tools slide 10 px out from the rail and fade in, each time the stage changes.
    let p = super::entrance(&ctx, Id::new("bcg_drawer_in"), stage as u64, 0.24);
    let mut run: Option<Action> = None;
    let mut close = false;
    let rect = ui.max_rect().translate(vec2(-10.0 * (1.0 - p), 0.0));
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.multiply_opacity(p);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::icons::button(ui, "x", 26.0, false, "Close tools").clicked() {
                    close = true;
                }
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(spec.title).font(theme::semibold(19.0)).color(t.text)).truncate());
                });
            });
        });
        if !spec.blurb.is_empty() {
            ui.add(egui::Label::new(egui::RichText::new(spec.blurb).font(theme::regular(13.0)).color(t.text_muted)).wrap());
        }
        ui.add_space(12.0);
        egui::ScrollArea::vertical().id_salt(("bcg_drawer", stage as u64)).auto_shrink([false, false]).show(ui, |ui| {
            if let Some(primary) = &spec.primary
                && main_action(ui, t, primary, usable(app, primary.action)).clicked()
            {
                run = Some(primary.action);
            }
            let mut later: Vec<&Entry> = Vec::new();
            let ready_formats: Vec<&Entry> = spec.formats.iter().filter(|f| f.action.ready()).collect();
            later.extend(spec.formats.iter().filter(|f| !f.action.ready()));
            if !ready_formats.is_empty() {
                widgets::section_title(ui, "Save a copy as");
                if let Some(a) = format_grid(app, ui, t, &ready_formats) {
                    run = Some(a);
                }
            }
            for section in spec.sections {
                let ready: Vec<&Entry> = section.entries.iter().filter(|e| e.action.ready()).collect();
                later.extend(section.entries.iter().filter(|e| !e.action.ready()));
                if ready.is_empty() {
                    continue;
                }
                widgets::section_title(ui, section.title);
                for entry in ready {
                    if let Some(a) = row(app, ui, t, entry) {
                        run = Some(a);
                    }
                }
            }
            if !later.is_empty() {
                widgets::section_title(ui, "Coming later");
                for entry in later {
                    planned_row(ui, t, entry);
                }
            }
            ui.add_space(12.0);
        });
    });
    if close {
        app.left_open = false;
    }
    match run {
        Some(Action::Command(c)) => app.run_command(c),
        Some(Action::Tool(group)) => app.select_tool(group),
        Some(Action::Menu(_)) | None => {}
    }
}

/// Hover, press and focus amounts for a drawer control, motion-aware.
struct Feel {
    hover: f32,
    down: f32,
}

fn feel(ui: &egui::Ui, resp: &egui::Response) -> Feel {
    let ctx = ui.ctx();
    let hot = resp.hovered() || resp.is_pointer_button_down_on();
    Feel {
        hover: ctx.animate_bool_with_time(resp.id.with("bcg-h"), hot && ui.is_enabled(), secs(ctx, 0.16)),
        down: ctx.animate_bool_with_time(resp.id.with("bcg-d"), resp.is_pointer_button_down_on(), secs(ctx, 0.09)),
    }
}

fn focus_ring(ui: &egui::Ui, t: &Tokens, resp: &egui::Response, rect: Rect, radius: u8) {
    if resp.has_focus() {
        ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(radius + 2), Stroke::new(2.0, t.accent_text), StrokeKind::Outside);
    }
}

/// The one navy main action of a drawer.
fn main_action(ui: &mut egui::Ui, t: &Tokens, entry: &Entry, usable: Result<(), &'static str>) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
    let enabled = usable.is_ok();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, entry.label));
    let resp = match usable {
        Ok(()) => theme::hand(resp),
        Err(why) => resp.on_hover_text(why),
    };
    let f = feel(ui, &resp);
    let body = Rect::from_center_size(rect.center(), rect.size() * (1.0 - 0.02 * f.down));
    let p = ui.painter();
    if enabled {
        if f.hover > 0.01 {
            let shadow = egui::Shadow { offset: [0, 8], blur: 18, spread: 0, color: super::color::NAVY.gamma_multiply(0.35 * f.hover) };
            p.add(shadow.as_shape(body, CornerRadius::same(8)));
        }
        p.rect_filled(body, CornerRadius::same(8), mix(t.accent, super::color::NAVY_2, f.hover));
    } else {
        p.rect_filled(body, CornerRadius::same(8), t.pressed);
    }
    let fg = if enabled { egui::Color32::WHITE } else { t.text_faint };
    let font = theme::semibold(14.0);
    let w = widgets::text_width(ui, entry.label, &font);
    let start = body.center().x - (w + 30.0) / 2.0;
    crate::icons::paint(ui, Rect::from_center_size(egui::pos2(start + 10.0, body.center().y), vec2(20.0, 20.0)), entry.icon, 19.0, fg);
    ui.painter().text(egui::pos2(start + 30.0, body.center().y), Align2::LEFT_CENTER, entry.label, font, fg);
    focus_ring(ui, t, &resp, rect, 8);
    ui.add_space(4.0);
    resp
}

/// Paint `text` on one line, cut with "…" to `max_width`.
fn one_line(ui: &egui::Ui, text: &str, font: egui::FontId, color: egui::Color32, left_center: egui::Pos2, max_width: f32) -> bool {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: max_width.max(1.0), max_rows: 1, break_anywhere: true, ..Default::default() };
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let elided = galley.elided;
    ui.painter().galley(left_center - vec2(0.0, galley.size().y / 2.0), galley, color);
    elided
}

/// A tool button: icon on a sage tint, name, and one line on what it does. Hover tints the row
/// and turns the icon square dark sage, lifting the icon a pixel.
fn row(app: &PdfCraftApp, ui: &mut egui::Ui, t: &Tokens, entry: &'static Entry) -> Option<Action> {
    let usable = usable(app, entry.action);
    let enabled = usable.is_ok();
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, entry.label));
    let resp = match usable {
        Ok(()) => theme::hand(resp),
        Err(why) => resp.on_hover_text(why),
    };
    let on = matches!(entry.action, Action::Tool(g) if app.left_open && app.left == crate::LeftPanel::Tool(g));
    let f = feel(ui, &resp);
    let body = Rect::from_center_size(rect.center(), rect.size() * (1.0 - 0.015 * f.down));
    let p = ui.painter();
    let fill = if on { t.accent_soft } else { t.hover.gamma_multiply(f.hover) };
    let border = if on { SAGE } else { t.border.gamma_multiply(f.hover) };
    if fill.a() > 0 || border.a() > 0 {
        p.rect(body, CornerRadius::same(8), fill, Stroke::new(1.0, border), StrokeKind::Inside);
    }
    let lit = if on { 1.0 } else { f.hover };
    let icon_box = Rect::from_min_size(body.min + vec2(8.0, (ROW_H - ICON_BOX) / 2.0), vec2(ICON_BOX, ICON_BOX));
    let (box_fill, icon_color, text, hint) = if enabled {
        (mix(t.accent_soft, t.accent_text, lit), mix(t.accent_text, egui::Color32::WHITE, lit), t.text, t.text_muted)
    } else {
        (t.pressed, t.text_faint, t.text_faint, t.text_faint)
    };
    p.rect_filled(icon_box, CornerRadius::same(8), box_fill);
    let lift = if enabled { lit } else { 0.0 };
    crate::icons::paint(ui, icon_box.translate(vec2(0.0, -lift)), entry.icon, 19.0, icon_color);
    let x = icon_box.right() + 12.0;
    let width = body.right() - x - 8.0;
    let elided_name = one_line(ui, entry.label, theme::semibold(14.0), text, egui::pos2(x, body.center().y - 9.0), width);
    let elided_hint = one_line(ui, entry.hint, theme::regular(12.5), hint, egui::pos2(x, body.center().y + 10.0), width);
    focus_ring(ui, t, &resp, rect, 8);
    let resp = if enabled && (elided_name || elided_hint) { resp.on_hover_text(format!("{}\n{}", entry.label, entry.hint)) } else { resp };
    if let Action::Menu(actions) = entry.action {
        let mut chosen = None;
        egui::Popup::menu(&resp).show(|ui| {
            ui.set_min_width(180.0);
            for (n, id) in actions.iter().enumerate() {
                let Some(spec) = pdfcraft_engine::commands::command(id) else { continue };
                let enabled = (n == 0 && app.active.is_none()) || app.command_enabled(spec);
                let label = crate::i18n::menu_label(spec.id, spec.label);
                if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                    chosen = Some(spec.id);
                    ui.close();
                }
            }
        });
        return chosen.map(Action::Command);
    }
    (enabled && resp.clicked()).then_some(entry.action)
}

/// A tool PdfCraft hasn't finished: grey, says so, and does nothing.
fn planned_row(ui: &mut egui::Ui, t: &Tokens, entry: &Entry) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, false, entry.label));
    let _ = resp.on_hover_text("Not available yet");
    let p = ui.painter();
    let icon_box = Rect::from_min_size(rect.min + vec2(8.0, (ROW_H - ICON_BOX) / 2.0), vec2(ICON_BOX, ICON_BOX));
    p.rect_filled(icon_box, CornerRadius::same(8), t.pressed);
    crate::icons::paint(ui, icon_box, entry.icon, 19.0, t.text_faint);
    let soon_font = theme::semibold(11.5);
    let soon_w = widgets::text_width(ui, "Not yet", &soon_font);
    ui.painter().text(rect.right_center() - vec2(8.0, 0.0), Align2::RIGHT_CENTER, "Not yet", soon_font, t.text_faint);
    let x = icon_box.right() + 12.0;
    let width = rect.right() - x - soon_w - 18.0;
    one_line(ui, entry.label, theme::semibold(14.0), t.text_faint, egui::pos2(x, rect.center().y - 9.0), width);
    one_line(ui, entry.hint, theme::regular(12.5), t.text_faint, egui::pos2(x, rect.center().y + 10.0), width);
}

/// Export's format cards, two to a row. Hover turns the border sage and adds one soft shadow.
fn format_grid(app: &PdfCraftApp, ui: &mut egui::Ui, t: &Tokens, formats: &[&'static Entry]) -> Option<Action> {
    let mut run = None;
    let gap = 8.0;
    let w = ((ui.available_width() - gap) / 2.0).floor().max(60.0);
    for pair in formats.chunks(2) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for entry in pair {
                let usable = usable(app, entry.action);
                let enabled = usable.is_ok();
                let (rect, resp) = ui.allocate_exact_size(vec2(w, 88.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, format!("{} {}", entry.label, entry.hint)));
                let resp = match usable {
                    Ok(()) => theme::hand(resp),
                    Err(why) => resp.on_hover_text(why),
                };
                let f = feel(ui, &resp);
                let body = Rect::from_center_size(rect.center(), rect.size() * (1.0 - 0.02 * f.down));
                let p = ui.painter();
                if f.hover > 0.01 {
                    let shadow = egui::Shadow { offset: [0, 8], blur: 18, spread: 0, color: super::color::NAVY.gamma_multiply(0.22 * f.hover) };
                    p.add(shadow.as_shape(body, CornerRadius::same(10)));
                }
                p.rect(body, CornerRadius::same(10), t.card, Stroke::new(1.0, mix(t.border, SAGE, f.hover)), StrokeKind::Inside);
                let (ink, sub) = if enabled { (t.text, t.text_muted) } else { (t.text_faint, t.text_faint) };
                crate::icons::paint(
                    ui,
                    Rect::from_min_size(body.min + vec2(12.0, 12.0), vec2(24.0, 24.0)),
                    entry.icon,
                    23.0,
                    if enabled { t.accent_text } else { t.text_faint },
                );
                one_line(ui, entry.label, theme::semibold(14.0), ink, body.left_top() + vec2(12.0, 54.0), body.width() - 20.0);
                one_line(ui, entry.hint, theme::regular(12.0), sub, body.left_top() + vec2(12.0, 72.0), body.width() - 20.0);
                focus_ring(ui, t, &resp, rect, 10);
                if enabled && resp.clicked() {
                    run = Some(entry.action);
                }
            }
        });
        ui.add_space((gap - ui.spacing().item_spacing.y).max(0.0));
    }
    run
}

/// For tests and the control channel: the stage whose drawer shows.
pub fn stage(ctx: &egui::Context) -> Stage {
    super::chosen(ctx)
}
