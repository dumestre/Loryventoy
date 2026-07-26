use eframe::egui::{Context, Event, MouseWheelUnit};

/// Soma do scroll do frame (mouse e trackpad de 2 dedos), em pontos de
/// tela. Lê os eventos `MouseWheel` brutos diretamente e, como reforço,
/// soma o `smooth_scroll_delta` (caso os eventos já tenham sido
/// consumidos/zerados por algum ScrollArea ou plataforma específica).
pub fn scroll_delta(ctx: &Context) -> eframe::egui::Vec2 {
    let pp = ctx.pixels_per_point();
    let mut total = eframe::egui::Vec2::ZERO;
    for ev in &ctx.input(|i| i.events.clone()) {
        if let Event::MouseWheel { unit, delta, .. } = ev {
            let d = match unit {
                MouseWheelUnit::Point => *delta,
                MouseWheelUnit::Line => *delta * 16.0 * pp,
                MouseWheelUnit::Page => *delta * 100.0 * pp,
            };
            total += d;
        }
    }
    total += ctx.input(|i| i.smooth_scroll_delta());
    total
}

pub mod splitter;
pub mod preview;
pub mod timeline;
pub mod graph;
pub mod bartool;
pub mod graph_toolbar;
pub mod node_component;
pub mod text_raster;