mod app;
mod nodes;
mod projeto_arquivo;
mod procedural;
mod dsl;
mod graph_editor;
mod theme;
mod ui;
mod export;
mod log;

/// Lê `app.ico` e converte para `eframe::IconData` (RGBA + tamanho). Retorna
/// `None` se o arquivo não existir ou falhar ao decodificar.
fn load_icon() -> Option<std::sync::Arc<eframe::egui::IconData>> {
    let bytes = std::fs::read("app.ico").ok()?;
    let img = image::load(std::io::Cursor::new(bytes), image::ImageFormat::Ico).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(std::sync::Arc::new(eframe::egui::IconData {
        rgba: rgba.into_raw(),
        width: w,
        height: h,
    }))
}

fn main() -> eframe::Result<()> {
    // Carrega o ícone (app.ico) para a janela/barra de tarefas em runtime.
    // O `Cargo.toml` também embuteia o .ico no próprio .exe no Windows.
    let icon_data = load_icon();

    let viewport = eframe::egui::ViewportBuilder::default().with_maximized(true);
    let viewport = if let Some(icon) = icon_data {
        viewport.with_icon(icon)
    } else {
        viewport
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Lory Hub",
        options,
        Box::new(|cc| {
            Ok(Box::new(
                app::MovimentoApp::new(cc)
            ))
        }),
    )
}

