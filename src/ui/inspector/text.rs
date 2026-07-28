use crate::graph_editor::NodeId;
use crate::nodes::TextParams;

use super::{grid_2, grid_combo_cena, grid_xyz, editar_cor, registrar_linha, draggable_value};

use eframe::egui::Grid;

pub fn show(ui: &mut eframe::egui::Ui, texto: &mut TextParams, cenas: &[(String, NodeId)]) {
    grid_combo_cena(ui, "Cena", &mut texto.cena, cenas);
    Grid::new("texto_conteudo")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Conteúdo");
            ui.text_edit_singleline(&mut texto.conteudo);
            ui.end_row();
        });
    grid_2(ui, "Tamanho", &mut texto.tamanho, 1.0..=2000.0, "", 1);
    Grid::new("texto_estilo")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Estilo");
            ui.horizontal(|ui| {
                ui.checkbox(&mut texto.negrito, "Negrito");
                ui.checkbox(&mut texto.italico, "Itálico");
            });
            ui.end_row();
        });
    grid_xyz(ui, "Posição", &mut texto.px, &mut texto.py, &mut 0.0);
    let rc = Grid::new("texto_cor")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Cor");
            ui.horizontal(|ui| {
                editar_cor(ui, &mut texto.cor);
            });
            ui.end_row();
        });
    registrar_linha("Cor", &rc.response);
    {
        let r = Grid::new("texto_trim_inicio")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim início");
                let mut v = texto.trim_inicio * 100.0;
                if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    texto.trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim início", &r.response);
    }
    {
        let r = Grid::new("texto_trim_fim")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim fim");
                let mut v = texto.trim_fim * 100.0;
                if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    texto.trim_fim = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim fim", &r.response);
    }
}
