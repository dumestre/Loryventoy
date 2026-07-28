use crate::nodes::AnimParams;

use super::{grid_combo_anim_alvo, grid_combo_loop, draggable_value};

use eframe::egui::{ComboBox, Grid};

pub fn show(ui: &mut eframe::egui::Ui, anim: &mut AnimParams) {
    grid_combo_anim_alvo(ui, "Alvo", &mut anim.alvo);
    grid_combo_loop(ui, "Loop", &mut anim.loop_mode);
    editor_segmentos(ui, &mut anim.segmentos);
}

pub fn editor_segmentos(ui: &mut eframe::egui::Ui, segs: &mut Vec<crate::domain::AnimSeg>) {
    use crate::domain::{AnimSeg, Easing};
    let easings = ["Linear", "Ease-in", "Ease-out", "Ease-in-out", "Step"];
    ui.label(eframe::egui::RichText::new("Trechos").strong());
    let mut remover: Option<usize> = None;
    for (i, s) in segs.iter_mut().enumerate() {
        Grid::new(("anim_seg", i))
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("t");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.t_ini, 0.0..=3600.0, 0.05, "s", 2);
                    ui.label("→");
                    draggable_value(ui, &mut s.t_fim, 0.0..=3600.0, 0.05, "s", 2);
                });
                ui.end_row();
                ui.label("de");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.v_ini[0], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                    draggable_value(ui, &mut s.v_ini[1], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                });
                ui.end_row();
                ui.label("para");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.v_fim[0], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                    draggable_value(ui, &mut s.v_fim[1], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                });
                ui.end_row();
                ui.label("curva");
                let mut e = s.easing.to_u8() as usize;
                let atual = easings.get(e).copied().unwrap_or(easings[0]);
                ComboBox::from_id_salt(("anim_ease_cb", i))
                    .selected_text(atual)
                    .show_ui(ui, |ui| {
                        for (k, n) in easings.iter().enumerate() {
                            if ui.selectable_label(e == k, *n).clicked() {
                                e = k;
                            }
                        }
                    });
                s.easing = Easing::from_u8(e as u8);
                ui.end_row();
            });
        if ui.small_button("Remover trecho").clicked() {
            remover = Some(i);
        }
        ui.separator();
    }
    if let Some(i) = remover {
        if segs.len() > 1 {
            segs.remove(i);
        }
    }
    if ui.small_button("+ Trecho").clicked() {
        let (t0, v) = segs
            .last()
            .map(|s| (s.t_fim, s.v_fim))
            .unwrap_or((0.0, [0.0, 0.0]));
        segs.push(AnimSeg {
            t_ini: t0,
            t_fim: t0 + 1.0,
            v_ini: v,
            v_fim: v,
            easing: Easing::EaseInOut,
        });
    }
}
