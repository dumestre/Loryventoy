use crate::graph_editor::NodeId;
use crate::nodes::ShapeParams;

use super::{editar_cor, grid_2, grid_combo_cena, grid_combo_tipo, grid_xyz, registrar_linha};

use eframe::egui::Grid;

pub fn show(ui: &mut eframe::egui::Ui, shape: &mut ShapeParams, cenas: &[(String, NodeId)]) {
    let ShapeParams {
        cena,
        tipo,
        px,
        py,
        largura,
        altura,
        rotacao,
        cor,
        seed,
        noise_scale,
        amp,
        veloc,
        trim_inicio,
        trim_fim,
    } = shape;
    grid_combo_cena(ui, "Cena", cena, cenas);
    grid_combo_tipo(ui, "Tipo", tipo);
    grid_xyz(ui, "Posição", px, py, &mut 0.0);
    grid_2(ui, "Largura", largura, 1.0..=8000.0, "", 1);
    grid_2(ui, "Altura", altura, 1.0..=8000.0, "", 1);
    grid_2(ui, "Rotação", rotacao, -360.0..=360.0, "°", 1);
    let rc = Grid::new("shape_cor")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Cor");
            ui.horizontal(|ui| {
                editar_cor(ui, cor);
            });
            ui.end_row();
        });
    registrar_linha("Cor", &rc.response);
    grid_2(ui, "Seed", seed, 0.0..=9999.0, "", 0);
    grid_2(ui, "Ruído", noise_scale, 0.01..=5.0, "", 2);
    grid_2(ui, "Amplitude", amp, 0.0..=500.0, "", 1);
    grid_2(ui, "Velocidade", veloc, 0.0..=10.0, "x", 2);
    {
        let r = Grid::new("trim_inicio")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim início");
                let mut v = *trim_inicio * 100.0;
                if super::draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    *trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim início", &r.response);
    }
    {
        let r = Grid::new("trim_fim")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim fim");
                let mut v = *trim_fim * 100.0;
                if super::draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    *trim_fim = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim fim", &r.response);
    }
}
