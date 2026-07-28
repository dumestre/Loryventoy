use crate::domain::Color;

/// Dados persistentes de uma layer.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerEntry {
    pub nome: String,
    pub ordem: f32,
    pub opacidade: f32,
    pub cor: Color,
    pub visivel: bool,
}

impl LayerEntry {
    const PALETTE: [Color; 8] = [
        Color::from_rgb(90, 170, 235),
        Color::from_rgb(235, 150, 120),
        Color::from_rgb(150, 200, 120),
        Color::from_rgb(200, 120, 220),
        Color::from_rgb(235, 185, 95),
        Color::from_rgb(120, 200, 220),
        Color::from_rgb(230, 130, 170),
        Color::from_rgb(170, 120, 235),
    ];

    pub fn cor_por_idx(idx: usize) -> Color {
        Self::PALETTE[idx % Self::PALETTE.len()]
    }
}
