use crate::graph_editor::NodeId;
use crate::nodes::CenaParams;

use super::{grid_2, grid_texto, AcaoInspector};

pub fn show(
    ui: &mut eframe::egui::Ui,
    cena: &mut CenaParams,
    cenas: &[(String, NodeId)],
) -> AcaoInspector {
    let mut acao = AcaoInspector::Nenhuma;
    grid_texto(ui, "Cena", &mut cena.nome_cena);
    eframe::egui::Grid::new("cena_ativa")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Ativa");
            ui.checkbox(&mut cena.ativa, "");
            ui.end_row();
        });
    grid_2(ui, "Zoom", &mut cena.zoom, 0.01..=10.0, "", 2);
    grid_2(ui, "Ângulo", &mut cena.angulo, -360.0..=360.0, "°", 1);
    grid_2(ui, "Opacidade", &mut cena.opacidade, 0.0..=1.0, "", 2);

    ui.separator();
    ui.label(eframe::egui::RichText::new("Cenas disponíveis").strong());
    for (nome, idx) in cenas {
        ui.horizontal(|ui| {
            ui.label(nome);
            if ui.small_button("Focar").clicked() {
                acao = AcaoInspector::FocarCena(*idx);
            }
        });
    }
    if cenas.is_empty() {
        ui.label("(crie marcadores na timeline)");
    }
    acao
}
