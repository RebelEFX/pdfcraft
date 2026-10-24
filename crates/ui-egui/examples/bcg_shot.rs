//! BCG PDF build: a headless screenshot of the app on a given rail stage.
//!
//! ```text
//! cargo run -p pdfcraft-ui-egui --example bcg_shot -- out.png [file.pdf] [--stage edit] [--size 1440x900]
//! ```

use std::time::{Duration, Instant};

use egui_kittest::Harness;
use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::bcg::Stage;

fn stage(name: &str) -> Result<Stage, String> {
    Ok(match name {
        "start" => Stage::Start,
        "create" => Stage::Create,
        "edit" => Stage::Edit,
        "review" => Stage::Review,
        "pages" => Stage::Pages,
        "sign" => Stage::Sign,
        "protect" => Stage::Protect,
        "export" => Stage::Export,
        "more" => Stage::More,
        _ => return Err(format!("unknown stage {name}")),
    })
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let out = args.next().ok_or("usage: bcg_shot <out.png> [file.pdf] [--stage name] [--size WxH]")?;
    let (mut file, mut pick, mut size) = (None, Stage::Start, egui::vec2(1440.0, 900.0));
    while let Some(a) = args.next() {
        match a.as_str() {
            "--stage" => pick = stage(&args.next().unwrap_or_default())?,
            "--size" => {
                let v = args.next().unwrap_or_default();
                let (w, h) = v.split_once('x').ok_or("--size WxH")?;
                size = egui::vec2(w.parse().map_err(|_| "bad width")?, h.parse().map_err(|_| "bad height")?);
            }
            _ => file = Some(a),
        }
    }
    let mut harness = Harness::builder().with_size(size).with_pixels_per_point(1.0).build_eframe(move |cc| {
        let mut app = PdfCraftApp::new();
        let _ = app.set_option("language", "en");
        pdfcraft_ui_egui::bcg::enable(&mut app, &cc.egui_ctx);
        if let Some(f) = &file {
            let bytes = std::fs::read(f).unwrap_or_default();
            let name = std::path::Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if let Err(e) = app.open_bytes(&name, Some(f.clone()), bytes) {
                eprintln!("bcg_shot: {f}: {e}");
            }
        }
        pdfcraft_ui_egui::bcg::go(&mut app, &cc.egui_ctx, pick);
        app
    });
    let start = Instant::now();
    harness.run_steps(6);
    while start.elapsed() < Duration::from_secs(20) {
        harness.run_steps(2);
        if !harness.state().render_pending() {
            harness.run_steps(20);
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    let image = harness.render()?;
    image.save(&out).map_err(|e| e.to_string())?;
    eprintln!("bcg_shot: wrote {out} ({}×{})", image.width(), image.height());
    Ok(())
}
