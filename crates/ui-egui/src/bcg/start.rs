//! The Start screen's top: a heading, three ways in (Open a PDF in navy, Create a PDF, Combine
//! files) and the drop hint. Upstream's pinned folders and Recent list follow it unchanged.
//! The three cards rise in once, 50 ms apart, the first time the screen shows.

use egui::{Align2, CornerRadius, Id, Rect, Sense, Stroke, StrokeKind, vec2};

use super::color::{NAVY, NAVY_2, SAGE, SAGE_SOFT};
use super::stages::Stage;
use super::{mix, secs};
use crate::PdfCraftApp;
use crate::theme::{self, Tokens};

/// The display face: the BCG wordmark and the Start heading only (`DESIGN.md`). Yeseva One when
/// the BCG fonts are built in ([`super::fonts`]), the app's semibold otherwise.
pub fn display(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(super::fonts::DISPLAY.into()))
}

enum Way {
    Open,
    Create,
    Combine,
}

pub fn hero(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Start with a file").font(display(30.0)).color(t.text));
    ui.add_space(2.0);
    ui.add(
        egui::Label::new(
            egui::RichText::new("Quotes, drawings, spec sheets and contracts all open here. Files stay on this computer: nothing is uploaded.")
                .font(theme::regular(14.5))
                .color(t.text_muted),
        )
        .wrap(),
    );
    ui.add_space(18.0);
    let ways = [
        (Way::Open, "folder-open", "Open a PDF", "Read, mark up, sign or edit it."),
        (Way::Create, "file-plus-2", "Create a PDF", "From Word files, images, a photo of a page, or a blank page."),
        (Way::Combine, "copy-plus", "Combine files", "Put several files into one PDF, in the order you choose."),
    ];
    let gap = 14.0;
    let avail = ui.available_width();
    let stacked = avail < 640.0;
    let w = if stacked { avail } else { ((avail - gap * 2.0) / 3.0).floor() };
    let mut chosen = None;
    let layout = if stacked { egui::Layout::top_down(egui::Align::Min) } else { egui::Layout::left_to_right(egui::Align::Min) };
    ui.with_layout(layout, |ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        for (i, (way, icon, title, line)) in ways.into_iter().enumerate() {
            if card(ui, &t, i, w, i == 0, icon, title, line).clicked() {
                chosen = Some(way);
            }
        }
    });
    ui.add_space(14.0);
    drop_hint(ui, &t);
    match chosen {
        Some(Way::Open) => app.open_dialog(),
        Some(Way::Create) => super::go(app, &ui.ctx().clone(), Stage::Create),
        Some(Way::Combine) => app.run_command("page.combine"),
        None => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn card(ui: &mut egui::Ui, t: &Tokens, index: usize, width: f32, lead: bool, icon: &str, title: &str, line: &str) -> egui::Response {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 150.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title));
    let resp = theme::hand(resp);
    // Rise in once, staggered: 360 ms each, 50 ms apart.
    let rise = super::entrance(&ctx, Id::new("bcg_start_rise").with(index), 0, 0.36 + 0.05 * index as f32);
    let hot = resp.hovered() || resp.is_pointer_button_down_on();
    let hover = ctx.animate_bool_with_time(resp.id.with("bcg-h"), hot, secs(&ctx, 0.18));
    let down = ctx.animate_bool_with_time(resp.id.with("bcg-d"), resp.is_pointer_button_down_on(), secs(&ctx, 0.09));
    let body = Rect::from_center_size(rect.center() + vec2(0.0, 10.0 * (1.0 - rise)), rect.size() * (1.0 - 0.015 * down));
    let alpha = rise;
    let p = ui.painter();
    if hover > 0.01 {
        let shadow = egui::Shadow { offset: [0, 14], blur: 30, spread: 0, color: NAVY.gamma_multiply(0.3 * hover * alpha) };
        p.add(shadow.as_shape(body, CornerRadius::same(12)));
    }
    let (fill, border, ink, sub, chip, chip_ink) = if lead {
        (mix(NAVY, NAVY_2, hover), mix(NAVY, NAVY_2, hover), egui::Color32::WHITE, egui::Color32::from_rgb(0xDC, 0xE7, 0xE6), SAGE, NAVY)
    } else {
        (t.card, mix(t.border, SAGE, hover), t.text, t.text_muted, t.accent_soft, t.accent_text)
    };
    p.rect(body, CornerRadius::same(12), fill.gamma_multiply(alpha), Stroke::new(1.0, border.gamma_multiply(alpha)), StrokeKind::Inside);
    let chip_rect = Rect::from_min_size(body.min + vec2(20.0, 20.0), vec2(44.0, 44.0));
    p.rect_filled(chip_rect, CornerRadius::same(10), chip.gamma_multiply(alpha));
    crate::icons::paint(ui, chip_rect, icon, 22.0, chip_ink.gamma_multiply(alpha));
    let p = ui.painter();
    p.text(body.left_top() + vec2(20.0, 80.0), Align2::LEFT_TOP, title, theme::semibold(17.0), ink.gamma_multiply(alpha));
    let mut job = egui::text::LayoutJob::simple(line.to_owned(), theme::regular(13.5), sub.gamma_multiply(alpha), body.width() - 40.0);
    job.wrap.max_rows = 2;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(body.left_top() + vec2(20.0, 106.0), galley, sub);
    if resp.has_focus() {
        ui.painter().rect_stroke(rect.expand(2.0), CornerRadius::same(14), Stroke::new(2.0, t.accent_text), StrokeKind::Outside);
    }
    resp
}

fn drop_hint(ui: &mut egui::Ui, t: &Tokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(12), t.card, Stroke::new(1.5, mix(t.border, SAGE_SOFT, 0.5)), StrokeKind::Inside);
    let text = "Or drop PDFs anywhere in this window to open them.";
    let font = theme::regular(14.0);
    let w = crate::widgets::text_width(ui, text, &font);
    let left = rect.center().x - (w + 30.0) / 2.0;
    crate::icons::paint(ui, Rect::from_center_size(egui::pos2(left + 10.0, rect.center().y), vec2(20.0, 20.0)), "cloud-upload", 19.0, t.accent_text);
    ui.painter().text(egui::pos2(left + 30.0, rect.center().y), Align2::LEFT_CENTER, text, font, t.text_muted);
}

#[cfg(test)]
mod tests {
    use egui::{Rect, vec2};

    use crate::PdfCraftApp;
    use crate::theme;

    /// The three ways in show with their titles, and each card's line stays within two rows.
    #[test]
    fn start_cards_fit_their_text() {
        let ctx = egui::Context::default();
        ctx.set_fonts(theme::font_definitions());
        let mut app = PdfCraftApp::new();
        super::super::enable(&mut app, &ctx);
        let input = || egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1100.0, 800.0))), ..Default::default() };
        // Fonts registered via set_fonts take effect on the next frame.
        ctx.run_ui(input(), |ui| crate::home::show(&mut app, ui)).drop_without_applying_deltas();
        let output = ctx.run_ui(input(), |ui| crate::home::show(&mut app, ui));
        let texts: Vec<_> =
            output.shapes.iter().filter_map(|shape| if let egui::Shape::Text(text) = &shape.shape { Some(text) } else { None }).collect();
        for title in ["Open a PDF", "Create a PDF", "Combine files", "Start with a file"] {
            assert!(texts.iter().any(|t| t.galley.job.text == title), "{title}");
        }
        assert!(!texts.iter().any(|t| t.galley.job.text.contains("Welcome to")), "upstream's welcome is replaced");
        let line = texts.iter().find(|t| t.galley.job.text.starts_with("From Word files")).expect("Create card line");
        assert!(line.galley.rows.len() <= 2);
        output.drop_without_applying_deltas();
    }
}
