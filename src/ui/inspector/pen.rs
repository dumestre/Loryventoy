use crate::nodes::PenParams;

use super::{grid_2, grid_combo_cena, grid_xyz, grid_escala, editar_cor, registrar_linha, draggable_value};

use eframe::egui::{Color32, Grid};

pub fn show(ui: &mut eframe::egui::Ui, pen: &mut PenParams, cenas: &[(String, crate::graph_editor::NodeId)]) {
    grid_combo_cena(ui, "Cena", &mut pen.cena, cenas);
    grid_xyz(ui, "Posição", &mut pen.pos_x, &mut pen.pos_y, &mut 0.0);
    grid_2(ui, "Espessura", &mut pen.espessura, 0.0..=100.0, "px", 1);
    Grid::new("pen_preench")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Preencher");
            ui.checkbox(&mut pen.preenchimento, "");
            ui.end_row();
        });
    grid_2(ui, "Cantos", &mut pen.cantos, 0.0..=1.0, "", 2);
    grid_2(ui, "Ordem", &mut pen.ordem, -100.0..=100.0, "", 1);
    grid_escala(ui, &mut pen.escala_x, &mut pen.escala_y);
    grid_2(ui, "Seed", &mut pen.seed, 0.0..=9999.0, "", 0);
    Grid::new("pen_cor")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Cor traço");
            ui.horizontal(|ui| {
                editar_cor(ui, &mut pen.cor);
            });
            ui.end_row();
            ui.label("Cor preench.");
            ui.horizontal(|ui| {
                editar_cor(ui, &mut pen.cor_fill);
            });
            ui.end_row();
        });
    Grid::new("pen_codigo")
        .num_columns(1)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Código DSL");
            let largura = (ui.available_width() * 1.6).max(280.0);
            eframe::egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    ui.add(
                        eframe::egui::TextEdit::multiline(&mut pen.codigo)
                            .code_editor()
                            .font(eframe::egui::TextStyle::Monospace)
                            .desired_rows(6)
                            .desired_width(largura),
                    );
                });
            ui.end_row();
        });
    ui.horizontal_wrapped(|ui| {
        let cmds = [
            "move", "line", "rect", "circle", "bezier", "close", "fill on",
            "stroke", "color", "stroke_color", "fill_color", "repeat", "if",
        ];
        for c in cmds {
            if ui.small_button(c).clicked() {
                let sep = if pen.codigo.is_empty() || pen.codigo.ends_with('\n') {
                    ""
                } else {
                    "\n"
                };
                pen.codigo.push_str(&format!("{sep}{c} "));
            }
        }
    });
    pen.erro = match crate::dsl::Program::parse(&pen.codigo) {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    };

    ui.add_space(6.0);
    ui.separator();
    let log_txt = match pen.erro.as_deref() {
        Some(e) => format!("ERRO: {e}"),
        None => "OK: código válido.".to_string(),
    };
    ui.horizontal(|ui| {
        ui.label(eframe::egui::RichText::new("Log").strong());
        if ui.small_button("Copiar log").clicked() {
            ui.ctx().copy_text(log_txt.clone());
        }
    });
    eframe::egui::ScrollArea::vertical()
        .id_salt("pen_log")
        .max_height(70.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let cor = if pen.erro.is_some() {
                Color32::from_rgb(230, 120, 120)
            } else {
                Color32::from_rgb(150, 200, 150)
            };
            ui.colored_label(cor, &log_txt);
        });
    ui.add_space(6.0);
    {
        let r = Grid::new("pen_trim_inicio")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim início");
                let mut v = pen.trim_inicio * 100.0;
                if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    pen.trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim início", &r.response);
    }
    {
        let r = Grid::new("pen_trim_fim")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("Trim fim");
                let mut v = pen.trim_fim * 100.0;
                if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                    pen.trim_fim = (v / 100.0).clamp(0.0, 1.0);
                }
                ui.end_row();
            });
        registrar_linha("Trim fim", &r.response);
    }
}
