use crate::nodes::RuidoParams;

use super::{grid_2, grid_combo_alvo};

pub fn show(ui: &mut eframe::egui::Ui, ruido: &mut RuidoParams) {
    grid_combo_alvo(ui, "Alvo", &mut ruido.alvo);
    grid_2(ui, "Seed", &mut ruido.seed, 0.0..=9999.0, "", 0);
    grid_2(ui, "Frequência", &mut ruido.freq, 0.01..=5.0, "", 2);
    grid_2(ui, "Amplitude", &mut ruido.amp, 0.0..=1000.0, "", 1);
    grid_2(ui, "Velocidade", &mut ruido.veloc, 0.0..=10.0, "x", 2);
}
