use eframe::egui::{
    Align, Area, Color32, CornerRadius, CursorIcon, Id, Image, Layout, Order,
    Pos2, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};
use eframe::egui::color_picker::Alpha;
use eframe::egui::epaint::RectShape;
use eframe::egui::Popup;

use super::GraphPanel;
use super::types::NodeId;
use super::ports::tamanho;

pub const CORES_GRUPO: [Color32; 8] = [
    Color32::from_rgb(90, 140, 220),
    Color32::from_rgb(220, 130, 90),
    Color32::from_rgb(110, 200, 120),
    Color32::from_rgb(200, 120, 200),
    Color32::from_rgb(220, 195, 90),
    Color32::from_rgb(90, 200, 200),
    Color32::from_rgb(180, 150, 225),
    Color32::from_rgb(225, 150, 175),
];

#[derive(Clone)]
pub struct Grupo {
    pub nos: Vec<NodeId>,
    pub titulo: String,
    pub cor: Color32,
    pub handle: Rect,
}

impl GraphPanel {
    pub(super) fn bounding_grupo_idx(&self, i: usize) -> Option<Rect> {
        let grp = self.grupos.get(i)?;
        let mut r: Option<Rect> = None;
        for idx in &grp.nos {
            if let Some(pos) = self.editor_state.node_positions.get(*idx) {
                let label = self.obter_label(*idx);
                let half = tamanho(&label);
                let nr = Rect::from_center_size(*pos, half * 2.0);
                r = Some(match r {
                    Some(c) => c.union(nr),
                    None => nr,
                });
            }
        }
        r
    }

    pub(super) fn rect_grupo(
        &self,
        i: usize,
        pan: eframe::egui::Vec2,
        _zoom: f32,
        rect: Rect,
    ) -> Option<(Rect, f32)> {
        let bb = self.bounding_grupo_idx(i)?.expand(16.0);
        let min_s = self.canvas_para_screen(bb.min, pan, _zoom, rect);
        let max_s = self.canvas_para_screen(bb.max, pan, _zoom, rect);
        let header_h = 22.0;
        let surf = Rect::from_min_max(Pos2::new(min_s.x, min_s.y - header_h), max_s);
        Some((surf, header_h))
    }

    pub(super) fn grupo_header_sob(
        &self,
        p: Pos2,
        pan: eframe::egui::Vec2,
        zoom: f32,
        rect: Rect,
    ) -> Option<usize> {
        for i in (0..self.grupos.len()).rev() {
            if let Some((surf, header_h)) = self.rect_grupo(i, pan, zoom, rect) {
                let header =
                    Rect::from_min_max(surf.min, Pos2::new(surf.max.x, surf.min.y + header_h));
                if header.contains(p) && !self.grupos[i].handle.contains(p) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(super) fn desenhar_grupos_fundo(
        &self,
        ui: &Ui,
        rect: Rect,
        pan: eframe::egui::Vec2,
        zoom: f32,
    ) {
        let painter = ui.painter().with_clip_rect(rect);
        for i in 0..self.grupos.len() {
            let (surf, header_h) = match self.rect_grupo(i, pan, zoom, rect) {
                Some(v) => v,
                None => continue,
            };
            if !rect.intersects(surf) {
                continue;
            }
            let c = self.grupos[i].cor;
            let fill = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 28);
            let head = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 210);
            painter.add(RectShape::new(
                surf,
                CornerRadius::same(10),
                fill,
                Stroke::new(1.5, c),
                StrokeKind::Inside,
            ));
            let header =
                Rect::from_min_max(surf.min, Pos2::new(surf.max.x, surf.min.y + header_h));
            let mut cr = CornerRadius::same(10);
            cr.sw = 0;
            cr.se = 0;
            painter.add(RectShape::new(header, cr, head, Stroke::NONE, StrokeKind::Inside));
        }
    }

    pub(super) fn desenhar_grupos_header(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        pan: eframe::egui::Vec2,
        zoom: f32,
    ) {
        for i in 0..self.grupos.len() {
            let (surf, _header_h) = match self.rect_grupo(i, pan, zoom, rect) {
                Some(v) => v,
                None => continue,
            };
            if !rect.intersects(surf) {
                continue;
            }
            let grp = &mut self.grupos[i];
            let txt = Color32::from_rgb(20, 20, 26);
            let header_h = _header_h;
            Area::new(Id::new(("grupo_header", i)))
                .order(Order::Middle)
                .fixed_pos(surf.min)
                .movable(false)
                .constrain(false)
                .show(ui.ctx(), |ui| {
                    ui.set_clip_rect(rect);
                    let mut usado = Rect::ZERO;
                    ui.allocate_ui_with_layout(
                        Vec2::new(surf.width(), header_h),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.add_space(8.0);
                            let fonte = eframe::egui::TextStyle::Body.resolve(ui.style());
                            let larg_txt = ui
                                .painter()
                                .layout_no_wrap(grp.titulo.clone(), fonte, txt)
                                .size()
                                .x
                                .max(24.0)
                                + 6.0;
                            ui.add(
                                TextEdit::singleline(&mut grp.titulo)
                                    .frame(eframe::egui::Frame::NONE)
                                    .desired_width(larg_txt)
                                    .text_color(txt),
                            );
                            ui.add_space(2.0);
                            let (rect_btn, btn) =
                                ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                            if btn.hovered() {
                                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                ui.painter().rect_filled(
                                    rect_btn,
                                    5.0,
                                    Color32::from_rgba_unmultiplied(0, 0, 0, 45),
                                );
                            }
                            let img = Image::new(eframe::egui::include_image!("../ui/icons/cor.svg"))
                                .fit_to_exact_size(Vec2::splat(14.0));
                            img.paint_at(
                                ui,
                                Rect::from_center_size(rect_btn.center(), Vec2::splat(14.0)),
                            );
                            Popup::menu(&btn).show(|ui| {
                                eframe::egui::color_picker::color_picker_color32(
                                    ui,
                                    &mut grp.cor,
                                    Alpha::Opaque,
                                );
                            });
                            usado = ui.min_rect();
                        },
                    );
                    grp.handle = usado;
                });
        }
    }
}
