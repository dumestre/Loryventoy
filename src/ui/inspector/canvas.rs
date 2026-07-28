use crate::nodes::ProjetoConfig;

use super::{draggable_value, PRESETS_RESOLUCAO};

use eframe::egui::{ComboBox, DragValue, Grid, TextStyle};

pub fn show(ui: &mut eframe::egui::Ui, cfg: &mut ProjetoConfig) {
    let atual = PRESETS_RESOLUCAO
        .iter()
        .find(|(_, w, h)| *w == cfg.largura && *h == cfg.altura)
        .map(|(nome, _, _)| *nome)
        .unwrap_or("Personalizado");
    let font = TextStyle::Button.resolve(ui.style());
    let largura_combo = std::iter::once("Personalizado")
        .chain(PRESETS_RESOLUCAO.iter().map(|(n, _, _)| *n))
        .map(|t| {
            ui.ctx().fonts_mut(|f| {
                f.layout_no_wrap(t.to_string(), font.clone(), eframe::egui::Color32::WHITE)
                    .size()
                    .x
            })
        })
        .fold(0.0f32, f32::max)
        + 24.0;

    Grid::new("canvas_cfg")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Preset");
            ComboBox::from_id_salt("preset_res")
                .selected_text(atual)
                .width(largura_combo)
                .show_ui(ui, |ui| {
                    for (nome, w, h) in PRESETS_RESOLUCAO {
                        let sel = *w == cfg.largura && *h == cfg.altura;
                        if ui.selectable_label(sel, *nome).clicked() {
                            cfg.largura = *w;
                            cfg.altura = *h;
                        }
                    }
                });
            ui.end_row();

            ui.label("Largura");
            ui.add(DragValue::new(&mut cfg.largura).range(16..=7680));
            ui.end_row();

            ui.label("Altura");
            ui.add(DragValue::new(&mut cfg.altura).range(16..=4320));
            ui.end_row();

            ui.label("FPS");
            draggable_value(ui, &mut cfg.fps, 1.0..=120.0, 0.1, "", 1);
            ui.end_row();

            ui.label("Duração");
            draggable_value(ui, &mut cfg.duracao_seg, 0.1..=3600.0, 0.1, " s", 1);

            ui.label("Fundo");
            ui.horizontal(|ui| {
                let mut fundo = eframe::egui::Color32::from_rgba_unmultiplied(
                    cfg.fundo.r,
                    cfg.fundo.g,
                    cfg.fundo.b,
                    cfg.fundo.a,
                );
                if ui.color_edit_button_srgba(&mut fundo).changed() {
                    cfg.fundo =
                        crate::domain::Color::from_rgba(fundo.r(), fundo.g(), fundo.b(), fundo.a());
                }
                ui.label(super::hex_de(fundo));
            });
            ui.end_row();
        });
}
