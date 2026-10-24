//! BCG PDF build: the workflow rail, its drawers and the Start screen.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::bcg::{self, Stage};
use pdfcraft_ui_egui::{LeftPanel, PdfCraftApp};

const PDF: &[u8] = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >> endobj\n4 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF";

fn harness(with_doc: bool) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        bcg::enable(&mut app, &cc.egui_ctx);
        if with_doc {
            app.open_bytes("quote.pdf", None, PDF.to_vec()).unwrap();
        }
        app
    });
    h.run_steps(4);
    h
}

/// Click a rail button by its accessible name ("Edit tools", "Start screen").
fn rail(h: &mut Harness<'static, PdfCraftApp>, name: &str) {
    h.get_by_label(name).click();
    h.run_steps(3);
}

#[test]
fn the_rail_lists_every_stage_and_replaces_the_mode_tabs() {
    let h = harness(true);
    for stage in Stage::RAIL {
        h.get_by_label(&bcg::rail::accessible_name(stage));
    }
    h.get_by_label("More tools");
    h.get_by_label("App settings");
    for gone in ["All tools", "Read", "Convert", "E-Sign"] {
        assert_eq!(h.query_all_by_label(gone).count(), 0, "mode tab {gone}");
    }
}

#[test]
fn start_shows_the_ways_in_with_the_tools_closed() {
    let h = harness(false);
    assert!(!h.state().left_open, "Start has no drawer");
    h.get_by_label("Start with a file");
    for way in ["Open a PDF", "Create a PDF", "Combine files"] {
        h.get_by_label(way);
    }
}

#[test]
fn a_stage_opens_its_drawer_and_a_second_click_closes_it() {
    let mut h = harness(true);
    rail(&mut h, "Protect tools");
    assert!(h.state().left_open);
    assert_eq!(h.state().left, LeftPanel::AllTools);
    h.get_by_label("Add a password");
    h.get_by_label("Mark text and images");
    h.get_by_label("Encrypt for certain people");
    rail(&mut h, "Protect tools");
    assert!(!h.state().left_open, "picking the current stage again closes its drawer");
}

#[test]
fn the_create_card_opens_the_create_drawer() {
    let mut h = harness(false);
    h.get_by_label("Create a PDF").click();
    h.run_steps(3);
    assert!(h.state().left_open);
    h.get_by_label("Create from a file");
    h.get_by_label("Several files, one PDF");
}

#[test]
fn pages_opens_the_page_grid() {
    let mut h = harness(true);
    rail(&mut h, "Pages tools");
    let app = h.state();
    assert!(app.views[0].organize, "the page grid shows");
    assert_eq!(app.left, LeftPanel::Tool("organize"));
}

#[test]
fn a_tool_with_its_own_panel_opens_it_and_the_rail_follows() {
    let mut h = harness(true);
    rail(&mut h, "Review tools");
    h.get_by_label("Stamp").click();
    h.run_steps(3);
    assert_eq!(h.state().left, LeftPanel::Tool("stamp"));
    let ctx = h.ctx.clone();
    assert_eq!(bcg::current(h.state(), &ctx), Some(Stage::Review));
}

#[test]
fn a_document_opens_on_the_edit_drawer_and_its_command_runs() {
    let mut h = harness(true);
    // Opening a document moves the rail from Start to Edit, with its drawer open.
    let ctx = h.ctx.clone();
    assert_eq!(bcg::current(h.state(), &ctx), Some(Stage::Edit));
    assert!(h.state().left_open);
    h.get_by_label("Edit text & images").click();
    h.run_steps(3);
    assert_eq!(h.state().quick_tool, pdfcraft_ui_egui::QuickTool::EditText);
}

#[test]
fn more_tools_shows_the_full_tool_list() {
    let mut h = harness(true);
    rail(&mut h, "More tools");
    assert!(h.state().left_open);
    h.get_by_label("All tools");
    assert!(h.query_all_by_label("Use JavaScript").count() + h.query_all_by_label("View more").count() >= 1);
}

#[test]
fn start_closes_the_drawer_and_shows_the_start_screen() {
    let mut h = harness(true);
    rail(&mut h, "Review tools");
    assert!(h.state().left_open, "the Review drawer is open");
    rail(&mut h, "Start screen");
    assert!(h.state().active.is_none());
    assert!(!h.state().left_open);
    h.get_by_label("Start with a file");
}

/// Upstream's Edit tab behaviour, through the rail: Edit text & images is put down when another
/// stage is chosen, and a tool picked inside Edit is not taken away by leaving it.
#[test]
fn leaving_edit_puts_the_edit_tool_down() {
    use pdfcraft_ui_egui::QuickTool;
    let mut h = harness(true);
    h.get_by_label("Edit text & images").click();
    h.run_steps(3);
    assert_eq!(h.state().quick_tool, QuickTool::EditText);
    rail(&mut h, "Review tools");
    assert_eq!(h.state().quick_tool, QuickTool::Select);
    rail(&mut h, "Edit tools");
    h.state_mut().quick_tool = QuickTool::Hand;
    rail(&mut h, "Export tools");
    assert_eq!(h.state().quick_tool, QuickTool::Hand);
}

/// At the desktop window's minimum size (apps/pdfcraft/src/main.rs, 916 by 520) the stages
/// scroll and More tools and Settings stay whole at the foot, inside the window.
#[test]
fn the_rail_fits_the_smallest_window() {
    let mut h = Harness::builder().with_size(egui::vec2(916.0, 520.0)).build_eframe(|cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        bcg::enable(&mut app, &cc.egui_ctx);
        app.open_bytes("quote.pdf", None, PDF.to_vec()).unwrap();
        app
    });
    h.run_steps(4);
    let window = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(916.0, 520.0));
    let more = h.get_by_label("More tools").rect();
    let settings = h.get_by_label("App settings").rect();
    for r in [more, settings] {
        assert!(window.contains_rect(r), "{r:?} is inside the window");
        assert!(r.right() <= bcg::rail::WIDTH, "{r:?} is on the rail");
    }
    assert!(!more.intersects(settings));
    // The stage list starts under the wordmark, in the window, above the foot.
    let start = h.get_by_label("Start screen").rect();
    assert!(window.contains_rect(start) && start.bottom() < more.top(), "{start:?}");
}
