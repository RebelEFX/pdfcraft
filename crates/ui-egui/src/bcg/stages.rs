//! The workflow rail's stages and the tools in each drawer: a table of upstream command ids and
//! tool groups, with BCG wording. Whether a tool works yet comes from upstream's catalogue, so a
//! tool PdfCraft finishes turns on here by itself.

use pdfcraft_engine::catalog::{Availability, TOOL_GROUPS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Start,
    Create,
    Edit,
    Review,
    Pages,
    Sign,
    Protect,
    Export,
    /// Upstream's full tool list, for tools no stage offers.
    More,
}

impl Stage {
    /// The rail's stages, top to bottom (More tools sits at the rail's foot).
    pub const RAIL: [Stage; 8] = [Stage::Start, Stage::Create, Stage::Edit, Stage::Review, Stage::Pages, Stage::Sign, Stage::Protect, Stage::Export];

    pub fn spec(self) -> &'static StageSpec {
        match self {
            Stage::Start => &START,
            Stage::Create => &CREATE,
            Stage::Edit => &EDIT,
            Stage::Review => &REVIEW,
            Stage::Pages => &PAGES,
            Stage::Sign => &SIGN,
            Stage::Protect => &PROTECT,
            Stage::Export => &EXPORT,
            Stage::More => &MORE,
        }
    }

    /// The stage an upstream tool group belongs to, so the rail follows a command that opened
    /// one of upstream's tool panels.
    pub fn for_group(group: &str) -> Option<Stage> {
        Some(match group {
            "create" | "combine" | "scan" => Stage::Create,
            "edit" => Stage::Edit,
            "comment" | "stamp" | "measure" | "compare" => Stage::Review,
            "organize" => Stage::Pages,
            "fill_sign" | "form" | "certificate" => Stage::Sign,
            "protect" | "redact" => Stage::Protect,
            "export" | "compress" | "print_production" => Stage::Export,
            _ => Stage::More,
        })
    }
}

/// What a drawer entry does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Run a registered command.
    Command(&'static str),
    /// Open an upstream tool panel that has live controls (`app.select_tool`).
    Tool(&'static str),
    /// A menu of commands: Add / Update / Remove (header and footer, watermark, background).
    Menu(&'static [&'static str]),
}

impl Action {
    /// The command whose catalogue entry says whether this works yet.
    pub fn catalogue_command(self) -> Option<&'static str> {
        match self {
            Action::Command(c) => Some(c),
            Action::Menu(actions) => actions.first().copied(),
            Action::Tool(_) => None,
        }
    }

    /// Whether the tool works today, per upstream's catalogue. A command the catalogue doesn't
    /// list is ready when it is registered.
    pub fn ready(self) -> bool {
        match self {
            Action::Tool(group) => pdfcraft_engine::catalog::group(group).is_some_and(|g| matches!(g.availability, Availability::Ready)),
            _ => {
                let Some(command) = self.catalogue_command() else { return false };
                let listed = TOOL_GROUPS.iter().flat_map(|g| g.sections).flat_map(|s| s.items).find(|i| i.command == command);
                match listed {
                    Some(item) => matches!(item.availability, Availability::Ready),
                    None => pdfcraft_engine::commands::command(command).is_some(),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub label: &'static str,
    /// One line on what it does (format cards: the file type).
    pub hint: &'static str,
    /// Lucide icon name (assets/icons).
    pub icon: &'static str,
    pub action: Action,
}

#[derive(Debug)]
pub struct Section {
    pub title: &'static str,
    pub entries: &'static [Entry],
}

#[derive(Debug)]
pub struct StageSpec {
    pub stage: Stage,
    /// The rail label.
    pub label: &'static str,
    pub icon: &'static str,
    /// The drawer heading and its one line.
    pub title: &'static str,
    pub blurb: &'static str,
    /// The one navy main action.
    pub primary: Option<Entry>,
    /// Format cards (Export).
    pub formats: &'static [Entry],
    pub sections: &'static [Section],
}

const fn e(label: &'static str, hint: &'static str, icon: &'static str, action: Action) -> Entry {
    Entry { label, hint, icon, action }
}

use Action::{Command as C, Menu as M, Tool as T};

static START: StageSpec =
    StageSpec { stage: Stage::Start, label: "Start", icon: "house", title: "Start", blurb: "", primary: None, formats: &[], sections: &[] };

static CREATE: StageSpec = StageSpec {
    stage: Stage::Create,
    label: "Create",
    icon: "file-plus-2",
    title: "Create a PDF",
    blurb: "Turn files, images or a photo of a page into a PDF.",
    primary: Some(e("Create from a file", "", "file-input", C("create.file"))),
    formats: &[],
    sections: &[
        Section {
            title: "From",
            entries: &[
                e("Several files, one PDF", "Word files, images and PDFs, in your order", "files", C("create.multiple")),
                e("Separate PDFs", "One PDF for each file you pick", "files", C("create.multiple_separate")),
                e("Images", "Photos or screenshots, a page each", "image", C("create.images")),
                e("Photo of a page", "Straightens and cleans up a camera shot", "camera", C("ocr.enhance_camera")),
                e("Clipboard", "Whatever you last copied", "clipboard-paste", C("create.clipboard")),
                e("Blank page", "Start from an empty page", "file-plus", C("create.blank")),
                e("Combine files", "Put several files into one, in order", "copy-plus", C("page.combine")),
            ],
        },
        Section {
            title: "Make it searchable",
            entries: &[
                e("Recognize text", "Lets you search and copy a scan", "scan-text", C("ocr.recognize")),
                e("Recognize text in several files", "Runs on a whole batch", "files", C("ocr.recognize_batch")),
            ],
        },
    ],
};

static EDIT: StageSpec = StageSpec {
    stage: Stage::Edit,
    label: "Edit",
    icon: "pen-line",
    title: "Edit",
    blurb: "Change what is on the page, or add to it.",
    primary: Some(e("Edit text & images", "", "text-cursor-input", C("edit.edit_text"))),
    formats: &[],
    sections: &[Section {
        title: "Add to the page",
        entries: &[
            e("Text", "Type anywhere on the page", "type", C("edit.text")),
            e("Image", "Place a picture or logo", "image-plus", C("edit.image")),
            e("Link", "To a web page or another page", "link-2", C("edit.link")),
            e(
                "Header and footer",
                "Page numbers, dates, titles",
                "heading",
                M(&["edit.header_footer", "edit.header_footer.update", "edit.header_footer.remove"]),
            ),
            e("Watermark", "Draft, Confidential, or your own", "droplet", M(&["edit.watermark", "edit.watermark.update", "edit.watermark.remove"])),
            e(
                "Background",
                "A colour or image behind the page",
                "paint-bucket",
                M(&["edit.background", "edit.background.update", "edit.background.remove"]),
            ),
            e("Bates numbering", "Numbers every page for records", "hash", C("edit.bates")),
            e("Attach a file", "Embed a file in the PDF", "paperclip", C("edit.attach")),
        ],
    }],
};

static REVIEW: StageSpec = StageSpec {
    stage: Stage::Review,
    label: "Review",
    icon: "message-square-text",
    title: "Review",
    blurb: "Mark up a drawing or a draft and send it back.",
    primary: None,
    formats: &[],
    sections: &[
        Section {
            title: "Mark up",
            entries: &[
                e("Sticky note", "A comment pinned to a spot", "sticky-note", C("comment.note")),
                e("Highlight", "Select text to highlight it", "highlighter", C("comment.highlight")),
                e("Underline", "Select text to underline it", "underline", C("comment.underline")),
                e("Strike through", "Show text to remove", "strikethrough", C("comment.strikeout")),
                e("Text box", "A note written on the page", "type", C("comment.freetext")),
            ],
        },
        Section {
            title: "Draw",
            entries: &[
                e("Freehand", "Circle a detail, sketch a fix", "pencil", C("comment.ink")),
                e("Box", "Frame an area", "square", C("comment.square")),
                e("Arrow", "Point at something", "move-right", C("comment.arrow")),
                e("Stamp", "Approved, Revised, Void", "stamp", T("stamp")),
                e("Measure", "Distances and areas on a drawing", "ruler", T("measure")),
            ],
        },
        Section {
            title: "Go over it",
            entries: &[
                e("All comments", "Read, reply and resolve", "message-square-text", C("comment.list")),
                e("Compare two files", "See what changed between versions", "git-compare", C("doc.compare")),
                e("Flatten comments", "Make markup part of the page", "layers", C("comment.flatten")),
            ],
        },
    ],
};

static PAGES: StageSpec = StageSpec {
    stage: Stage::Pages,
    label: "Pages",
    icon: "layout-grid",
    title: "Pages",
    blurb: "Open a PDF to see its pages here, then pick what to do with them.",
    primary: Some(e("Open a PDF", "", "folder-open", C("file.open"))),
    formats: &[],
    sections: &[],
};

static SIGN: StageSpec = StageSpec {
    stage: Stage::Sign,
    label: "Sign",
    icon: "signature",
    title: "Fill & sign",
    blurb: "Fill in a form, then sign it.",
    primary: Some(e("Sign this document", "", "signature", T("fill_sign"))),
    formats: &[],
    sections: &[
        Section {
            title: "Fill in",
            entries: &[
                e("Text", "Type into any blank", "type", C("sign.fill.text")),
                e("Check mark", "Tick a box", "check", C("sign.fill.check")),
                e("Cross", "Mark a box with an X", "x", C("sign.fill.cross")),
                e("Date", "Today's date, in one click", "clock-3", C("sign.fill.date")),
            ],
        },
        Section {
            title: "Forms",
            entries: &[
                e("Prepare a form", "Find the fields and make them fillable", "text-cursor-input", T("form")),
                e("Sign with a certificate", "A digital ID that proves who signed", "badge-check", T("certificate")),
            ],
        },
    ],
};

static PROTECT: StageSpec = StageSpec {
    stage: Stage::Protect,
    label: "Protect",
    icon: "shield-check",
    title: "Protect",
    blurb: "Control who can open it, and remove what should not leave.",
    primary: Some(e("Add a password", "", "lock", C("protect.password"))),
    formats: &[],
    sections: &[
        Section {
            title: "Redact",
            entries: &[
                e("Mark text and images", "Blacked out for good when applied", "rectangle-horizontal", C("redact.mark")),
                e("Find and redact", "Every match of a name or number", "file-search", C("redact.search")),
                e("Whole pages", "Remove pages entirely", "file-x", C("redact.pages")),
            ],
        },
        Section {
            title: "Clean up",
            entries: &[
                e("Remove hidden information", "Metadata, comments, hidden layers", "eye-off", C("protect.remove_hidden")),
                e("Sanitize", "Everything hidden, in one pass", "sparkles", C("redact.sanitize")),
                e("Security settings", "What people can print or change", "shield-check", C("protect.properties")),
                e("Remove a password", "If you know it", "lock-open", C("protect.remove")),
                e("Encrypt for certain people", "With their digital IDs", "file-lock-2", C("protect.certificate")),
            ],
        },
    ],
};

static EXPORT: StageSpec = StageSpec {
    stage: Stage::Export,
    label: "Export",
    icon: "share-2",
    title: "Export",
    blurb: "Save a copy in another format, smaller, or on paper.",
    primary: None,
    formats: &[
        e("Word", ".docx", "file-text", C("export.docx")),
        e("PowerPoint", ".pptx", "presentation", C("export.pptx")),
        e("Image", ".png", "image", C("export.image")),
        e("All images", "Every picture inside", "image", C("export.all_images")),
        e("Plain text", ".txt", "type", C("export.text")),
        e("Rich text", ".rtf", "file-text", C("export.rtf")),
        e("Web page", ".html", "code-xml", C("export.html")),
        e("Excel", ".xlsx", "file-spreadsheet", C("export.xlsx")),
        e("PostScript", ".ps and .eps", "file-down", C("export.ps")),
    ],
    sections: &[Section {
        title: "Also",
        entries: &[
            e("Make the file smaller", "For email and upload limits", "file-down", C("optimize.reduce")),
            e("Print", "To any printer on this computer", "printer", C("print.dialog")),
        ],
    }],
};

static MORE: StageSpec =
    StageSpec { stage: Stage::More, label: "More", icon: "ellipsis", title: "All tools", blurb: "", primary: None, formats: &[], sections: &[] };

#[cfg(test)]
mod tests {
    use super::*;

    fn all_entries() -> impl Iterator<Item = &'static Entry> {
        Stage::RAIL.into_iter().chain([Stage::More]).flat_map(|s| {
            let spec = s.spec();
            spec.primary.iter().chain(spec.formats).chain(spec.sections.iter().flat_map(|sec| sec.entries))
        })
    }

    /// A command is registered, or is a tool upstream's catalogue lists as not built yet (shown
    /// under "Coming later" until upstream builds and registers it).
    fn known(command: &str) -> bool {
        pdfcraft_engine::commands::command(command).is_some()
            || TOOL_GROUPS
                .iter()
                .flat_map(|g| g.sections)
                .flat_map(|s| s.items)
                .any(|i| i.command == command && !matches!(i.availability, Availability::Ready))
    }

    /// An upstream rename must fail here, not leave a button that does nothing.
    #[test]
    fn every_entry_runs_a_registered_command_or_opens_a_real_tool_panel() {
        for entry in all_entries() {
            match entry.action {
                Action::Command(c) => assert!(known(c), "{}: {c}", entry.label),
                Action::Menu(actions) => {
                    for c in actions {
                        assert!(known(c), "{}: {c}", entry.label);
                    }
                }
                Action::Tool(g) => assert!(pdfcraft_engine::catalog::group(g).is_some(), "{}: {g}", entry.label),
            }
        }
    }

    #[test]
    fn every_icon_exists() {
        for s in Stage::RAIL.into_iter().chain([Stage::More]) {
            assert!(crate::icons::exists(s.spec().icon), "{:?}", s);
        }
        for entry in all_entries() {
            assert!(crate::icons::exists(entry.icon), "{}: {}", entry.label, entry.icon);
        }
    }

    #[test]
    fn specs_match_their_stage_and_groups_map_to_stages() {
        for s in Stage::RAIL.into_iter().chain([Stage::More]) {
            assert_eq!(s.spec().stage, s);
        }
        for g in TOOL_GROUPS {
            assert!(Stage::for_group(g.id).is_some(), "{}", g.id);
        }
        assert_eq!(Stage::for_group("organize"), Some(Stage::Pages));
        assert_eq!(Stage::for_group("javascript"), Some(Stage::More));
    }

    /// Planned tools come from upstream's catalogue: these are planned in 0.9.0, ready ones aren't.
    #[test]
    fn availability_follows_the_catalogue() {
        assert!(Action::Command("export.docx").ready());
        assert!(!Action::Command("export.xlsx").ready());
        assert!(!Action::Command("edit.attach").ready());
        assert!(Action::Command("print.dialog").ready(), "registered but not in the catalogue");
        assert!(Action::Tool("fill_sign").ready());
        assert!(Action::Menu(&["edit.watermark"]).ready());
    }
}
