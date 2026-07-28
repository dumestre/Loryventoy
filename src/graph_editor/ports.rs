#![allow(dead_code)]

use eframe::egui::epaint::CircleShape;
use eframe::egui::{Color32, Pos2, Shape, Stroke, Vec2};

use super::types::cor_tipo_no;
use crate::nodes::{portos, TipoNo};
use crate::ui::node_component;

pub fn port_offsets(tipo: TipoNo, half: Vec2) -> (Vec<Vec2>, Vec<Vec2>) {
    let spec = portos(tipo);
    let n = spec.entradas.len().max(spec.saidas.len()).max(1);
    let top = -half.y + node_component::CABECALHO_H + node_component::MARGEM_Y;
    let bottom = half.y - node_component::MARGEM_Y;
    let span = (bottom - top).max(0.0);
    let y_uniforme = |i: usize| -> f32 {
        if n <= 1 {
            0.0
        } else {
            top + (i as f32 + 0.5) * span / (n as f32)
        }
    };
    let y = |i: usize, nome: &str| -> f32 {
        match node_component::linha_y(tipo, nome) {
            Some(ry) => top + ry.clamp(0.0, span),
            None => y_uniforme(i),
        }
    };
    let ins: Vec<Vec2> = spec
        .entradas
        .iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(-half.x, y(i, &p.nome)))
        .collect();
    let outs: Vec<Vec2> = spec
        .saidas
        .iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(half.x, y(i, &p.nome)))
        .collect();
    (ins, outs)
}

pub fn desenhar_portos(
    painter: &egui::Painter,
    pos_screen: Pos2,
    half: Vec2,
    tipo: TipoNo,
    zoom: f32,
) {
    let fill = Color32::from_rgb(30, 30, 40);
    let accent = cor_tipo_no(tipo);
    let port_r = (4.5 * zoom).clamp(2.5, 7.0);
    let (ins, outs) = port_offsets(tipo, half);

    for off in &ins {
        let c = pos_screen + *off * zoom;
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r,
            fill: accent,
            stroke: Stroke::NONE,
        }));
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r * 0.45,
            fill,
            stroke: Stroke::NONE,
        }));
    }

    for (i, off) in outs.iter().enumerate() {
        let c = pos_screen + *off * zoom;
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r,
            fill: accent,
            stroke: Stroke::NONE,
        }));
        if portos(tipo).saidas.get(i).map_or(false, |p| p.is_vetor()) {
            painter.add(Shape::Circle(CircleShape {
                center: c,
                radius: port_r * 0.45,
                fill,
                stroke: Stroke::NONE,
            }));
        }
    }
}

pub fn node_screen_rect(pos_screen: Pos2, half: Vec2, zoom: f32) -> eframe::egui::Rect {
    let size = half * 2.0 * zoom;
    eframe::egui::Rect::from_center_size(pos_screen, size)
}

pub fn port_in_screen(node_center: Pos2, half: Vec2, zoom: f32, tipo: TipoNo, idx: usize) -> Pos2 {
    let (ins, _) = port_offsets(tipo, half);
    let off = ins
        .get(idx)
        .copied()
        .unwrap_or_else(|| Vec2::new(-half.x, 0.0));
    node_center + off * zoom
}

pub fn port_out_screen(node_center: Pos2, half: Vec2, zoom: f32, tipo: TipoNo, idx: usize) -> Pos2 {
    let (_, outs) = port_offsets(tipo, half);
    let off = outs
        .get(idx)
        .copied()
        .unwrap_or_else(|| Vec2::new(half.x, 0.0));
    node_center + off * zoom
}

pub fn tamanho(label: &str) -> Vec2 {
    let tipo = TipoNo::from_label(label).unwrap_or(TipoNo::Transform);
    node_component::content_size(tipo)
}
