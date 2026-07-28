use crate::graph_editor::NodeId;
use crate::domain::LayerEntry;
use crate::nodes::LayerParams;

use super::{AcaoInspector, icon_button, hover_bg, HOVER_VERDE, HOVER_VERMELHO};

pub fn show(
    ui: &mut eframe::egui::Ui,
    layer: &mut LayerParams,
    cenas: &[(String, NodeId)],
    node_id: NodeId,
    renaming_layer: &mut Option<(NodeId, usize)>,
) -> AcaoInspector {
    let mut acao = AcaoInspector::Nenhuma;
    let h = render_layer_header(ui, &mut layer.cena, cenas);
    if h != AcaoInspector::Nenhuma {
        acao = h;
    }
    let count = layer.layers.len();
    for rev_i in 0..count {
        let i = count - 1 - rev_i;
        let is_renaming = *renaming_layer == Some((node_id, i));
        let (r, rename_changed) = render_layer_row(
            ui,
            i,
            &mut layer.layers,
            layer.selected,
            node_id,
            is_renaming,
        );
        if r != AcaoInspector::Nenhuma && acao == AcaoInspector::Nenhuma {
            acao = r;
        }
        if rename_changed {
            *renaming_layer = if is_renaming {
                None
            } else {
                Some((node_id, i))
            };
            if is_renaming {
                acao = AcaoInspector::RenomearLayerEntry(node_id, i);
            }
        }
    }
    acao
}

/// Renderiza o cabeçalho do Layer (combo de cena + separador + botão add).
pub fn render_layer_header(
    ui: &mut eframe::egui::Ui,
    cena: &mut String,
    cenas: &[(String, NodeId)],
) -> AcaoInspector {
    let mut acao = AcaoInspector::Nenhuma;
    super::grid_combo_cena(ui, "Cena", cena, cenas);
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(2.0);
    ui.vertical_centered(|ui| {
        let btn = icon_button(
            ui,
            eframe::egui::include_image!("../icons/add_clean.svg").into(),
            16.0,
        )
        .on_hover_text("Adicionar Layer");
        hover_bg(ui, &btn, HOVER_VERDE);
        if btn.clicked() {
            acao = AcaoInspector::CriarLayerEntry;
        }
    });
    ui.add_space(4.0);
    acao
}

/// Renderiza uma única row de layer (cor, toggle, nome, delete, setas).
pub fn render_layer_row(
    ui: &mut eframe::egui::Ui,
    i: usize,
    layers: &mut [LayerEntry],
    selected: usize,
    node_id: NodeId,
    is_renaming: bool,
) -> (AcaoInspector, bool) {
    use eframe::egui::{Align, Layout, Sense, Stroke, TextStyle};
    let mut acao = AcaoInspector::Nenhuma;
    let mut start_rename = false;
    let mut finish_rename = false;
    let is_selected = selected == i;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(eframe::egui::Vec2::new(14.0, 14.0), Sense::hover());
        let cor32 = eframe::egui::Color32::from_rgba_unmultiplied(
            layers[i].cor.r,
            layers[i].cor.g,
            layers[i].cor.b,
            layers[i].cor.a,
        );
        let alpha = if layers[i].visivel { 1.0 } else { 0.35 };
        ui.painter()
            .circle_filled(rect.center(), 5.0, cor32.gamma_multiply(alpha));
        if !layers[i].visivel {
            ui.painter().circle_stroke(
                rect.center(),
                5.0,
                Stroke::new(1.0, eframe::egui::Color32::from_rgb(80, 80, 90)),
            );
        }
        let vis_btn = if layers[i].visivel {
            icon_button(
                ui,
                eframe::egui::include_image!("../icons/view_on.svg").into(),
                14.0,
            )
            .on_hover_text("Ocultar layer")
        } else {
            icon_button(
                ui,
                eframe::egui::include_image!("../icons/view_off.svg").into(),
                14.0,
            )
            .on_hover_text("Mostrar layer")
        };
        hover_bg(ui, &vis_btn, HOVER_VERDE);
        if vis_btn.clicked() {
            layers[i].visivel = !layers[i].visivel;
        }
        if is_renaming {
            let mut renamed = false;
            let edit = ui.add(
                egui::TextEdit::singleline(&mut layers[i].nome)
                    .desired_width(120.0)
                    .font(TextStyle::Monospace)
                    .lock_focus(true),
            );
            if edit.has_focus() && !edit.lost_focus() {
                ui.ctx().memory_mut(|m| m.request_focus(edit.id));
            }
            if edit.has_focus() {
                ui.input(|i| {
                    if i.key_pressed(egui::Key::Escape) {
                        renamed = true;
                    }
                });
            }
            if edit.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                finish_rename = true;
            }
            if renamed {
                start_rename = true;
            }
        } else {
            let nome_txt = if is_selected {
                egui::RichText::new(&layers[i].nome)
                    .strong()
                    .color(eframe::egui::Color32::from_rgb(220, 220, 240))
            } else {
                egui::RichText::new(&layers[i].nome)
            };
            let resp = ui
                .scope(|ui| {
                    ui.visuals_mut().widgets.hovered.bg_fill = eframe::egui::Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill = eframe::egui::Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.active.bg_fill = eframe::egui::Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.active.weak_bg_fill = eframe::egui::Color32::TRANSPARENT;
                    ui.add(egui::Label::new(nome_txt).sense(Sense::click()))
                })
                .inner;
            if resp.double_clicked() {
                start_rename = true;
            } else if resp.clicked() {
                acao = AcaoInspector::SelecionarLayer(node_id, i);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let del_btn = icon_button(
                ui,
                eframe::egui::include_image!("../icons/delete_clean.svg").into(),
                13.0,
            )
            .on_hover_text("Remover layer");
            hover_bg(ui, &del_btn, HOVER_VERMELHO);
            if del_btn.clicked() {
                acao = AcaoInspector::RemoverLayerEntry(node_id, i);
            }
            let down_btn = icon_button(
                ui,
                eframe::egui::include_image!("../icons/arrow_down_clean.svg").into(),
                13.0,
            )
            .on_hover_text("Mover para trás");
            hover_bg(ui, &down_btn, HOVER_VERDE);
            if down_btn.clicked() {
                acao = AcaoInspector::SubirLayerEntry(node_id, i);
            }
            let up_btn = icon_button(
                ui,
                eframe::egui::include_image!("../icons/arrow_up_clean.svg").into(),
                13.0,
            )
            .on_hover_text("Mover para frente");
            hover_bg(ui, &up_btn, HOVER_VERDE);
            if up_btn.clicked() {
                acao = AcaoInspector::DescerLayerEntry(node_id, i);
            }
        });
    });
    (acao, start_rename || finish_rename)
}
