use eframe::egui::Color32;

/// Dados de uma layer usados pelo modelo atual do projeto.
///
/// `renomeando` ainda é mantido temporariamente por compatibilidade com a UI.
/// Ele será movido para o estado de apresentação em uma etapa específica.
#[derive(Clone, Debug)]
pub struct LayerEntry {
    pub nome: String,
    pub ordem: f32,
    pub opacidade: f32,
    pub cor: Color32,
    pub visivel: bool,
    pub renomeando: bool,
}

impl LayerEntry {
    const PALETTE: [Color32; 8] = [
        Color32::from_rgb(90, 170, 235),
        Color32::from_rgb(235, 150, 120),
        Color32::from_rgb(150, 200, 120),
        Color32::from_rgb(200, 120, 220),
        Color32::from_rgb(235, 185, 95),
        Color32::from_rgb(120, 200, 220),
        Color32::from_rgb(230, 130, 170),
        Color32::from_rgb(170, 120, 235),
    ];

    pub fn cor_por_idx(idx: usize) -> Color32 {
        Self::PALETTE[idx % Self::PALETTE.len()]
    }
}
