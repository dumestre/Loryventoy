use crate::nodes::{SaidaParams, TransformParams};

use super::{grid_2, grid_xyz};

pub fn show_transform(ui: &mut eframe::egui::Ui, t: &mut TransformParams) {
    grid_xyz(ui, "Posição", &mut t.px, &mut t.py, &mut t.pz);
    grid_xyz(ui, "Rotação", &mut t.rx, &mut t.ry, &mut t.rz);
    grid_xyz(ui, "Escala", &mut t.sx, &mut t.sy, &mut t.sz);
}

pub fn show_output(ui: &mut eframe::egui::Ui, saida: &mut SaidaParams) {
    grid_2(ui, "Brilho", &mut saida.brilho, 0.0..=2.0, "", 2);
    grid_2(ui, "Contraste", &mut saida.contraste, 0.0..=2.0, "", 2);
    grid_2(ui, "Saturação", &mut saida.saturacao, 0.0..=2.0, "", 2);
}
