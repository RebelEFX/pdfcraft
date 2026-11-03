//! What staff never see: tools that don't work in this build. With the BCG shell on, a tool
//! upstream's catalogue lists as planned (a milestone) or as needing an AI provider is hidden
//! everywhere it would show (the rail's drawers, More tools, the page grid's panel, Find a tool
//! or command), and so is text recognition while its model files are missing (the web build
//! never has them). Nothing is removed: when upstream finishes a tool, or the models are
//! installed, it appears without any change here.
//!
//! With the shell off, upstream's interface lists everything as before.

use egui::Context;
use pdfcraft_engine::catalog::{Availability, TOOL_GROUPS, ToolGroup};

/// Commands that only work with the text-recognition models installed.
const NEEDS_OCR: &[&str] = &["ocr.recognize", "ocr.recognize_batch"];

/// Whether `command` works in this build: upstream's catalogue lists it as ready (a command it
/// doesn't list counts when it is registered, or names no command at all), and any model files
/// it needs are installed.
pub fn works(command: &str) -> bool {
    if NEEDS_OCR.contains(&command) && !pdfcraft_engine::ocr::available() {
        return false;
    }
    let mut listed = TOOL_GROUPS.iter().flat_map(|g| g.sections).flat_map(|s| s.items).filter(|i| i.command == command).peekable();
    if listed.peek().is_none() {
        return true;
    }
    listed.any(|i| matches!(i.availability, Availability::Ready))
}

/// Whether a tool group has anything that works: at least one working tool. A group's own
/// availability says when the whole feature is finished (Edit a PDF is "planned" while most of
/// its tools already work), so only a group with no tools listed goes by it.
pub fn group_works(g: &ToolGroup) -> bool {
    let mut items = g.sections.iter().flat_map(|s| s.items).peekable();
    if items.peek().is_none() {
        return matches!(g.availability, Availability::Ready);
    }
    items.any(|i| works(i.command))
}

/// Whether to hide the tool that runs `command`: the shell is on and the tool doesn't work.
pub fn hides(ctx: &Context, command: &str) -> bool {
    super::enabled(ctx) && !works(command)
}

/// Whether to hide a whole tool group (its row in More tools, its palette entry).
pub fn hides_group(ctx: &Context, g: &ToolGroup) -> bool {
    super::enabled(ctx) && !group_works(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_and_provider_tools_do_not_work_and_ready_ones_do() {
        assert!(!works("edit.attach"), "planned in the catalogue");
        assert!(!works("protect.certificate"));
        assert!(works("edit.watermark"));
        assert!(works("print.dialog"), "registered, not in the catalogue");
        assert_eq!(works("ocr.recognize"), pdfcraft_engine::ocr::available(), "needs the models");
        let group = |id| pdfcraft_engine::catalog::group(id).expect("catalogue group");
        assert!(group_works(group("edit")), "planned as a whole, but most of its tools work");
        assert!(group_works(group("protect")));
        assert!(!group_works(group("javascript")), "every tool planned");
        assert!(!group_works(group("ai")), "needs an AI provider");
    }

    #[test]
    fn nothing_hides_with_the_shell_off() {
        let ctx = Context::default();
        assert!(!hides(&ctx, "edit.attach"));
        assert!(TOOL_GROUPS.iter().all(|g| !hides_group(&ctx, g)));
        let mut app = crate::PdfCraftApp::new();
        super::super::enable(&mut app, &ctx);
        assert!(hides(&ctx, "edit.attach"));
        assert!(!hides(&ctx, "edit.watermark"));
    }
}
